use Error;

use byteorder::{ByteOrder, LittleEndian};
use core::ops::{Index, Range};

pub type ReaderResult<T> = Result<T, Error>;

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf: buf, pos: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos
    }

    #[inline]
    pub fn incr(&mut self) {
        self.pos += 1
    }

    #[inline]
    pub fn advance(&mut self, offset: usize) {
        self.pos += offset
    }

    #[inline]
    pub fn done(&self) -> bool {
        self.pos >= self.buf.len()
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
        let value = self.read_var_u7()?;
        if value <= 1 {
            Ok(value)
        } else {
            return Err(Error::Leb128Overflow)
        }
    }

    #[inline]
    pub fn read_var_u7(&mut self) -> ReaderResult<u8> {
        let byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            Ok(byte)
        } else {
            return Err(Error::Leb128Overflow)
        }
    }

    #[inline]
    pub fn read_var_u32(&mut self) -> ReaderResult<u32> { 
        let mut byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            Ok(byte as u32)
        } else {
            let mut value = (byte & 0x7f) as u32;
            let mut shift = 7;            
            loop {
                byte = self.read_u8()?;
                value |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 { break }
                shift += 7;
                if shift > 31 { return Err(Error::Leb128Overflow) }
            }
            Ok(value)
        }          
    }

    pub fn read_var_u64(&mut self) -> ReaderResult<u64> {
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
                if shift > 63 { return Err(Error::Leb128Overflow) }
            }
            Ok(value)
        }    
    }

    #[inline]
    pub fn read_var_i7(&mut self) -> ReaderResult<i8> { 
        let mut byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                byte |= 0x80;
            }            
            Ok(byte as i8)
        } else {
            return Err(Error::Leb128Overflow)
        }
    }

    #[inline]
    pub fn read_var_i32(&mut self) -> ReaderResult<i32> { 
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
                if shift > 31 { return Err(Error::Leb128Overflow) }
            }
            Ok(value as i32)
        }
    }

    #[inline]
    pub fn read_var_i64(&mut self) -> ReaderResult<i64> { 
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
                if shift > 63 { return Err(Error::Leb128Overflow) }
            }
            Ok(value as i64)
        }
    }

    #[inline]
    pub fn read_range(&mut self, len: usize) -> ReaderResult<Range<usize>> {
        let v = self.pos..self.pos+len;
        self.pos += len;
        Ok(v)
    }    
    #[inline]
    pub fn slice(&self, range: Range<usize>) -> &[u8] {
        &self.buf[range]
    }
}

impl<'a> Index<usize> for Reader<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.buf[index]
    }
}

