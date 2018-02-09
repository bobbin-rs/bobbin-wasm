use wasm_leb128::{write_u1, write_u7, write_i7, write_u32, write_i32};
use byteorder::{ByteOrder, LittleEndian};
use core::ops::Deref;
use core::slice;
use reader::Reader;
use Error;

pub type WriteResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Writer<'a> {
    pub(crate) buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Writer { buf: buf, pos: 0 }
    }

    pub fn cap(&self) -> usize {
        self.buf.len()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn write_u8(&mut self, value: u8) -> WriteResult<()> {
        if self.pos + 1 >= self.buf.len() { return Err(Error::End) }
        self.buf[self.pos] = value;
        self.pos += 1;        
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> WriteResult<()> {
        if self.pos + 4 >= self.buf.len() { return Err(Error::End) }
        LittleEndian::write_u32(&mut self.buf[self.pos..], value);
        self.pos += 4;
        Ok(())
    }

    pub fn write_u32_at(&mut self, value: u32, offset: usize) -> WriteResult<()> {
        if offset + 4 > self.buf.len() { return Err(Error::End) }
        if offset + 4 > self.pos { return Err(Error::End) }
        LittleEndian::write_u32(&mut self.buf[offset..], value);
        Ok(())
    }

    pub fn write_i8(&mut self, value: i8) -> WriteResult<()> {
        if self.pos + 1 >= self.buf.len() { return Err(Error::End) }
        self.buf[self.pos] = value as u8;
        self.pos += 1;        
        Ok(())
    }

    pub fn write_i32(&mut self, value: i32) -> WriteResult<()> {
        if self.pos + 4 >= self.buf.len() { return Err(Error::End) }
        LittleEndian::write_i32(&mut self.buf[self.pos..], value);
        self.pos += 4;
        Ok(())
    }

    pub fn write_var_u1(&mut self, value: bool) -> WriteResult<()> {
        self.pos += write_u1(&mut self.buf[self.pos..], value).unwrap();        
        Ok(())
    }

    pub fn write_var_u7(&mut self, value: u8) -> WriteResult<()> {
        self.pos += write_u7(&mut self.buf[self.pos..], value).unwrap();        
        Ok(())
    }

    pub fn write_var_i7(&mut self, value: i8) -> WriteResult<()> {
        self.pos += write_i7(&mut self.buf[self.pos..], value).unwrap();        
        Ok(())
    }

    pub fn write_var_u32(&mut self, value: u32) -> WriteResult<()> {
        self.pos += write_u32(&mut self.buf[self.pos..], value).unwrap();        
        Ok(())
    }

    pub fn write_var_i32(&mut self, value: i32) -> WriteResult<()> {
        self.pos += write_i32(&mut self.buf[self.pos..], value).unwrap();        
        Ok(())
    }

    pub fn split(&mut self) -> &'a [u8] {
        unsafe {
            // First Half
            let a_ptr = self.buf.as_ptr();
            let a_len = self.pos;

            // Second Half
            let b_ptr = self.buf.as_mut_ptr().offset(a_len as isize);
            let b_len = self.buf.len() - a_len;

            // Update Writer
            self.buf = slice::from_raw_parts_mut(b_ptr, b_len);
            self.pos = 0;

            // Return New Reader
            slice::from_raw_parts(a_ptr, a_len)
        }        
    }

    pub fn split_reader(&mut self) -> Reader<'a> {
        Reader::new(self.split())
    }
}

impl<'a> Into<Reader<'a>> for Writer<'a> {
    fn into(self) -> Reader<'a> {
        Reader::new(&self.buf[..self.pos])
    }
}


impl<'a> Deref for Writer<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buf[..self.pos]
    }
}
