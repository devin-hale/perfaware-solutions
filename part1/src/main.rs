use clap::Parser;
use std::{fmt::Display, fs, path::PathBuf};

mod mov;
use mov::MOV;

use crate::mov::{ImmReg, MovReg};

#[derive(Parser, Debug)]
struct Args {
    path: String,
}

fn main() {
    let args = Args::parse();
    let path = PathBuf::from(args.path);
    let bytes = fs::read(path).unwrap();

    let mut current_op: Option<Op> = None;
    let mut dasm = String::from("bits 16\n");
    for b in bytes {
        //println!("{b:0>8b}");
        match current_op {
            None => {
                dasm.push_str("\n");
                current_op = Some(decode_op(b));
            }
            Some(mut op) => {
                op.decode(b);
                if op.done() {
                    println!("{op}");
                    dasm.push_str(format!("{op}").as_str());
                    current_op = None;
                } else {
                    current_op = Some(op);
                }
            }
        }
    }
    println!("{dasm}");
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

#[derive(PartialEq)]
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
    Reg(REG),
}

impl Display for RM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Reg(r) => format!("{r}"),
            Self::Mem(md) => format!("{md}"),
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
        write!(f, "[{s}]")
    }
}

#[derive(Debug, Clone, Copy)]
enum Op {
    MOV(MOV),
}

impl Op {
    fn decode(&mut self, b: u8) {
        match self {
            Self::MOV(m) => m.decode(b),
        }
    }

    fn done(&self) -> bool {
        match self {
            Self::MOV(m) => m.done(),
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MOV(m) => format!("{m}"),
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

fn decode_op(opcode: u8) -> Op {
    if ((opcode >> 2) & 0b1111_11) == 0b1000_10 {
        Op::MOV(MOV::Reg(MovReg::new(opcode)))
    } else if ((opcode >> 4) & 0b1111) == 0b1011 {
        Op::MOV(MOV::ImmReg(ImmReg::new(opcode)))
    } else {
        todo!("{opcode:0>8b}")
    }
}
