use byteorder::{ByteOrder, LittleEndian};
use core::slice;

pub struct Cursor<'a> {
    buf: &'a [u8],
}

impl<'a> Cursor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Cursor { buf }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn slice(&mut self, len: usize) -> &'a [u8] {
        let v = &self.buf[..len];
        self.buf = &self.buf[len..];
        v
    }

    pub fn slice_u32(&mut self, len: usize) -> &'a [u32] {
        let ptr = self.buf.as_ptr() as *const u32;
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    pub fn slice_identifier(&mut self) -> &'a [u8] {
        let len = self.read_u32();
        self.slice(len as usize)
    }

    pub fn read_u8(&mut self) -> u8 {
        let v = self.buf[0];
        self.buf = &self.buf[1..];
        v
    }

    pub fn read_u32(&mut self) -> u32 {
        let v = LittleEndian::read_u32(self.buf);
        self.buf = &self.buf[4..];
        v
    }

    pub fn read_i8(&mut self) -> i8 {
        let v = self.buf[0] as i8;
        self.buf = &self.buf[1..];
        v
    }    

    pub fn read_i32(&mut self) -> i32 {
        let v = LittleEndian::read_i32(self.buf);
        self.buf = &self.buf[4..];
        v
    }
}