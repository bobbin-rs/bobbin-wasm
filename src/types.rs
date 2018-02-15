use {Error, TypeValue, Value};
use opcode::*;

#[derive(Debug)]
pub struct Identifier<'a>(pub &'a [u8]);

#[derive(Debug)]
pub struct TypeIndex(pub u32);
#[derive(Debug)]
pub struct FuncIndex(pub u32);
#[derive(Debug)]
pub struct TableIndex(pub u32);
#[derive(Debug)]
pub struct MemIndex(pub u32);
#[derive(Debug)]
pub struct GlobalIndex(pub u32);
#[derive(Debug)]
pub struct LocalIndex(pub u32);
#[derive(Debug)]
pub struct LabelIndex(pub u32);

#[derive(Debug)]
pub enum ExternalIndex {
    Func(FuncIndex),
    Table(TableIndex),
    Mem(MemIndex),
    Global(GlobalIndex),
}

impl ExternalIndex {
    pub fn kind(&self) -> u8 {
        use ExternalIndex::*;
        match *self {
            Func(_) => 0x00,
            Table(_) => 0x01,
            Mem(_) => 0x02,
            Global(_) => 0x03,
        }
    }
    pub fn index(&self) -> u32 {
        use ExternalIndex::*;
        match *self {
            Func(FuncIndex(n)) => n,
            Table(TableIndex(n)) => n,
            Mem(MemIndex(n)) => n,
            Global(GlobalIndex(n)) => n,
        }        
    }
}

#[derive(Debug)]
pub struct Limits {
    pub flags: u32,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug)]
pub struct Initializer {
    pub opcode: u8,
    pub immediate: i32,
    pub end: u8,
}

impl Initializer {
    pub fn value(&self) -> Result<Value, Error> {
        match self.opcode {
            I32_CONST => Ok(Value(self.immediate)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sig<'a> {
    buf: &'a [u8],
}

impl<'a> Sig<'a> {
    pub fn new(buf: &'a [u8]) -> Sig<'a> {
        assert!(buf.len() == (1 + buf[0] as usize +1 +  buf[1 + buf[0] as usize] as usize), "Sig buffer too short");
        Sig { buf }
    }

    pub fn parameters(&self) -> SigIter {
        let len = self.buf[0];
        let buf = &self.buf[1..1+len as usize];
        let pos = 0;
        SigIter { buf, len, pos }
    }

    pub fn returns(&self) -> SigIter {
        let p_len = self.buf[0];
        let len = self.buf[1 + p_len as usize];
        let buf = &self.buf[(1 + p_len + 1) as usize .. (1 + p_len + 1 + len) as usize];
        let pos = 0;
        SigIter { buf, len, pos }        
    }
}

pub struct SigIter<'a> {
    buf: &'a [u8],
    len: u8,
    pos: u8,
}

impl<'a> Iterator for SigIter<'a> {
    type Item = TypeValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.len {
            let t = TypeValue::from(self.buf[self.pos as usize]);
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sig() {
        let buf = [2, I32 as u8, I32 as u8, 1, I64 as u8];
        let sig = Sig::new(&buf[..]);

        assert!(sig.parameters().count() == 2);
        assert!(sig.returns().count() == 1);

    }
}