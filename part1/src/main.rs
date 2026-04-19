use clap::Parser;
use std::{env::current_dir, fmt::Display, fs, io::Write, iter::Enumerate, process::Command};

mod add;
mod jmp;
mod mov;
use mov::MOV;

use crate::{
    add::{Acc, Arith, Arithmetic, Imm, is_i_to_a, is_i_to_r},
    jmp::{JMP, JmpRegistry},
    mov::{ImmRM, ImmReg, MemAcc, MovReg},
};

#[derive(Parser, Debug)]
struct Args {
    listing: u8,
}

fn main() {
    let args = Args::parse();
    let bytes = get_listing(args.listing);
    let dasm = disassemble(&bytes);
    println!("{dasm}");
}

fn get_listing(listing: u8) -> Vec<u8> {
    let mut path = current_dir().unwrap();
    path.push(format!("src/listings/{listing}"));
    fs::read(path).unwrap()
}

struct Dissassembly {
    pub bits: u8,
    pub jr: JmpRegistry,
    pub ops: Vec<Op>,
}

impl Display for Dissassembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ops: Vec<String> = self.ops.iter().map(|o| o.to_string()).collect();
        let s = vec![format!("bits {}", self.bits), ops.join("\n")].join("\n");
        write!(f, "{s}")
    }
}

fn disassemble(asm: &[u8]) -> String {
    let jmp_registry = JmpRegistry::new();
    let mut dasm = Dissassembly {
        bits: 16,
        ops: vec![],
        jr: jmp_registry,
    };
    let mut current_op: Option<Op> = None;
    let mut iter = asm.iter().enumerate();

    while let Some((_, b)) = iter.next() {
        match current_op {
            None => {
                let op = decode_op(*b, &mut iter, &mut dasm.jr);
                if op.done() {
                    dasm.ops.push(op);
                } else {
                    current_op = Some(op);
                }
            }
            Some(mut op) => {
                if !op.done() {
                    op.decode(*b);
                }
                if op.done() {
                    dasm.ops.push(op);
                    current_op = None;
                } else {
                    current_op = Some(op);
                }
            }
        }
    }
    dasm.to_string()
}

fn assemble(dasm: &str) -> Vec<u8> {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(dasm.as_bytes()).unwrap();

    let output = Command::new("nasm")
        .args(["-o", "/dev/stdout", tmp.path().to_str().unwrap()])
        .output()
        .expect("failed to start nasm");

    if !output.status.success() {
        panic!("nasm: {}", String::from_utf8_lossy(&output.stderr));
    }
    output.stdout
}

#[derive(Debug, Clone, Copy)]
struct SBF(bool);

impl From<u8> for SBF {
    fn from(value: u8) -> Self {
        Self(value != 0)
    }
}

impl From<bool> for SBF {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

#[derive(PartialEq, Debug)]
enum MOD {
    Mem,  // memory mode, no displacement
    Byte, // memory mode, 8 bit displacement
    Word, // memory mode, 16 bit displacement
    Reg,  // register mode
}

impl From<u8> for MOD {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Mem,
            1 => Self::Byte,
            2 => Self::Word,
            3 => Self::Reg,
            _ => panic!("invalid MOD value: {v}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RM {
    Mem(MemData),
    Byte(MemData, i16),
    Word(MemData, u16),
    Reg(REG),
}

impl Display for RM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Reg(r) => format!("{r}"),
            Self::Mem(md) => format!("[{md}]"),
            Self::Byte(md, b) => match md {
                MemData::Direct(_) => {
                    let mut s = String::from("bp");
                    if *b != 0 {
                        if *b > 0 {
                            s.push_str(format!(" + ").as_str());
                        }
                        if *b < 0 {
                            s.push_str(format!(" - ").as_str());
                        }
                        s.push_str(format!("{b}").as_str());
                    }
                    format!("[{s}]")
                }
                _ => {
                    let mut s = format!("{md}");
                    if *b > 0 {
                        s.push_str(format!(" + ").as_str());
                    }
                    if *b < 0 {
                        s.push_str(format!(" - ").as_str());
                    }
                    s.push_str(format!("{}", b.abs()).as_str());
                    format!("[{s}]")
                }
            },
            Self::Word(md, w) => match md {
                MemData::Direct(_) => {
                    let mut s = String::from("bp");
                    if *w != 0 {
                        s.push_str(format!(" + {w}").as_str());
                    }
                    format!("[{s}]")
                }
                _ => {
                    let mut s = format!("{md}");
                    let val = *w as i16;
                    if val > 0 {
                        s.push_str(format!(" + ").as_str());
                    }
                    if val < 0 {
                        s.push_str(format!(" - ").as_str());
                    }
                    s.push_str(format!("{}", val.abs()).as_str());
                    format!("[{s}]")
                }
            },
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemData {
    BXSI,
    BXDI,
    BPSI,
    BPDI,
    SI,
    DI,
    Direct(Option<u16>),
    BX,
}

impl From<u8> for MemData {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::BXSI,
            1 => Self::BXDI,
            2 => Self::BPSI,
            3 => Self::BPDI,
            4 => Self::SI,
            5 => Self::DI,
            6 => Self::Direct(None),
            7 => Self::BX,
            _ => panic!("invalid MemData value: {v}"),
        }
    }
}

