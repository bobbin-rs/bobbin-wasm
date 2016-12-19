use wasm_leb128::{read_u7, read_i7, read_u32};
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

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let v = LittleEndian::read_u32(try!(self.slice(4)));
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