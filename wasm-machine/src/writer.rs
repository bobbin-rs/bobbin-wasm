use wasm_leb128::{write_u1, write_u7, write_i7, write_u32, write_i32};
use byteorder::{ByteOrder, LittleEndian};
use core::ops::Deref;

#[derive(Debug)]
pub struct Writer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Writer { buf: buf, pos: 0 }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buf[self.pos] = value;
        self.pos += 1;        
    }

    pub fn write_u32(&mut self, value: u32) {
        LittleEndian::write_u32(&mut self.buf[self.pos..], value);
        self.pos += 4;
    }

    pub fn write_var_u1(&mut self, value: bool) {
        self.pos += write_u1(&mut self.buf[self.pos..], value).unwrap();        
    }

    pub fn write_var_u7(&mut self, value: u8) {
        self.pos += write_u7(&mut self.buf[self.pos..], value).unwrap();        
    }

    pub fn write_var_i7(&mut self, value: i8) {
        self.pos += write_i7(&mut self.buf[self.pos..], value).unwrap();        
    }

    pub fn write_var_u32(&mut self, value: u32) {
        self.pos += write_u32(&mut self.buf[self.pos..], value).unwrap();        
    }

    pub fn write_var_i32(&mut self, value: i32) {
        self.pos += write_i32(&mut self.buf[self.pos..], value).unwrap();        
    }

}


impl<'a> Deref for Writer<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buf[..self.pos]
    }
}