impl<'a> AsRef<[u8]> for Reader<'a> {
    fn as_ref(&self) -> &[u8] {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn test_read_u7() {
        fn read_u7(buf: &[u8]) -> u8 {
            let mut r = Reader::new(buf);
            let v = r.read_var_u7().unwrap();
            assert!(r.done());
            v
        }        
        for i in 0u8..128 {
            assert_eq!(read_u7(&[i]), i);            
        }
    }

    #[test]
    fn test_read_i7() {
        fn read_i7(buf: &[u8]) -> i8 {
            let mut r = Reader::new(buf);
            let v = r.read_var_i7().unwrap();
            assert!(r.done());
            v
        }        
        for i in 0u8..64 {
            assert_eq!(read_i7(&[i]), i as i8);            
        }
        assert_eq!(read_i7(&[0b0111_1111]), -1);            
        assert_eq!(read_i7(&[0b0111_1110]), -2);            
        assert_eq!(read_i7(&[0b0111_1100]), -4);
        assert_eq!(read_i7(&[0b0111_1000]), -8);
        assert_eq!(read_i7(&[0b0111_0000]), -16);
        assert_eq!(read_i7(&[0b0110_0000]), -32);
        assert_eq!(read_i7(&[0b0100_0000]), -64);
    }    

    #[test]
    fn test_read_u32() {
        fn read_u32(buf: &[u8]) -> u32 {
            let mut r = Reader::new(buf);
            let v = r.read_var_u32().unwrap();
            assert!(r.done());
            v       
        }

        // 0-7 bits
        assert_eq!(read_u32(&[0b0000000]), 0b0000000);
        assert_eq!(read_u32(&[0b1111111]), 0b1111111);

        // 8-14 bits
        assert_eq!(read_u32(&[0b10000000, 0b0000001]), 0b1_0000000);
        assert_eq!(read_u32(&[0b10000001, 0b0000001]), 0b1_0000001);
        assert_eq!(read_u32(&[0b11111111, 0b0000001]), 0b1_1111111);

        // 15-21 bits
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b0000001]), 0b1_0000000_0000000);
        assert_eq!(read_u32(&[0b10000001, 0b10000000, 0b0000001]), 0b1_0000000_0000001);
        assert_eq!(read_u32(&[0b10000000, 0b10000001, 0b0000001]), 0b1_0000001_0000000);
        assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b0000001]), 0b1_1111111_1111111);

        // 22-28 bits
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b10000000, 0b0000001]), 0b1_0000000_0000000_0000000);
        assert_eq!(read_u32(&[0b10000001, 0b10000000, 0b10000000, 0b0000001]), 0b1_0000000_0000000_0000001);
        assert_eq!(read_u32(&[0b10000000, 0b10000001, 0b10000000, 0b0000001]), 0b1_0000000_0000001_0000000);
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b10000001, 0b0000001]), 0b1_0000001_0000000_0000000);
        assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b11111111, 0b0000001]), 0b1_1111111_1111111_1111111);

        // 29-32 bits
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b10000000, 0b10000000, 0b00001000]), 0b1000_0000000_0000000_0000000_0000000);
        assert_eq!(read_u32(&[0b10000001, 0b10000010, 0b10000100, 0b10001000, 0b00001000]), 0b1000_0001000_0000100_0000010_0000001);
        assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00001111]), 0b1111_1111111_1111111_1111111_1111111);
        //assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00010000]), 0b1111_1111111_1111111_1111111_1111111);
    }

    #[test]
    fn test_read_u64() {
        fn read_u64(buf: &[u8]) -> u64 {
            let mut r = Reader::new(buf);
            let v = r.read_var_u64().unwrap();
            assert!(r.done());
            v
        }

        // 0-7 bits
        assert_eq!(read_u64(&[0b0000000]), 0b0000000);
        assert_eq!(read_u64(&[0b1111111]), 0b1111111);

        // 8-14 bits
        assert_eq!(read_u64(&[0b10000000, 0b0000001]), 0b1_0000000);
        assert_eq!(read_u64(&[0b10000001, 0b0000001]), 0b1_0000001);
        assert_eq!(read_u64(&[0b11111111, 0b0000001]), 0b1_1111111);

        // 15-21 bits
        assert_eq!(read_u64(&[0b10000000, 0b10000000, 0b0000001]), 0b1_0000000_0000000);
        assert_eq!(read_u64(&[0b10000001, 0b10000000, 0b0000001]), 0b1_0000000_0000001);
        assert_eq!(read_u64(&[0b10000000, 0b10000001, 0b0000001]), 0b1_0000001_0000000);
        assert_eq!(read_u64(&[0b11111111, 0b11111111, 0b0000001]), 0b1_1111111_1111111);

        // 22-28 bits
        assert_eq!(read_u64(&[0b10000000, 0b10000000, 0b10000000, 0b0000001]), 0b1_0000000_0000000_0000000);
        assert_eq!(read_u64(&[0b10000001, 0b10000000, 0b10000000, 0b0000001]), 0b1_0000000_0000000_0000001);
        assert_eq!(read_u64(&[0b10000000, 0b10000001, 0b10000000, 0b0000001]), 0b1_0000000_0000001_0000000);
        assert_eq!(read_u64(&[0b10000000, 0b10000000, 0b10000001, 0b0000001]), 0b1_0000001_0000000_0000000);
        assert_eq!(read_u64(&[0b11111111, 0b11111111, 0b11111111, 0b0000001]), 0b1_1111111_1111111_1111111);

        // 29-32 bits
        assert_eq!(read_u64(&[0b10000000, 0b10000000, 0b10000000, 0b10000000, 0b00001000]), 0b1000_0000000_0000000_0000000_0000000);
        assert_eq!(read_u64(&[0b10000001, 0b10000010, 0b10000100, 0b10001000, 0b00001000]), 0b1000_0001000_0000100_0000010_0000001);
        assert_eq!(read_u64(&[0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00001111]), 0b1111_1111111_1111111_1111111_1111111);
        //assert_eq!(read_u64(&[0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00010000]), 0b1111_1111111_1111111_1111111_1111111);
    }


    #[test]
    fn test_read_i32() {
        fn read_i32(buf: &[u8]) -> i32 {
            let mut r = Reader::new(buf);
            let v = r.read_var_i32().unwrap();
            assert!(r.done());
            v
        }
        
        // 0-7 bits
        assert_eq!(read_i32(&[0x0]), 0);
        assert_eq!(read_i32(&[0x1]), 1);
        assert_eq!(read_i32(&[0x2]), 2);
        assert_eq!(read_i32(&[0x7f]), -1);
        assert_eq!(read_i32(&[0x7e]), -2);

        assert_eq!(read_i32(&[0xff, 0]), 127);
        assert_eq!(read_i32(&[0x80, 1]), 128);
        assert_eq!(read_i32(&[0x81, 1]), 129);

        assert_eq!(read_i32(&[0x81, 0x7f]), -127);
        assert_eq!(read_i32(&[0x80, 0x7f]), -128);
        assert_eq!(read_i32(&[0xff, 0x7e]), -129);
    }

    #[test]
    fn test_read_i64() {
        fn read_i64(buf: &[u8]) -> i64 {
            let mut r = Reader::new(buf);
            let v = r.read_var_i64().unwrap();
            assert!(r.done());
            v
        }
        
        // 0-7 bits
        assert_eq!(read_i64(&[0x0]), 0);
        assert_eq!(read_i64(&[0x1]), 1);
        assert_eq!(read_i64(&[0x2]), 2);
        assert_eq!(read_i64(&[0x7f]), -1);
        assert_eq!(read_i64(&[0x7e]), -2);

        assert_eq!(read_i64(&[0xff, 0]), 127);
        assert_eq!(read_i64(&[0x80, 1]), 128);
        assert_eq!(read_i64(&[0x81, 1]), 129);

        assert_eq!(read_i64(&[0x81, 0x7f]), -127);
        assert_eq!(read_i64(&[0x80, 0x7f]), -128);
        assert_eq!(read_i64(&[0xff, 0x7e]), -129);
    }    
}