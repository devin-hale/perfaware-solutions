use std::{fmt::Display, iter::Enumerate};

use crate::{MOD, MemData, REG, RM, RegField, SBF};

pub struct Arithmetic {
    a_type: Option<ArithmeticType>,
    reg_mov: RegMovement,
}

impl Arithmetic {
    fn new(b: u8, iter: Enumerate<std::slice::Iter<u8>>) -> Self {
        let a_type: Option<ArithmeticType> = b.try_into().ok();
        let reg_mov: RegMovement = b.into();
        let arith = Self { a_type, reg_mov };

        arith
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticType {
    Add,
    Sub,
    Cmp,
}

impl TryFrom<u8> for ArithmeticType {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Add),
            0b101 => Ok(Self::Sub),
            0b111 => Ok(Self::Cmp),
            _ => Err(()),
        }
    }
}

impl Display for ArithmeticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Cmp => "cmp",
        };
        write!(f, "{s}")
    }
}

pub enum RegMovement {
    Reg,
    Imm,
    Acc,
}

impl From<u8> for RegMovement {
    fn from(b: u8) -> Self {
        match (b >> 2) & 0b1111_11 {
            0 | 0b00_1010 | 0b00_1110 => Self::Reg,
            0b10_0000 => Self::Imm,
            _ => match (b >> 1) & 0b1111_111 {
                0b000_0010 | 0b0010_110 | 0b0011_110 => Self::Acc,
                _ => panic!("invalid RegMovement: {b}"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Arith {
    Reg(Reg),
    Imm(Imm),
    Acc(Acc),
}

impl Arith {
    pub fn done(&self) -> bool {
        match self {
            Self::Reg(a) => a.done(),
            Self::Imm(irm) => irm.done(),
            Self::Acc(a) => a.done(),
        }
    }

    pub fn decode(&mut self, b: u8) {
        match self {
            Self::Reg(a) => a.decode(b),
            Self::Imm(irm) => irm.decode(b),
            Self::Acc(a) => a.decode(b),
        }
    }
}

impl Display for Arith {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Reg(a) => format!("{a}"),
            Self::Imm(irm) => format!("{irm}"),
            Self::Acc(a) => format!("{a}"),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    op: Op,
    w: SBF,
    d: SBF,
    data: Option<u8>,
    lo: Option<u8>,
    hi: Option<u8>,
    done: bool,
}

impl Reg {
    pub fn new(opcode: u8) -> Self {
        let op = ((opcode >> 3) & 7).into();
        let w: SBF = (opcode & 1).into();
        let d: SBF = ((opcode >> 1) & 1).into();
        Self {
            op,
            w,
            d,
            data: None,
            lo: None,
            hi: None,
            done: false,
        }
    }

    pub fn done(&self) -> bool {
        self.done
    }

    pub fn decode(&mut self, b: u8) {
        if self.data == None {
            self.data = Some(b);
            match self.r#mod() {
                MOD::Reg => {
                    self.done = true;
                }
                MOD::Mem => {
                    let md = self.decode_mem();
                    match md {
                        MemData::Direct(_) => {}
                        _ => {
                            self.done = true;
                        }
                    }
                }
                _ => {}
            }
        } else {
            match self.r#mod() {
                MOD::Reg => self.done = true,
                MOD::Mem => {
                    let md = self.decode_mem();
                    match md {
                        MemData::Direct(_) => {
                            if self.lo == None {
                                self.lo = Some(b);
                            } else if self.hi == None {
                                self.hi = Some(b);
                                self.done = true;
                            }
                        }
                        _ => {
                            self.done = true;
                        }
                    }
                }
                MOD::Byte => {
                    if self.lo == None {
                        self.lo = Some(b);
                        self.done = true;
                    }
                }
                MOD::Word => {
                    if self.lo == None {
                        self.lo = Some(b);
                    } else if self.hi == None {
                        self.hi = Some(b);
                        self.done = true;
                    }
                }
            }
        }
    }

    pub fn r#mod(&self) -> MOD {
        match self.data {
            None => panic!("data is None"),
            Some(d) => ((d >> 6) & 0x3).into(),
        }
    }

    pub fn rm(&self) -> RM {
        match self.data {
            None => panic!("data is None"),
            Some(d) => match self.r#mod() {
                MOD::Reg => {
                    let val = d & 0x7;
                    let rf = RegField::new(val, self.w.0);
                    RM::Reg(rf.reg)
                }
                MOD::Mem => RM::Mem(self.decode_mem()),
                MOD::Byte => {
                    let (rm, b) = self.decode_mem_byte();
                    RM::Byte(rm, b)
                }
                MOD::Word => {
                    let (rm, w) = self.decode_mem_word();
                    RM::Word(rm, w)
                }
            },
        }
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
    pub fn data(&self) -> u16 {
        let mut val: u16 = 0;
        if let Some(lo) = self.lo {
            val = lo as u16;
        }
        if let Some(hi) = self.hi {
            val = ((hi as u16) << 8) | val;
        }
        val
    }

    fn decode_mem(&self) -> MemData {
        let mut rm: MemData = (self.data.unwrap() & 0x7).into();
        if rm == MemData::Direct(None) {
            rm = MemData::Direct(Some(self.data()));
        }
        rm
    }

    fn decode_mem_byte(&self) -> (MemData, i16) {
        let rm: MemData = (self.data.unwrap() & 0x7).into();
        let b = match self.lo {
            None => 0,
            Some(v) => v,
        };
        (rm, (b as i8) as i16)
    }

    fn decode_mem_word(&self) -> (MemData, u16) {
        let rm: MemData = (self.data.unwrap() & 0x7).into();
        let lo = self.lo.unwrap();
        let hi = self.hi.unwrap();
        (rm, ((hi as u16) << 8) | (lo as u16))
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = match self.op {
            Op::Add => String::from("add "),
            Op::Sub => String::from("sub "),
            Op::Cmp => String::from("cmp "),
        };
        let reg = self.reg();
        let rm = self.rm();
        if self.d.0 {
            s.push_str(format!("{reg}, {rm}").as_str());
        } else {
            s.push_str(format!("{rm}, {reg}").as_str());
        }
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Imm {
    op: Option<Op>,
    w: SBF,
    s: SBF,
    data: Option<u8>,
    lo: Option<u8>,
    hi: Option<u8>,
    d0: Option<u8>,
    d1: Option<u8>,
    done: bool,
}

impl Imm {
    pub fn new(opcode: u8) -> Self {
        let w: SBF = (opcode & 1).into();
        let s: SBF = ((opcode >> 1) & 1).into();
        Self {
            op: None,
            w,
            s,
            data: None,
            lo: None,
            hi: None,
            d0: None,
            d1: None,
            done: false,
        }
    }

    pub fn done(&self) -> bool {
        self.done
    }

    pub fn decode(&mut self, b: u8) {
        if self.data == None {
            self.data = Some(b);
            self.op = Some(((b >> 3) & 7).into());
        } else {
            match self.r#mod() {
                MOD::Reg => {
                    if self.d0 == None {
                        self.d0 = Some(b);
                        if self.sw() {
                            self.done = true;
                        }
                    } else if self.sw() && self.d1 == None {
                        self.d1 = Some(b);
                        self.done = true
                    }
                }
                MOD::Mem => {
                    let md = self.decode_mem();
                    match md {
                        MemData::Direct(_) => {
                            if self.lo == None {
                                self.lo = Some(b);
                            } else if self.hi == None {
                                self.hi = Some(b);
                            } else if self.d0 == None {
                                self.d0 = Some(b);
                                if self.sw() {
                                    self.done = true;
                                }
                            } else if self.d1 == None {
                                self.d1 = Some(b);
                                self.done = true
                            }
                        }
                        _ => {
                            if self.d0 == None {
                                self.d0 = Some(b);
                                if self.sw() {
                                    self.done = true;
                                }
                            } else if self.sw() && self.d1 == None {
                                self.d1 = Some(b);

                                self.done = true
                            }
                        }
                    }
                }
                MOD::Byte => {
                    if self.lo == None {
                        self.lo = Some(b);
                    } else if self.d0 == None {
                        self.d0 = Some(b);
                        if !self.w.0 {
                            self.done = true;
                        }
                    } else if self.sw() && self.d1 == None {
                        self.d1 = Some(b);
                        self.done = true
                    }
                }
                MOD::Word => {
                    if self.lo == None {
                        self.lo = Some(b);
                    } else if self.hi == None {
                        self.hi = Some(b);
                    } else if self.d0 == None {
                        self.d0 = Some(b);
                        if self.sw() {
                            self.done = true;
                        }
                    } else if self.sw() && self.d1 == None {
                        self.d1 = Some(b);
                        self.done = true
                    }
                }
            }
        }
    }

    pub fn r#mod(&self) -> MOD {
        match self.data {
            None => panic!("data is None"),
            Some(d) => ((d >> 6) & 0x3).into(),
        }
    }

    pub fn sw(&self) -> bool {
        match self.op {
            None => false,
            Some(op) => match op {
                Op::Add | Op::Sub | Op::Cmp => !self.w.0 || self.s.0,
                //Op::Cmp => !self.w.0 || !self.s.0,
            },
        }
    }

    pub fn rm(&self) -> RM {
        match self.data {
            None => panic!("data is None"),
            Some(d) => match self.r#mod() {
                MOD::Reg => {
                    let val = d & 0x7;
                    let rf = RegField::new(val, self.w.0);
                    RM::Reg(rf.reg)
                }
                MOD::Mem => RM::Mem(self.decode_mem()),
                MOD::Byte => {
                    let (rm, b) = self.decode_mem_byte();
                    RM::Byte(rm, b)
                }
                MOD::Word => {
                    let (rm, w) = self.decode_mem_word();
                    RM::Word(rm, w)
                }
            },
        }
    }

    pub fn imm(&self) -> u16 {
        let mut val: u16 = 0;
        if let Some(lo) = self.d0 {
            val = lo as u16;
        }
        if self.sw()
            && let Some(hi) = self.d1
        {
            val = ((hi as u16) << 8) | val;
        }
        val
    }

    pub fn data(&self) -> u16 {
        let mut val: u16 = 0;
        if let Some(lo) = self.lo {
            val = lo as u16;
        }
        if let Some(hi) = self.hi {
            val = ((hi as u16) << 8) | val;
        }
        val
    }

    fn decode_mem(&self) -> MemData {
        let mut rm: MemData = (self.data.unwrap() & 0x7).into();
        if rm == MemData::Direct(None) {
            rm = MemData::Direct(Some(self.data()));
        }
        rm
    }

    fn decode_mem_byte(&self) -> (MemData, i16) {
        let rm: MemData = (self.data.unwrap() & 0x7).into();
        let b = match self.lo {
            None => 0,
            Some(v) => v,
        };
        (rm, (b as i8) as i16)
    }

    fn decode_mem_word(&self) -> (MemData, u16) {
        let rm: MemData = (self.data.unwrap() & 0x7).into();
        let lo = self.lo.unwrap();
        let hi = self.hi.unwrap();
        (rm, ((hi as u16) << 8) | (lo as u16))
    }
}

impl Display for Imm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = match self.op {
            Some(op) => match op {
                Op::Add => String::from("add "),
                Op::Sub => String::from("sub "),
                Op::Cmp => String::from("cmp "),
            },
            None => panic!("op is None"),
        };
        let rm = self.rm();
        match rm {
            RM::Byte(_, _) => s.push_str(format!("byte {rm}").as_str()),
            RM::Word(_, _) => s.push_str(format!("word {rm}").as_str()),
            RM::Mem(_) => s.push_str(format!("byte {rm}").as_str()),
            _ => s.push_str(format!("{rm}").as_str()),
        }
        let imm = self.imm();
        s.push_str(format!(", {imm}").as_str());
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Acc {
    op: Op,
    w: SBF,
    d0: Option<u8>,
    d1: Option<u8>,
    done: bool,
}

impl Acc {
    pub fn new(opcode: u8) -> Self {
        let w: SBF = (opcode & 1).into();
        Self {
            op: ((opcode >> 3) & 7).into(),
            w,
            d0: None,
            d1: None,
            done: false,
        }
    }

