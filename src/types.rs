use {Error, TypeValue, Value};
use opcode::*;
use cursor::Cursor;
use core::{mem, slice};

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

pub const MAX_PARAMETERS: usize = 16;
pub const MAX_RETURNS: usize = 1;

#[derive(Debug, PartialEq, Eq)]
pub struct Sig<'a> {
    buf: &'a [u8],    
}

impl<'a> Sig<'a> {
    fn is_valid_type(v: u8) -> bool {
        match v as i8 {
            -0x01 | -0x02 |-0x03 | -0x04 => true,
            _ => false,
        }
    }

    pub fn new(buf: &mut Cursor<'a>) -> Sig<'a> {
        let mut orig = buf.clone();
        let p_num = buf.read_var_i32();
        
        for _ in 0..p_num {
            buf.read_var_i7();
        }
        let r_num = buf.read_var_i32();
        
        for _ in 0..r_num {
            buf.read_var_i7();
        }
        let len = orig.len() - buf.len();
        
        let buf = orig.slice(len);        
        Sig { buf }
    }

    pub fn read_slice(buf: &[u8]) -> Option<(Sig, &[u8])> {
        debug_assert!(mem::size_of::<TypeValue>() == mem::size_of::<u8>());
        
        // Check for appropriate length

        if buf.len() < 1 { return None }

        let p_num = buf[0] as usize;

        // Check parameter count
        if p_num > MAX_PARAMETERS { return None }

        let p_len = 1 + p_num;

        if buf.len() < p_len + 1 { return None }

        let r_num = buf[p_len] as usize;

        // Check return count
        if r_num > MAX_RETURNS { return None }
        
        let s_len = p_len + 1 + r_num;

        if buf.len() < s_len { return None }

        // Check that type is one of I32, I64, F32, F64

        for i in 1..p_len {
            if !Self::is_valid_type(buf[i]) { return None }
        }

        for i in p_len+1..s_len {
            if !Self::is_valid_type(buf[i]) { return None }
        }

        Some((Sig { buf: &buf[..s_len] }, &buf[s_len..]))
    }    

    pub fn buf(&self) -> &'a [u8] {
        self.buf
    }

    pub fn parameters(&self) -> &'a [TypeValue] {
        let len = self.buf[0] as usize;        
        let buf = &self.buf[1..1+len];        
        unsafe { slice::from_raw_parts(buf.as_ptr() as *const TypeValue, buf.len())}
    }

    pub fn returns(&self) -> &'a [TypeValue] {
        let p_num = self.buf[0] as usize;        
        let p_len = 1 + p_num;        
        let r_num = self.buf[p_len] as usize;        
        let buf = &self.buf[p_len+1..p_len+1+r_num];
        unsafe { slice::from_raw_parts(buf.as_ptr() as *const TypeValue, buf.len())}
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
    fn test_read_sig() {
        let (sig, buf) = Sig::read_slice(&[0, 0]).unwrap();
        assert_eq!(sig.parameters(), &[]);
        assert_eq!(sig.returns(), &[]);
        assert_eq!(buf, &[]);

        let (sig, buf) = Sig::read_slice(&[1, I32 as u8, 0]).unwrap();
        assert_eq!(sig.parameters(), &[TypeValue::I32]);
        assert_eq!(sig.returns(), &[]);
        assert_eq!(buf, &[]);        

        let (sig, buf) = Sig::read_slice(&[0, 1, I32 as u8]).unwrap();
        assert_eq!(sig.parameters(), &[]);
        assert_eq!(sig.returns(), &[TypeValue::I32]);
        assert_eq!(buf, &[]);        


        let (sig, buf) = Sig::read_slice(&[2, I32 as u8, I32 as u8, 1, I64 as u8]).unwrap();
        assert_eq!(sig.parameters(), &[TypeValue::I32, TypeValue::I32]);
        assert_eq!(sig.returns(), &[TypeValue::I64]);
        assert_eq!(buf, &[]);
    }

    #[test]
    fn test_read_invalid_sig() {
        // empty
        assert!(Sig::read_slice(&[]).is_none());
        // too short
        assert!(Sig::read_slice(&[0]).is_none());
        // parameters too short
        assert!(Sig::read_slice(&[2, I32 as u8, 1, I64 as u8]).is_none());
        // parameters too long
        assert!(Sig::read_slice(&[2, I32 as u8, I32 as u8, I32 as u8, 1, I64 as u8]).is_none());
        // returns too short
        assert!(Sig::read_slice(&[2, I32 as u8, I32 as u8, 1]).is_none());
    }    
}