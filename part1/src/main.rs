use std::{fmt::Display, fs, path::PathBuf};

use clap::Parser;

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
        match current_op {
            None => {
                dasm.push_str("\n");
                current_op = Some(decode_op(b));
            }
            Some(mut op) => {
                op.decode(b);
                if op.done() {
                    dasm.push_str(format!("{op}").as_str());
                    current_op = None;
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

#[derive(Debug, Clone, Copy)]
enum MOV {
    Reg(MovReg),
}

impl MOV {
    fn decode(&mut self, b: u8) {
        match self {
            Self::Reg(mr) => mr.decode(b),
        }
    }

    fn done(&self) -> bool {
        match self {
            Self::Reg(mr) => mr.done(),
        }
    }
}

impl Display for MOV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Reg(mr) => format!("{mr}"),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
struct MovReg {
    d: SBF,
    w: SBF,
    data: Option<u8>,
    dlo: Option<u8>,
    dhi: Option<u8>,
    done: bool,
}

impl MovReg {
    pub fn done(&self) -> bool {
        self.done
    }

    pub fn decode(&mut self, b: u8) {
        if self.data == None {
            self.data = Some(b);
            if self.r#mod() == MOD::Reg {
                self.done = true;
            }
        } else {
            match self.r#mod() {
                MOD::Reg => self.done = true,
                _ => panic!(""),
            }
        }
    }

    pub fn r#mod(&self) -> MOD {
        match self.data {
            None => panic!("data is None"),
            Some(d) => ((d >> 6) & 0x3).into(),
        }
    }

    pub fn src(&self) -> REG {
        if self.d.0 { self.rm() } else { self.reg() }
    }

    pub fn dest(&self) -> REG {
        if self.d.0 { self.reg() } else { self.rm() }
    }

    pub fn reg(&self) -> REG {
        match self.data {
            None => panic!("data is None"),
            Some(d) => {
                let val = (d >> 3) & 0x7;
                let rf = RegField::new(val, self.w.0);
                rf.reg
            }
        }
    }

    pub fn rm(&self) -> REG {
        match self.data {
            None => panic!("data is None"),
            Some(d) => {
                let val = d & 0x7;
                let rf = RegField::new(val, self.w.0);
                rf.reg
            }
        }
    }
}

impl Display for MovReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = String::from("mov ");
        i.push_str(format!("{}, ", self.dest()).as_str());
        i.push_str(format!("{}", self.src()).as_str());
        write!(f, "{i}")
    }
}

enum REG {
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
        Op::MOV(MOV::Reg(MovReg {
            d: ((opcode >> 1) & 1).into(),
            w: (opcode & 1).into(),
            done: false,
            data: None,
            dlo: None,
            dhi: None,
        }))
    } else {
        todo!("{opcode}")
    }
}
