use wasm_leb128::{read_u1, read_u7, read_i7, read_u32};
use byteorder::{ByteOrder, LittleEndian};

use Error;

pub struct Buf<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Buf<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Buf { buf: buf, pos: 0 }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }    

    pub fn slice(&mut self, size: usize) -> Result<&'a [u8], Error> {
        if size <= self.remaining() { 
            let p = self.pos;
            self.pos += size;
            Ok(&self.buf[p..self.pos])
        } else {
            Err(Error::BufferTooShort)
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let v = LittleEndian::read_u32(try!(self.slice(4)));
        Ok(v)
    }

    pub fn read_var_u1(&mut self) -> Result<bool, Error> {
        let (v, n) = try!(read_u1(&self.buf[self.pos..]));
        self.pos += n;
        Ok(v)
    }

    pub fn read_var_u7(&mut self) -> Result<u8, Error> {
        let (v, n) = try!(read_u7(&self.buf[self.pos..]));
        self.pos += n;
        Ok(v)
    }

    pub fn read_var_i7(&mut self) -> Result<i8, Error> {
        let (v, n) = try!(read_i7(&self.buf[self.pos..]));
        self.pos += n;
        Ok(v)
    }    

    pub fn read_var_u32(&mut self) -> Result<u32, Error> {
        let (v, n) = try!(read_u32(&self.buf[self.pos..]));
        self.pos += n;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../testdata/basic.wasm");

    #[test]
    fn test_buf() {
        let mut r = Buf::new(BASIC);
        // MagicNumber
        assert_eq!(r.read_u32().unwrap(), 0x6d736100);
        // Version
        assert_eq!(r.read_u32().unwrap(), 0xd);
        
        // Section 

        // Section Id
        assert_eq!(r.read_var_u7().unwrap(), 1);
        // Section Payload Len
        assert_eq!(r.read_var_u32().unwrap(), 7);
        // Type Count
        assert_eq!(r.read_var_u32().unwrap(), 1);
        // Form: Func
        assert_eq!(r.read_var_i7().unwrap(), -0x20);
        // Parameter Count
        assert_eq!(r.read_var_u32().unwrap(), 2);
        // Parameter 1 Type
        assert_eq!(r.read_var_i7().unwrap(), -0x01);
        // Parameter 2 Type
        assert_eq!(r.read_var_i7().unwrap(), -0x01);
        // Result Count
        assert_eq!(r.read_var_u32().unwrap(), 1);
        // Result 1 Type
        assert_eq!(r.read_var_i7().unwrap(), -0x01);
    }
}
