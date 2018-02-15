use byteorder::{ByteOrder, LittleEndian};

use core::fmt;

#[derive(Clone)]
pub struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Cursor { buf, pos: 0 }
    }

    pub fn new_at_pos(buf: &'a [u8], pos: usize ) -> Self {
        Cursor { buf, pos }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn done(&self) -> bool {
        self.buf.len() == 0
    }

    pub fn advance(&mut self, count: usize) -> &mut Self {
        self.pos += count;
        self.buf = &self.buf[count..];
        self
    }

    pub fn advanced(&self, count: usize) -> Self {
        let pos = self.pos + count;
        let buf = &self.buf[count..];
        Cursor { buf, pos }
    }    

    pub fn rest(&self) -> Cursor<'a> {
        Cursor { buf: self.buf, pos: self.pos }
    }

    pub fn split(&mut self, len: usize) -> Cursor<'a> {        
        let buf = &self.buf[..len];
        let pos = self.pos;
        self.advance(len);
        Cursor { buf, pos }
    }

    pub fn slice(&mut self, len: usize) -> &'a [u8] {
        let v = &self.buf[..len];
        self.advance(len);
        v
    }

    // Read Unsigned

    pub fn read_u8(&mut self) -> u8 {
        let v = self.buf[0];
        self.advance(1);
        v
    }

    pub fn read_u16(&mut self) -> u16 {
        let v = LittleEndian::read_u16(self.buf);
        self.advance(2);        
        v
    }

    pub fn read_u32(&mut self) -> u32 {
        let v = LittleEndian::read_u32(self.buf);
        self.advance(4);
        v
    }

    pub fn read_u64(&mut self) -> u64 {
        let v = LittleEndian::read_u64(self.buf);
        self.advance(8);
        v
    }    

    // Read Signed    
    pub fn read_i8(&mut self) -> i8 {
        let v = self.read_u8() as i8;
        self.advance(1);
        v
    }

    pub fn read_i16(&mut self) -> i16 {
        let v = LittleEndian::read_i16(self.buf);
        self.advance(2);
        v
    }

    pub fn read_i32(&mut self) -> i32 {
        let v = LittleEndian::read_i32(self.buf);
        self.advance(4);
        v
    }

    pub fn read_i64(&mut self) -> i64 {
        let v = LittleEndian::read_i64(self.buf);
        self.advance(8);
        v
    }

    // Read Floating Point

    pub fn read_f32(&mut self) -> f32 {
        let v = LittleEndian::read_f32(self.buf);
        self.advance(4);
        v
    }    

    pub fn read_f64(&mut self) -> f64 {
        let v = LittleEndian::read_f64(self.buf);
        self.advance(8);
        v
    }

    // Read LEB128

    pub fn read_var_u1(&mut self) -> u8 {
        let byte = self.read_u8();
        if byte & 0x80 == 0 {
            if byte > 1 { panic!("Overflow") }
            byte
        } else {
            panic!("Overflow");
        }
    }

    pub fn read_var_u7(&mut self) -> u8 {
        let byte = self.read_u8();
        if byte & 0x80 == 0 {
            byte
        } else {
            panic!("Overflow");
        }
    }

    pub fn read_var_u32(&mut self) -> u32 {
        let mut byte = self.read_u8();
        if byte & 0x80 == 0 {
            byte as u32
        } else {
            let mut value = (byte & 0x7f) as u32;
            let mut shift = 7;            
            loop {
                byte = self.read_u8();
                value |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 { break }
                shift += 7;
                if shift > 31 { panic!("Overflow") }
            }
            value
        }    
    }


    pub fn read_var_u64(&mut self) -> u64 {
        let mut byte = self.read_u8();
        if byte & 0x80 == 0 {
            byte as u64
        } else {
            let mut value = (byte & 0x7f) as u64;
            let mut shift = 7;            
            loop {
                byte = self.read_u8();
                value |= ((byte & 0x7f) as u64) << shift;
                if byte & 0x80 == 0 { break }
                shift += 7;
                if shift > 63 { panic!("Overflow") }
            }
            value
        }    
    }

    pub fn read_var_i7(&mut self) -> i8 {
        let mut byte = self.read_u8();
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                byte |= 0x80;
            }            
            byte as i8
        } else {
            panic!("Overflow")
        }
    }

    pub fn read_var_i32(&mut self) -> i32 {
        let mut byte = self.read_u8();
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                byte |= 0x80;
            }            
            byte as i8 as i32
        } else {
            let mut value = (byte & 0x7f) as u32;
            let mut shift = 7;
            loop {
                byte = self.read_u8();                
                value |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 {
                    if byte & 0x40 != 0 {
                        value |= 0xffff_ff80 << shift;
                    }
                    break 
                }
                shift += 7;
                if shift > 31 { panic!("Overflow") }
            }
            value as i32
        }    
    }

    pub fn read_var_i64(&mut self) -> i64 {
        let mut byte = self.read_u8();
        if byte & 0x80 == 0 {
            if byte & 0x40 != 0 {
                byte |= 0x80;
            }            
            byte as i8 as i64
        } else {
            let mut value = (byte & 0x7f) as u64;
            let mut shift = 7;
            loop {
                byte = self.read_u8();
                value |= ((byte & 0x7f) as u64) << shift;
                if byte & 0x80 == 0 {
                    if byte & 0x40 != 0 {
                        value |= 0xffff_ffff_ffff_ff80 << shift;
                    }
                    break 
                }
                shift += 7;
                if shift > 63 { panic!("Overflow") }
            }
            value as i64
        }    
    }    
}

impl<'a> AsRef<[u8]> for Cursor<'a> {
    fn as_ref(&self) -> &[u8] {
        self.buf
    }
}

impl<'a> fmt::Debug for Cursor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cursor {{ pos: {} len: {} buf: [", self.pos, self.buf.len())?;
        for (i, b) in self.buf.iter().enumerate() {
            if i != 0 { write!(f, " ")?; }
            write!(f, "{:02x}", b)?;
        }
        writeln!(f, "] }}")?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::Cursor;

    #[test]
    fn test_read_u7() {
        fn read_u7(buf: &[u8]) -> u8 {
            let mut c = Cursor::new(buf);
            let v = c.read_var_u7();
            assert!(c.done());
            v
        }        
        for i in 0u8..128 {
            assert_eq!(read_u7(&[i]), i);            
        }
    }

    #[test]
    fn test_read_i7() {
        fn read_i7(buf: &[u8]) -> i8 {
            let mut c = Cursor::new(buf);
            let v = c.read_var_i7();
            assert!(c.done());
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
            let mut c = Cursor::new(buf);
            let v = c.read_var_u32();
            assert!(c.done());
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
            let mut c = Cursor::new(buf);
            let v = c.read_var_u64();
            assert!(c.done());
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
            let mut c = Cursor::new(buf);
            let v = c.read_var_i32();
            assert!(c.done());
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
            let mut c = Cursor::new(buf);
            let v = c.read_var_i64();
            assert!(c.done());
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