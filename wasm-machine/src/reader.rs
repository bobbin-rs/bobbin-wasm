use Error;
use wasm_leb128::*;

use byteorder::{ByteOrder, LittleEndian};

pub type ReaderResult<T> = Result<T, Error>;

// pub trait Reader {
//     fn read_opcode(&mut self) -> ReaderResult<u8>;
//     fn read_immediate_i32(&mut self) -> ReaderResult<i32>;
//     fn read_immediate_u32(&mut self) -> ReaderResult<u32>;
//     fn read_immediate_u8(&mut self) -> ReaderResult<u8>;
//     fn read_block_type(&mut self) -> ReaderResult<BlockType>;
// }

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf: buf, pos: 0 }
    }
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    #[inline]
    fn read<T, F: FnOnce(&[u8])->T>(&mut self, size: usize, f: F) -> ReaderResult<T> {
        if self.pos() + size > self.buf.len() { return Err(Error::End) }
        let v = f(&self.buf[self.pos..self.pos+size]);
        self.pos += size;
        Ok(v)        
    }

    #[inline]
    pub fn read_u8(&mut self) -> ReaderResult<u8> { 
        self.read(1, |buf| buf[0])
    }

    #[inline]
    pub fn read_u16(&mut self) -> ReaderResult<u16> { 
        self.read(2, LittleEndian::read_u16)
    }

    #[inline]
    pub fn read_u32(&mut self) -> ReaderResult<u32> { 
        self.read(4, LittleEndian::read_u32)
    }

    #[inline]
    pub fn read_u64(&mut self) -> ReaderResult<u64> { 
        self.read(8, LittleEndian::read_u64)
    }

    #[inline]
    pub fn read_i8(&mut self) -> ReaderResult<i8> { 
        self.read(1, |buf| buf[0] as i8)
    }

    #[inline]
    pub fn read_i16(&mut self) -> ReaderResult<i16> { 
        self.read(2, LittleEndian::read_i16)
    }

    #[inline]
    pub fn read_i32(&mut self) -> ReaderResult<i32> { 
        self.read(4, LittleEndian::read_i32)
    }

    #[inline]
    pub fn read_i64(&mut self) -> ReaderResult<i64> { 
        self.read(8, LittleEndian::read_i64)
    }

    #[inline]
    pub fn read_f32(&mut self) -> ReaderResult<f32> { 
        self.read(4, LittleEndian::read_f32)
    }

    #[inline]
    pub fn read_f64(&mut self) -> ReaderResult<f64> { 
        self.read(8, LittleEndian::read_f64)
    }

    #[inline]
    pub fn read_var_u1(&mut self) -> ReaderResult<u8> { 
        let (v, n) = read_u1(&self.buf[self.pos..])?;
        self.pos += n;
        Ok(if v { 1 } else { 0 })
    }

    #[inline]
    pub fn read_var_u7(&mut self) -> ReaderResult<u8> { 
        let (v, n) = read_u7(&self.buf[self.pos..])?;
        self.pos += n;
        Ok(v)
    }

    #[inline]
    pub fn read_var_u32(&mut self) -> ReaderResult<u32> { 
        let (v, n) = read_u32(&self.buf[self.pos..])?;
        self.pos += n;
        Ok(v)
    }
    
    #[inline]
    pub fn read_var_i7(&mut self) -> ReaderResult<i8> { 
        let (v, n) = read_i7(&self.buf[self.pos..])?;
        self.pos += n;
        Ok(v)
    }

    #[inline]
    pub fn read_var_i32(&mut self) -> ReaderResult<i32> { 
        let (v, n) = read_i32(&self.buf[self.pos..])?;
        self.pos += n;
        Ok(v)
    }
}