use parser::Error;
use parser::util::*;
use byteorder::{ByteOrder, LittleEndian};
pub use fallible_iterator::FallibleIterator;

use core::str;
use core::mem;
use core::marker::PhantomData;

pub struct Reader<'a> {
    buf: &'a [u8]
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn into_slice(self) -> &'a [u8] {
        self.buf
    }

    pub fn rest(&self) -> &'a [u8] {
        self.buf
    }

    pub fn offset_from(&self, base: &Reader<'a>) -> usize {
        let base_ptr = base.buf.as_ptr() as *const u8 as usize;
        let ptr = self.buf.as_ptr() as *const u8 as usize;
        debug_assert!(base_ptr <= ptr);
        debug_assert!(ptr + self.buf.len() <= base_ptr + base.buf.len());
        ptr - base_ptr        
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], Error> {
        if self.len() < len {
            Err(Error::UnexpectedEof)
        } else {
            let val = &self.buf[..len];
            self.buf = &self.buf[len..];
            Ok(val)
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let slice = self.read_slice(1)?;
        Ok(slice[0])
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let slice = self.read_slice(4)?;
        Ok(
            (slice[0] as u32) << 0 |
            (slice[1] as u32) << 8 |
            (slice[2] as u32) << 16 |
            (slice[3] as u32) << 24
        )
    }

    pub fn read_var_u0(&mut self) -> Result<(), Error> {
        match self.read_u8()? {
            0 => Ok(()),
            _ => Err(Error::InvalidU0),
        }
    }

    pub fn read_var_u1(&mut self) -> Result<bool, Error> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::InvalidU1),
        }
    }

    pub fn read_var_u7(&mut self) -> Result<u8, Error> {
        let byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            Ok(byte)
        } else {
            Err(Error::InvalidU7)
        }
    }   

    pub fn read_var_u32(&mut self) -> Result<u32, Error> {
        let mut byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            Ok(byte.into())
        } else {
            let mut value = (byte & 0x7f) as u32;
            let mut shift = 7;            
            loop {
                byte = self.read_u8()?;
                value |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 { break }
                shift += 7;
                if shift > 31 { 
                    return Err(Error::InvalidU32)
                }
            }
            Ok(value)
        }    
    }
    pub fn read_var_u64(&mut self) -> Result<u64, Error> {
        let mut byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            Ok(byte as u64)
        } else {
            let mut value = (byte & 0x7f) as u64;
            let mut shift = 7;            
            loop {
                byte = self.read_u8()?;
                value |= ((byte & 0x7f) as u64) << shift;
                if byte & 0x80 == 0 { break }
                shift += 7;
                if shift > 63 { return Err(Error::InvalidU64) }
            }
            Ok(value)
        }    
    }    

    #[inline]
    pub fn read_var_i32(&mut self) -> Result<i32, Error> { 
        let mut byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                byte |= 0x80;
            }            
            Ok(byte as i8 as i32)
        } else {
            let mut value = (byte & 0x7f) as u32;
            let mut shift = 7;
            loop {
                byte = self.read_u8()?;
                value |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 {
                    if byte & 0x40 != 0 {
                        value |= 0xffff_ff80 << shift;
                    }
                    break 
                }
                shift += 7;
                if shift > 31 { return Err(Error::InvalidI32) }
            }
            Ok(value as i32)
        }
    }

    #[inline]
    pub fn read_var_i64(&mut self) -> Result<i64, Error> { 
        let mut byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                byte |= 0x80;
            }            
            Ok(byte as i8 as i64)
        } else {
            let mut value = (byte & 0x7f) as u64;
            let mut shift = 7;
            loop {
                byte = self.read_u8()?;
                value |= ((byte & 0x7f) as u64) << shift;
                if byte & 0x80 == 0 {
                    if byte & 0x40 != 0 {
                        value |= 0xffff_ffff_ffff_ff80 << shift;
                    }
                    break 
                }
                shift += 7;
                if shift > 63 { return Err(Error::InvalidI64) }
            }
            Ok(value as i64)
        }
    }

    #[inline]
    pub fn read_f32(&mut self) -> Result<f32, Error> { 
        self.read_slice(4).map(LittleEndian::read_f32)
    }

    #[inline]
    pub fn read_f64(&mut self) -> Result<f64, Error> { 
        self.read_slice(8).map(LittleEndian::read_f64)
    }    
}

impl<'a> Clone for Reader<'a> {
    fn clone(&self) -> Self {
        Reader::new(self.buf)
    }
}

pub trait Read<T> {
    fn read(&mut self) -> Result<T, Error>;
}

impl<'a> Read<bool> for Reader<'a> {
    fn read(&mut self) -> Result<bool, Error> {
        self.read_u8().map(|v| v != 0)
    }
}

impl<'a> Read<u8> for Reader<'a> {
    fn read(&mut self) -> Result<u8, Error> {
        self.read_u8()
    }
}

impl<'a> Read<u32> for Reader<'a> {
    fn read(&mut self) -> Result<u32, Error> {
        self.read_var_u32()
    }
}

impl<'a> Read<i32> for Reader<'a> {
    fn read(&mut self) -> Result<i32, Error> {
        self.read_var_i32()
    }
}

impl<'a> Read<i64> for Reader<'a> {
    fn read(&mut self) -> Result<i64, Error> {
        self.read_var_i64()
    }
}

impl<'a> Read<f32> for Reader<'a> {
    fn read(&mut self) -> Result<f32, Error> {
        self.read_f32()
    }
}

impl<'a> Read<f64> for Reader<'a> {
    fn read(&mut self) -> Result<f64, Error> {
        self.read_f64()
    }
}

impl<'a, T> Read<&'a [T]> for Reader<'a> {
    fn read(&mut self) -> Result<&'a [T], Error> {        
        let len = self.read_var_u32()? as usize;
        let t_len = len * mem::size_of::<T>();
        self.read_slice(t_len).map(from_byte_slice)
    }
}

impl<'a> Read<&'a str> for Reader<'a> {
    fn read(&mut self) -> Result<&'a str, Error> {
        match str::from_utf8(self.read()?) {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::InvalidUtf8)
        }
    }
}

pub struct ReadIterator<'a, T> {
    r: Reader<'a>,
    _phantom: PhantomData<T>,
}

impl<'a, T> ReadIterator<'a, T> { 
    pub fn new(r: Reader<'a>) -> Self {
        ReadIterator { r, _phantom: PhantomData }
    }
}

impl<'a, T> FallibleIterator for ReadIterator<'a, T> 
where Reader<'a>: Read<T> {
    type Item = T;
    type Error = Error;
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.r.len() > 0 {
            Ok(Some(self.r.read()?))
        } else {
            Ok(None)
        }
    }
}


pub struct SectionReadIterator<'a, T> {
    r: Reader<'a>,
    count: u32,
    _phantom: PhantomData<T>,
}

impl<'a, T> SectionReadIterator<'a, T> { 
    pub fn new(r: Reader<'a>) -> Self {
        SectionReadIterator { r, count: 0, _phantom: PhantomData }
    }
}

impl<'a, T> FallibleIterator for SectionReadIterator<'a, T> 
where Reader<'a>: Read<T> {
    type Item = T;
    type Error = Error;
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.count == 0 {
            self.r.read_var_u32()?;
        }
        if self.r.len() > 0 {
            self.count += 1;
            Ok(Some(self.r.read()?))
        } else {
            Ok(None)
        }
    }
}


