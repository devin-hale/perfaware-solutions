use std::{collections::HashMap, fmt::Display, iter::Enumerate};

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

    pub fn take(&mut self, addr: &i8) -> String {
        let label = self.map.remove(addr).unwrap().clone();

        let keys = self.keys();
        for k in keys {
            let l = self.map.remove(&k).unwrap();
            if k > *addr {}
        }
        label
    }

    fn keys(&self) -> Vec<i8> {
        self.map.keys().map(|i| *i).collect()
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

#[derive(Debug, Clone, Copy)]
pub enum JMPType {
    JE,
    JNZ,
    JL,
    JLE,
    JB,
    JBE,
    JP,
    JO,
    JS,
    JNL,
    JG,
    JNB,
    JA,
    JNP,
    JNO,
    JNS,
    LOOP,
    LOOPZ,
    LOOPNZ,
    JCXZ,
}

#[derive(Debug, Clone)]
pub struct JMP {
    jmp_type: JMPType,
    label: String,
    offset: i8,
}

impl JMP {
    pub fn decode(
        b: u8,
        iter: &mut Enumerate<std::slice::Iter<u8>>,
        jr: &mut JmpRegistry,
    ) -> Option<JMP> {
        let jmp_type: JMPType = b.try_into().unwrap();

        let (next_i, offset) = iter.next().unwrap() as (usize, &u8);
        let offset = *offset as i8;
        let absolute = next_i as i8 + offset;
        let label = jr.get_or_register(&absolute);

        Some(JMP {
            jmp_type,
            label,
            offset,
        })
    }

    pub fn to_bytes(&self) -> Vec<String> {
        vec![self.jmp_type.into(), self.label.clone()]
    }
}

impl TryFrom<u8> for JMPType {
    type Error = String;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0b0111_0100 => Ok(JMPType::JE),
            0b0111_0101 => Ok(JMPType::JNZ),
            0b0111_1100 => Ok(JMPType::JL),
            0b0111_1110 => Ok(JMPType::JLE),
            0b0111_0010 => Ok(JMPType::JB),
            0b0111_0110 => Ok(JMPType::JBE),
            0b0111_1010 => Ok(JMPType::JP),
            0b0111_0000 => Ok(JMPType::JO),
            0b0111_1000 => Ok(JMPType::JS),
            0b0111_1101 => Ok(JMPType::JNL),
            0b0111_1111 => Ok(JMPType::JG),
            0b0111_0011 => Ok(JMPType::JNB),
            0b0111_0111 => Ok(JMPType::JA),
            0b1110_0010 => Ok(JMPType::LOOP),
            0b0111_1011 => Ok(JMPType::JNP),
            0b0111_0001 => Ok(JMPType::JNO),
            0b0111_1001 => Ok(JMPType::JNS),
            0b1110_0001 => Ok(JMPType::LOOPZ),
            0b1110_0000 => Ok(JMPType::LOOPNZ),
            0b1110_0011 => Ok(JMPType::JCXZ),
            _ => Err(format!("invalid JMPType: {val}")),
        }
    }
}

impl From<JMPType> for String {
    fn from(jt: JMPType) -> Self {
        match jt {
            JMPType::JE => "je".to_string(),
            JMPType::JNZ => "jne".to_string(),
            JMPType::JL => "jl".to_string(),
            JMPType::JLE => "jle".to_string(),
            JMPType::JB => "jb".to_string(),
            JMPType::JBE => "jbe".to_string(),
            JMPType::JP => "jp".to_string(),
            JMPType::JO => "jo".to_string(),
            JMPType::JS => "js".to_string(),
            JMPType::JNL => "jnl".to_string(),
            JMPType::JG => "jg".to_string(),
            JMPType::JNB => "jnb".to_string(),
            JMPType::JA => "ja".to_string(),
            JMPType::JNP => "jnp".to_string(),
            JMPType::JNO => "jno".to_string(),
            JMPType::JNS => "jns".to_string(),
            JMPType::LOOP => "loop".to_string(),
            JMPType::LOOPZ => "loopz".to_string(),
            JMPType::LOOPNZ => "loopnz".to_string(),
            JMPType::JCXZ => "jcxz".to_string(),
        }
    }
}

impl From<JMP> for String {
    fn from(j: JMP) -> Self {
        let mut s = j.to_bytes().join(" ");
        s.push_str(format!("; {}", j.offset).as_str());
        s
    }
}

impl Display for JMP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.to_bytes().join(" ");
        s.push_str(format!("; {}", self.offset).as_str());
        write!(f, "{s}")
    }
}
