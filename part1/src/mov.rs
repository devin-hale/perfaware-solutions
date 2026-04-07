use crate::{MOD, MemData, REG, RM, RegField, SBF};
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum MOV {
    Reg(MovReg),
    ImmReg(ImmReg),
}

impl MOV {
    pub fn decode(&mut self, b: u8) {
        match self {
            Self::Reg(mr) => mr.decode(b),
            Self::ImmReg(ir) => ir.decode(b),
        }
    }

    pub fn done(&self) -> bool {
        match self {
            Self::Reg(mr) => mr.done(),
            Self::ImmReg(ir) => ir.done(),
        }
    }
}

impl Display for MOV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Reg(mr) => format!("{mr}"),
            Self::ImmReg(ir) => format!("{ir}"),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MovReg {
    d: SBF,
    w: SBF,
    data: Option<u8>,
    dlo: Option<u8>,
    dhi: Option<u8>,
    done: bool,
}

impl MovReg {
    pub fn new(opcode: u8) -> MovReg {
        Self {
            d: ((opcode >> 1) & 1).into(),
            w: (opcode & 1).into(),
            done: false,
            data: None,
            dlo: None,
            dhi: None,
        }
    }

    pub fn d(&self) -> bool {
        self.d.0
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
                            if self.dlo == None {
                                self.dlo = Some(b);
                            } else if self.dhi == None {
                                self.dhi = Some(b);
                                self.done = true;
                            }
                        }
                        _ => {
                            self.done = true;
                        }
                    }
                }
                MOD::Byte => {
                    if self.dlo == None {
                        self.dlo = Some(b);
                        self.done = true;
                    }
                }
                MOD::Word => {
                    if self.dlo == None {
                        self.dlo = Some(b);
                    } else if self.dhi == None {
                        self.dhi = Some(b);
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
        if let Some(lo) = self.dlo {
            val = lo as u16;
        }
        if let Some(hi) = self.dhi {
            val = ((hi as u16) << 8) | val;
        }
        val
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

    fn decode_mem(&self) -> MemData {
        let mut rm: MemData = (self.data.unwrap() & 0x7).into();
        if rm == MemData::Direct(None) {
            rm = MemData::Direct(Some(self.data()));
        }
        rm
    }

    fn decode_mem_byte(&self) -> (MemData, u8) {
        let rm: MemData = (self.data.unwrap() & 0x7).into();
        let b = self.dlo.unwrap();
        (rm, b)
    }

    fn decode_mem_word(&self) -> (MemData, u16) {
        let rm: MemData = (self.data.unwrap() & 0x7).into();
        let lo = self.dlo.unwrap();
        let hi = self.dhi.unwrap();
        (rm, ((hi as u16) << 8) | (lo as u16))
    }
}

impl Display for MovReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = String::from("mov ");
        if !self.d() {
            i.push_str(format!("{}, {}", self.rm(), self.reg()).as_str());
        } else {
            i.push_str(format!("{}, {}", self.reg(), self.rm()).as_str());
        }
        write!(f, "{i}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImmReg {
    w: SBF,
    reg: REG,
    d0: Option<u8>,
    d1: Option<u8>,
    done: bool,
}

impl ImmReg {
    pub fn new(opcode: u8) -> Self {
        let w: SBF = ((opcode >> 3) & 1).into();
        let reg_val = opcode & 0x7;
        let rf = RegField::new(reg_val, w.0);
        Self {
            w,
            reg: rf.reg,
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
        } else if self.d1 == None && self.w.0 {
            self.d1 = Some(b);
            self.done = true;
        }
    }

    pub fn data(&self) -> u16 {
        let mut val = 0 as u16;
        if let Some(d) = self.d0 {
            val += d as u16;
        }
        if let Some(d) = self.d1 {
            val = ((d as u16) << 8) | val;
        }
        val
    }
}

impl Display for ImmReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("mov ");
        s.push_str(format!("{}, {}", self.reg, self.data()).as_str());
        write!(f, "{s}")
    }
}