    pub fn done(&self) -> bool {
        self.done
    }

    pub fn decode(&mut self, b: u8) {
        if self.d0 == None {
            self.d0 = Some(b);
            if !self.w.0 {
                self.done = true;
            }
        } else if self.w.0 && self.d1 == None {
            self.d1 = Some(b);
            self.done = true
        }
    }

    pub fn imm(&self) -> u16 {
        let mut val: u16 = 0;
        if let Some(lo) = self.d0 {
            val = lo as u16;
        }
        if self.w.0
            && let Some(hi) = self.d1
        {
            ((hi as u16) << 8) | val
        } else {
            (((val as u8) as i8) as i16) as u16
        }
    }
}

impl Display for Acc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = match self.op {
            Op::Add => String::from("add "),
            Op::Sub => String::from("sub "),
            Op::Cmp => String::from("cmp "),
        };
        match self.w.0 {
            true => s.push_str("ax, "),
            false => s.push_str("al, "),
        }
        let imm = self.imm();
        match self.w.0 {
            true => s.push_str(format!("{imm}").as_str()),
            false => s.push_str(format!("{}", imm as i16).as_str()),
        }
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Cmp,
}

impl From<u8> for Op {
    fn from(v: u8) -> Self {
        match v {
            0 => Op::Add,
            0b101 => Op::Sub,
            0b111 => Op::Cmp,
            _ => panic!("invalid {}", v),
        }
    }
}

pub fn is_r_to_r(b: u8) -> bool {
    match (b >> 2) & 0b1111_11 {
        0 | 0b00_1010 | 0b00_1110 => true,
        _ => false,
    }
}

pub fn is_i_to_r(b: u8) -> bool {
    match (b >> 2) & 0b1111_11 {
        0b10_0000 => true,
        _ => false,
    }
}

pub fn is_i_to_a(b: u8) -> bool {
    match (b >> 1) & 0b1111_111 {
        0b000_0010 | 0b0010_110 | 0b0011_110 => true,
        _ => false,
    }
}
