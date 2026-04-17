use std::{collections::HashMap, fmt::Display};

pub struct JmpRegistry {
    current_label: u8,
    map: HashMap<i8, String>,
}

impl JmpRegistry {
    pub fn new() -> Self {
        JmpRegistry {
            current_label: 1,
            map: HashMap::new(),
        }
    }

    pub fn decrement_all(&mut self) {
        let keys: Vec<i8> = self.map.keys().map(|i| *i).collect();
        for k in keys {
            let val = self.map.remove(&k).unwrap();
            self.map.insert(k - 2, val);
        }
    }

    fn get_or_register(&mut self, addr: &i8) -> String {
        match self.map.get(addr) {
            Some(l) => l.clone(),
            None => {
                let next_label = format!("label_{}", self.current_label);
                self.current_label += 1;
                self.map.insert(*addr, next_label.clone());
                next_label
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum JMP {
    JE(String, i8),
    JNZ(String, i8),
    JL(String, i8),
    JLE(String, i8),
    JB(String, i8),
    JBE(String, i8),
    JP(String, i8),
    JO(String, i8),
    JS(String, i8),
    JNL(String, i8),
    JG(String, i8),
    JNB(String, i8),
    JA(String, i8),
    JNP(String, i8),
    JNO(String, i8),
    JNS(String, i8),
    LOOP(String, i8),
    LOOPZ(String, i8),
    LOOPNZ(String, i8),
    JCXZ(String, i8),
}

impl JMP {
    pub fn decode<'a, I>(b: u8, mut iter: I, jr: &mut JmpRegistry) -> Option<JMP>
    where
        I: Iterator<Item = &'a u8>,
    {
        let mut jmp: JMP = b.try_into().unwrap();
        let addr = *iter.next().unwrap_or(&0) as i8;
        let label = jr.get_or_register(&addr);
        match &mut jmp {
            JMP::JE(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JNZ(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JL(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JLE(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JB(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JBE(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JP(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JO(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JS(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JNL(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JG(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JNB(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JA(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::LOOP(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JNP(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JNO(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JNS(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::LOOPZ(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::LOOPNZ(a, o) => {
                *a = label;
                *o = addr;
            }
            JMP::JCXZ(a, o) => {
                *a = label;
                *o = addr;
            }
        }
        Some(jmp)
    }
}

impl TryFrom<u8> for JMP {
    type Error = String;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0b0111_0100 => Ok(JMP::JE(String::new(), 0)),
            0b0111_0101 => Ok(JMP::JNZ(String::new(), 0)),
            0b0111_1100 => Ok(JMP::JL(String::new(), 0)),
            0b0111_1110 => Ok(JMP::JLE(String::new(), 0)),
            0b0111_0010 => Ok(JMP::JB(String::new(), 0)),
            0b0111_0110 => Ok(JMP::JBE(String::new(), 0)),
            0b0111_1010 => Ok(JMP::JP(String::new(), 0)),
            0b0111_0000 => Ok(JMP::JO(String::new(), 0)),
            0b0111_1000 => Ok(JMP::JS(String::new(), 0)),
            0b0111_1101 => Ok(JMP::JNL(String::new(), 0)),
            0b0111_1111 => Ok(JMP::JG(String::new(), 0)),
            0b0111_0011 => Ok(JMP::JNB(String::new(), 0)),
            0b0111_0111 => Ok(JMP::JA(String::new(), 0)),
            0b1110_0010 => Ok(JMP::LOOP(String::new(), 0)),
            0b0111_1011 => Ok(JMP::JNP(String::new(), 0)),
            0b0111_0001 => Ok(JMP::JNO(String::new(), 0)),
            0b0111_1001 => Ok(JMP::JNS(String::new(), 0)),
            0b1110_0001 => Ok(JMP::LOOPZ(String::new(), 0)),
            0b1110_0000 => Ok(JMP::LOOPNZ(String::new(), 0)),
            0b1110_0011 => Ok(JMP::JCXZ(String::new(), 0)),
            _ => Err(format!("invalid JMP: {val}")),
        }
    }
}

impl Display for JMP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            JMP::JE(l, o) => format!("je {l}; {o}"),
            JMP::JNZ(l, o) => format!("jne {l}; {o}"),
            JMP::JL(l, o) => format!("jl {l}; {o}"),
            JMP::JLE(l, o) => format!("jle {l}; {o}"),
            JMP::JB(l, o) => format!("jb {l}; {o}"),
            JMP::JBE(l, o) => format!("jbe {l}; {o}"),
            JMP::JP(l, o) => format!("jp {l}; {o}"),
            JMP::JO(l, o) => format!("jo {l}; {o}"),
            JMP::JS(l, o) => format!("js {l}; {o}"),
            JMP::JNL(l, o) => format!("jnl {l}; {o}"),
            JMP::JG(l, o) => format!("jg {l}; {o}"),
            JMP::JNB(l, o) => format!("jnb {l}; {o}"),
            JMP::JA(l, o) => format!("ja {l}; {o}"),
            JMP::JNP(l, o) => format!("jnp {l}; {o}"),
            JMP::JNO(l, o) => format!("jno {l}; {o}"),
            JMP::JNS(l, o) => format!("jns {l}; {o}"),
            JMP::LOOP(l, o) => format!("loop {l}; {o}"),
            JMP::LOOPZ(l, o) => format!("loopz {l}; {o}"),
            JMP::LOOPNZ(l, o) => format!("loopnz {l}; {o}"),
            JMP::JCXZ(l, o) => format!("jcxz {l}; {o}"),
        };
        write!(f, "{s}")
    }
}