impl Display for MemData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BXSI => String::from("bx + si"),
            Self::BXDI => String::from("bx + di"),
            Self::BPSI => String::from("bp + si"),
            Self::BPDI => String::from("bp + di"),
            Self::SI => String::from("si"),
            Self::DI => String::from("di"),
            Self::Direct(d) => match d {
                Some(v) => format!("{}", v),
                None => String::from("0"),
            },
            Self::BX => String::from("bx"),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
enum Op {
    MOV(MOV),
    Arith(Arith),
    Arithmetic(Arithmetic),
    JMP(JMP),
}

impl Op {
    fn decode(&mut self, b: u8) {
        match self {
            Self::MOV(m) => m.decode(b),
            Self::Arith(a) => a.decode(b),
            _ => todo!(),
        }
    }

    fn done(&self) -> bool {
        match self {
            Self::MOV(m) => m.done(),
            Self::Arith(a) => a.done(),
            Self::Arithmetic(_) => true,
            Self::JMP(_) => true,
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MOV(m) => m.to_string(),
            Self::Arith(m) => m.to_string(),
            Self::Arithmetic(a) => a.to_string(),
            Self::JMP(jmp) => jmp.to_string(),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum REG {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl Display for REG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AL => "al",
            Self::CL => "cl",
            Self::DL => "dl",
            Self::BL => "bl",
            Self::AH => "ah",
            Self::CH => "ch",
            Self::DH => "dh",
            Self::BH => "bh",
            Self::AX => "ax",
            Self::CX => "cx",
            Self::DX => "dx",
            Self::BX => "bx",
            Self::SP => "sp",
            Self::BP => "bp",
            Self::SI => "si",
            Self::DI => "di",
        };
        write!(f, "{}", s)
    }
}

impl From<u8> for REG {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::AL,
            1 => Self::CL,
            2 => Self::DL,
            3 => Self::BL,
            4 => Self::AH,
            5 => Self::CH,
            6 => Self::DH,
            7 => Self::BH,
            8 => Self::AX,
            9 => Self::CX,
            10 => Self::DX,
            11 => Self::BX,
            12 => Self::SP,
            13 => Self::BP,
            14 => Self::SI,
            15 => Self::DI,
            _ => panic!("invalid REG value: {value}"),
        }
    }
}

#[derive(Debug)]
struct RegField {
    w: bool,
    reg: REG,
    val: u8,
}

impl RegField {
    pub fn new(val: u8, w: bool) -> RegField {
        let val = val & 0x7;
        let reg: REG = match w {
            false => val.into(),
            true => (val + 0x8).into(),
        };
        RegField { w, reg, val }
    }
}

fn decode_op(opcode: u8, iter: &mut Enumerate<std::slice::Iter<u8>>, jr: &mut JmpRegistry) -> Op {
    if let Some(op) = decode_mov(opcode) {
        op
    } else if let Some(op) = decode_arithmetic(opcode, iter) {
        op
    } else if let Some(op) = JMP::decode(opcode, iter, jr) {
        Op::JMP(op)
    } else {
        todo!("{opcode:0>8b}")
    }
}

fn decode_mov(opcode: u8) -> Option<Op> {
    if ((opcode >> 2) & 0b1111_11) == 0b1000_10 {
        Some(Op::MOV(MOV::Reg(MovReg::new(opcode))))
    } else if ((opcode >> 4) & 0b1111) == 0b1011 {
        Some(Op::MOV(MOV::ImmReg(ImmReg::new(opcode))))
    } else if ((opcode >> 1) & 0b111_1111) == 0b110_0011 {
        Some(Op::MOV(MOV::ImmRM(ImmRM::new(opcode))))
    } else if ((opcode >> 1) & 0b111_1111) == 0b101_0000 {
        Some(Op::MOV(MOV::MemAcc(MemAcc::new(opcode))))
    } else if ((opcode >> 1) & 0b111_1111) == 0b101_0001 {
        Some(Op::MOV(MOV::MemAcc(MemAcc::reversed(opcode))))
    } else {
        None
    }
}

fn decode_arithmetic(b: u8, iter: &mut Enumerate<std::slice::Iter<u8>>) -> Option<Op> {
    if let Some(op) = Arithmetic::new(b, iter) {
        Some(Op::Arithmetic(op))
    } else if is_i_to_r(b) {
        Some(Op::Arith(Arith::Imm(Imm::new(b))))
    } else if is_i_to_a(b) {
        Some(Op::Arith(Arith::Acc(Acc::new(b))))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{assemble, disassemble, get_listing};

    fn test_listing(l: u8) {
        let source = get_listing(l);
        let dasm = disassemble(&source);
        let asm = assemble(&dasm);
        assert_eq!(source, asm);
    }

    #[test]
    fn listing_37() {
        test_listing(37);
    }

    #[test]
    fn listing_38() {
        test_listing(38);
    }

    #[test]
    fn listing_39() {
        test_listing(39);
    }

    #[test]
    fn listing_40() {
        test_listing(40);
    }

    #[test]
    fn listing_41() {
        test_listing(41);
    }
}
