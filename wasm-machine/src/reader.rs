use Error;
use ops::*;
use wasm_leb128::*;

use byteorder::{ByteOrder, LittleEndian};

pub type ReaderResult<T> = Result<T, Error>;

pub trait Reader {
    fn read_opcode(&mut self) -> ReaderResult<u8>;
    fn read_immediate_i32(&mut self) -> ReaderResult<i32>;
    fn read_immediate_u32(&mut self) -> ReaderResult<u32>;
    fn read_immediate_u8(&mut self) -> ReaderResult<u8>;
    fn read_block_type(&mut self) -> ReaderResult<BlockType>;
}

pub struct BinaryReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    #[inline]
    pub fn pos(&mut self) -> usize {
        self.pos
    }

    #[inline]
    pub fn read_u8(&mut self) -> ReaderResult<u8> { 
        if self.pos() + 1 >= self.buf.len() { return Err(Error::End) }
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }

    #[inline]
    pub fn read_u16(&mut self) -> ReaderResult<u16> { 
        if self.pos() + 2 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_u16(&self.buf[self.pos..]);
        self.pos += 2;
        Ok(v)
    }

    #[inline]
    pub fn read_u32(&mut self) -> ReaderResult<u32> { 
        if self.pos() + 4 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_u32(&self.buf[self.pos..]);
        self.pos += 4;
        Ok(v)
    }

    #[inline]
    pub fn read_u64(&mut self) -> ReaderResult<u64> { 
        if self.pos() + 8 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_u64(&self.buf[self.pos..]);
        self.pos += 8;
        Ok(v)
    }

    #[inline]
    pub fn read_i8(&mut self) -> ReaderResult<i8> { 
        if self.pos() + 1 >= self.buf.len() { return Err(Error::End) }
        let v = self.buf[self.pos] as i8;
        self.pos += 1;
        Ok(v)
    }

    #[inline]
    pub fn read_i16(&mut self) -> ReaderResult<i16> { 
        if self.pos() + 2 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_i16(&self.buf[self.pos..]);
        self.pos += 2;
        Ok(v)
    }

    #[inline]
    pub fn read_i32(&mut self) -> ReaderResult<i32> { 
        if self.pos() + 4 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_i32(&self.buf[self.pos..]);
        self.pos += 4;
        Ok(v)
    }

    #[inline]
    pub fn read_i64(&mut self) -> ReaderResult<i64> { 
        if self.pos() + 8 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_i64(&self.buf[self.pos..]);
        self.pos += 8;
        Ok(v)
    }

    #[inline]
    pub fn read_f32(&mut self) -> ReaderResult<f32> { 
        if self.pos() + 4 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_f32(&self.buf[self.pos..]);
        self.pos += 4;
        Ok(v)        
    }

    #[inline]
    pub fn read_f64(&mut self) -> ReaderResult<f64> { 
        if self.pos() + 8 >= self.buf.len() { return Err(Error::End) }
        let v = LittleEndian::read_f64(&self.buf[self.pos..]);
        self.pos += 8;
        Ok(v)        
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

impl<'a> Reader for BinaryReader<'a> {
    #[inline]
    fn read_opcode(&mut self) -> ReaderResult<u8> { 
        self.read_u8()
    }

    #[inline]
    fn read_immediate_i32(&mut self) -> ReaderResult<i32> { 
        self.read_i32()
    }

    #[inline]
    fn read_immediate_u32(&mut self) -> ReaderResult<u32> { 
        self.read_u32()
    }

    #[inline]
    fn read_immediate_u8(&mut self) -> ReaderResult<u8> {
        self.read_u8()
    }

    #[inline]
    fn read_block_type(&mut self) -> ReaderResult<BlockType> { 
        match self.read_u8()? {
            0x7f => Ok(BlockType::I32),
            0x7e => Ok(BlockType::I64),
            0x7d => Ok(BlockType::F32),
            0x7c => Ok(BlockType::F64),
            0x40 => Ok(BlockType::Void),
            _ => Err(Error::InvalidBlockType),
        }
    }
}
