#![allow(unused_imports, dead_code)]

extern crate wasm_leb128;
extern crate byteorder;

use wasm_leb128::{read_u7, read_i7, read_u32};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub enum Error {
    BufferTooShort,
    Leb128Error(wasm_leb128::Error),
    Unspecified,
}

impl From<wasm_leb128::Error> for Error {
    fn from(other: wasm_leb128::Error) -> Error {
        Error::Leb128Error(other)
    }
}

pub struct Buf<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Buf<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Buf { buf: buf, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn slice(&self, size: usize) -> Result<&'a [u8], Error> {
        if size < self.remaining() { 
            Ok(&self.buf[self.pos..self.pos+4])
        } else {
            Err(Error::BufferTooShort)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let v = LittleEndian::read_u32(try!(self.slice(4)));
        self.pos += 4;
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
    fn test_Buf() {
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
