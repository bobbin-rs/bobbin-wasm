use byteorder::{ByteOrder, LittleEndian};
use core::ops::Deref;
use core::{mem, ptr, slice, str};
use stack::Stack;
use small_vec::SmallVec;
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

    pub fn advance(&mut self, len: usize) {
        self.pos += len;
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

    pub fn write_f32(&mut self, value: f32) -> WriteResult<()> {
        if self.pos + 4 >= self.buf.len() { return Err(Error::End) }
        LittleEndian::write_f32(&mut self.buf[self.pos..], value);
        self.pos += 4;
        Ok(())
    }

    pub fn write_i64(&mut self, value: i64) -> WriteResult<()> {
        if self.pos + 8 >= self.buf.len() { return Err(Error::End) }
        LittleEndian::write_i64(&mut self.buf[self.pos..], value);
        self.pos += 8;
        Ok(())
    }

    pub fn write_f64(&mut self, value: f64) -> WriteResult<()> {
        if self.pos + 8 >= self.buf.len() { return Err(Error::End) }
        LittleEndian::write_f64(&mut self.buf[self.pos..], value);
        self.pos += 8;
        Ok(())
    }

    pub fn write_var_u1(&mut self, value: u8) -> WriteResult<()> {
        if value > 1 { return Err(Error::Leb128Overflow) }
        self.write_u8(value)
    }

    pub fn write_var_u7(&mut self, value: u8) -> WriteResult<()> {
        if value & 0x80 != 0 { return Err(Error::Leb128Overflow) }
        self.write_u8(value)
    }

    pub fn write_var_u32(&mut self, mut value: u32) -> WriteResult<()> {
        loop {
            let mut b = value as u8 & 0b0111_1111;
            value >>= 7;
            if value != 0 {
                b |= 0b1000_0000;
            }
            self.write_u8(b)?;
            if value == 0 {
                return Ok(())
            }
        }        
    }

    pub fn write_var_u64(&mut self, mut value: u64) -> WriteResult<()> {
        loop {
            let mut b = value as u8 & 0b0111_1111;
            value >>= 7;
            if value != 0 {
                b |= 0b1000_0000;
            }
            self.write_u8(b)?;
            if value == 0 {
                return Ok(())
            }
        }        
    }

    pub fn write_var_i7(&mut self, value: i8) -> WriteResult<()> {
        let value = value as u8;
        if (value & 0x80 != 0) != (value & 0x40 != 0) {
             return Err(Error::Leb128Overflow)
        }
        self.write_u8(value & 0x7f)
    }

    pub fn write_var_i32(&mut self, mut value: i32) -> WriteResult<()> {
        const SIGN_BIT: u8 = 0b0100_0000;
        let mut more = true;
        loop {
            let mut b = value as u8 & 0b0111_1111;
            value >>= 7;
            if (value == 0 && b & SIGN_BIT == 0) ||
                (value == -1 && b & SIGN_BIT != 0) {
                    more = false;
            } else {
                b |= 0b1000_0000;
            }
            self.write_u8(b)?;
            if !more {
                return Ok(())
            }
        }           
    }

    pub fn write_var_i64(&mut self, mut value: i64) -> WriteResult<()> {
        const SIGN_BIT: u8 = 0b0100_0000;
        let mut more = true;
        loop {
            let mut b = value as u8 & 0b0111_1111;
            value >>= 7;
            if (value == 0 && b & SIGN_BIT == 0) ||
                (value == -1 && b & SIGN_BIT != 0) {
                    more = false;
            } else {
                b |= 0b1000_0000;
            }
            self.write_u8(b)?;
            if !more {
                return Ok(())
            }
        }           
    }

    // pub fn write_var_u32(&mut self, value: u32) -> WriteResult<()> {
    //     self.pos += write_u32(&mut self.buf[self.pos..], value).unwrap();        
    //     Ok(())
    // }

    // pub fn write_var_i32(&mut self, value: i32) -> WriteResult<()> {
    //     self.pos += write_i32(&mut self.buf[self.pos..], value).unwrap();        
    //     Ok(())
    // }

    pub fn write_len(&mut self, len: usize) -> WriteResult<()> {
        self.write_u32(len as u32)
    }

    pub fn copy_str(&mut self, s: &str) -> &'a str {
        assert!(self.pos == 0, "Allocation can only happen with empty writer");
        for b in s.bytes() {
            self.buf[self.pos] = b;
            self.pos += 1;
        }
        unsafe { str::from_utf8_unchecked(self.split()) }
    }

    pub fn split<T>(&mut self) -> &'a [T] {
        unsafe {
            // First Half
            let a_ptr = self.buf.as_ptr() as *const T;
            let a_len = self.pos;

            // Second Half
            let b_ptr = self.buf.as_mut_ptr().offset(self.pos() as isize);
            let b_len = self.buf.len() - self.pos();

            // Update Writer
            self.buf = slice::from_raw_parts_mut(b_ptr, b_len);
            self.pos = 0;

            // Return New Reader
            slice::from_raw_parts(a_ptr, a_len)
        }        
    }

    pub fn split_mut<T>(&mut self) -> &'a mut [T] {
        unsafe {
            // First Half
            let a_ptr = self.buf.as_mut_ptr() as *mut T;
            let a_len = self.pos / mem::size_of::<T>();

            // Second Half
            let b_ptr = self.buf.as_mut_ptr().offset(self.pos() as isize);
            let b_len = self.buf.len() - self.pos();

            // Update Writer
            self.buf = slice::from_raw_parts_mut(b_ptr, b_len);
            self.pos = 0;

            // Return New Reader
            slice::from_raw_parts_mut(a_ptr, a_len)
        }        
    }    

    // pub fn split_reader(&mut self) -> Reader<'a> {
    //     Reader::new(self.split())
    // }

    pub fn alloc_stack<T: Copy>(&mut self, len: usize) -> Stack<'a, T> {
        assert!(self.pos == 0, "Allocation can only happen with an empty writer.");
        self.pos += len * mem::size_of::<T>();
        // info!("alloc_stack: len = {} pos={}", len, self.pos);
        Stack::new(self.split_mut())
    }

    pub fn alloc_smallvec<T>(&mut self, len: usize) -> SmallVec<'a, T> {
        assert!(self.pos == 0, "Allocation can only happen with an empty writer.");        
        self.pos += len * mem::size_of::<T>();
        SmallVec::new(self.split_mut())
    }

    pub fn alloc_slice<T>(&mut self, len: usize) -> &'a mut [T] {
        assert!(self.pos == 0, "Allocation can only happen with an empty writer.");        
        self.pos += len * mem::size_of::<T>();
        self.split_mut()
    }

    // pub fn copy<T>(&mut self, value: T) -> WriteResult<&'a mut T> {
    //     unsafe { self.alloc().map(|v| { *v = value; v }) }
    // }

    pub fn align_to<T>(&mut self) -> WriteResult<()> {
        let align_of = mem::align_of::<T>();
        let cur_ptr = (&self.buf[self.pos..]).as_ptr();
        let align_offset = cur_ptr.align_offset(align_of);
        self.pos += align_offset;
        Ok(())
    }

    pub fn copy<T>(&mut self, value: T) -> WriteResult<&'a mut T> {
        unsafe {
            assert!(self.pos == 0, "Allocation can only happen with an empty writer.");
            let (size_of, align_of) = (mem::size_of::<T>(), mem::align_of::<T>());

            let buf_pos = self.pos();
            let buf_len = self.buf.len();
            let buf_ptr = self.buf.as_mut_ptr();

            let cur_ptr = buf_ptr.offset(buf_pos as isize);
            let end_ptr = cur_ptr.offset(buf_len as isize);
            let val_ptr = buf_ptr.offset(buf_ptr.align_offset(align_of) as isize);
            let new_ptr = val_ptr.offset(size_of as isize);
            if let Some(new_len) = new_ptr.offset_to(end_ptr) {
                if new_len < 0 {
                    return Err(Error::OutOfBounds);
                } else {
                    self.buf = slice::from_raw_parts_mut(new_ptr, new_len as usize);                    
                }
            }
            let val_ptr = val_ptr as *mut T;
            ptr::write(val_ptr, value);
            Ok(&mut *val_ptr)
        }
    }

    // pub fn copy_iter<T, I: Iterator<Item=T>>(&mut self, items: I) -> WriteResult<&'a mut [T]>  {
    //     self.align_to::<T>()?;
    //     self.split::<()>();

    //     let ptr = self.buf.as_ptr();
    //     let mut len = 0;

    //     for item in items {
    //         let v: T = *item;
    //         self.copy(v)?;
    //         len += 1;
    //     }
    //     Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut T, len) })        
    // }

    pub fn copy_slice<'i, T>(&mut self, items: &'i [T]) -> WriteResult<&'a mut [T]> 
    where T: Copy
    {
        self.align_to::<T>()?;
        self.split::<()>();

        let ptr = self.buf.as_ptr();

        for item in items {
            self.copy(*item)?;
        }
        Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut T, items.len()) })
    }

    pub fn into_slice(self) -> &'a mut [u8] {
        self.buf
    }
}

// impl<'a> Into<Reader<'a>> for Writer<'a> {
//     fn into(self) -> Reader<'a> {
//         Reader::new(&self.buf[..self.pos])
//     }
// }


impl<'a> Deref for Writer<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buf[..self.pos]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_mut() {
        let mut buf = [0u8; 32];
        let mut w = Writer::new(&mut buf);
        w.advance(16);
        let b: &mut [u32] = w.split_mut();
        assert_eq!(b.len(), 4);
    }

  #[test]
    fn test_alloc_stack() {
        let mut buf = [0u8; 32];
        let mut w = Writer::new(&mut buf);
        let mut v: Stack<u32> = w.alloc_stack(4);
        assert_eq!(v.cap(), 4);
        for i in 0..4 {
            v.push(i as u32).unwrap();
        }
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn test_alloc_smallvec() {
        let mut buf = [0u8; 32];
        let mut w = Writer::new(&mut buf);
        let mut v: SmallVec<u32> = w.alloc_smallvec(4);
        assert_eq!(v.cap(), 4);
        for i in 0..4 {
            v.push(i as u32);
        }
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn test_copy_str() {
        let mut buf = [0u8; 256];
        let mut w = Writer::new(&mut buf);
        let s = w.copy_str("Hello There!");
        assert_eq!(s, "Hello There!");
    }

    #[test]
    fn test_align_to() {
        let mut buf = [0u8; 256];
        let mut w = Writer::new(&mut buf);
        w.align_to::<u64>().unwrap();
        let pos = w.pos();
        w.advance(2);
        w.align_to::<u64>().unwrap();
        assert_eq!(w.pos() - pos, 8);
    }

    #[test]
    fn test_copy() {
        let mut buf = [0u8; 256];
        {
            let mut w = Writer::new(&mut buf);
            let v = w.copy(0).unwrap();
            assert_eq!(*v, 0);
            *v = 0x1234;
        }
        assert_eq!(buf[0], 0x34);
        assert_eq!(buf[1], 0x12);

    }

    #[test]
    fn test_copy_slice() {
        let mut buf = [0u8; 256];
        let mut w = Writer::new(&mut buf);

        let items: [u32; 4] = [0x12, 0x34, 0x56, 0x78];

        let new_items = w.copy_slice(&items).unwrap();
        for (a, b) in new_items.iter().zip(&items) {
            assert_eq!(*a, *b);
        }
    }

    #[test]
    fn test_copy_iter() {
        let mut buf = [0u8; 256];
        let mut w = Writer::new(&mut buf);

        let items: [u32; 4] = [0x12, 0x34, 0x56, 0x78];

        let ptr = w.buf.as_ptr();
        let mut len = 0;
        for i in items.iter() {
            w.copy(*i).unwrap();
            len += 1;
        }
        let new_items = unsafe { slice::from_raw_parts(ptr as *const u32, len) };
        for (a, b) in new_items.iter().zip(&items) {
            assert_eq!(*a, *b);
        }
    }    

    #[test]
    fn test_copy_struct() {
        let mut buf = [0u8; 256];
        let mut w = Writer::new(&mut buf);

        struct Thing<'a> {
            a: usize,
            b: u32,
            c: u8,
            d: &'a [u8],
        }

        let items = w.copy_slice(&[0u8, 1, 2, 3]).unwrap();
        let t_orig = Thing { a: 1, b: 2, c: 3, d: items };
        let t = w.copy(t_orig).unwrap();
        assert_eq!(t.a, 1);
        assert_eq!(t.b, 2);
        assert_eq!(t.c, 3);
        assert_eq!(t.d, &[0u8, 1, 2, 3]);

    }


    #[test]
    fn test_var_u1() {
        fn write_u1(buf: &mut [u8], value: u8) -> Result<usize, Error> {
            let mut w = Writer::new(buf);
            w.write_var_u1(value)?;
            Ok(w.pos())
        }
        let mut buf = [0u8; 8];
        
        assert_eq!(write_u1(&mut buf, 0).unwrap(), 1);
        assert_eq!(buf[0], 0);
        assert_eq!(write_u1(&mut buf, 1).unwrap(), 1);
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn test_var_u7() {
        fn write_u7(buf: &mut [u8], value: u8) -> Result<usize, Error> {
            let mut w = Writer::new(buf);
            w.write_var_u7(value)?;
            Ok(w.pos())
        }
        for i in 0..128 {
            let mut buf = [0u8; 8];            
            assert_eq!(write_u7(&mut buf, i).unwrap(), 1);
            assert_eq!(buf[0], i);
        }        
    }

    #[test]
    fn test_var_i7() {
        fn write_i7(buf: &mut [u8], value: i8) -> Result<usize, Error> {
            let mut w = Writer::new(buf);            
            w.write_var_i7(value)?;
            Ok(w.pos())
        }

        let mut buf = [0u8; 8];            

        assert!(write_i7(&mut buf, 64).is_err());
        assert!(write_i7(&mut buf, -65).is_err());

        assert_eq!(write_i7(&mut buf, 1).unwrap(), 1);
        assert_eq!(buf[0], 0b0000_0001);
        assert_eq!(write_i7(&mut buf, 63).unwrap(), 1);
        assert_eq!(buf[0], 0b0011_1111);

        assert_eq!(write_i7(&mut buf, -1).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1111);
        assert_eq!(write_i7(&mut buf, -2).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1110);
        assert_eq!(write_i7(&mut buf, -4).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1100);
        assert_eq!(write_i7(&mut buf, -8).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1000);
        assert_eq!(write_i7(&mut buf, -16).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_0000);
        assert_eq!(write_i7(&mut buf, -32).unwrap(), 1);
        assert_eq!(buf[0], 0b0110_0000);
        assert_eq!(write_i7(&mut buf, -64).unwrap(), 1);
        assert_eq!(buf[0], 0b0100_0000);
    }

    #[test]
    fn test_write_u32() {
        fn write_u32(buf: &mut [u8], value: u32) -> Result<usize, Error> {
            let mut w = Writer::new(buf);            
            w.write_var_u32(value)?;
            Ok(w.pos())
        }
        
        let mut buf = [0xffu8; 8];

        // 0-7 bits
        assert_eq!(write_u32(&mut buf, 0b000000).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0000]);
        assert_eq!(write_u32(&mut buf, 0b000001).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1111111).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0111_1111]);

        // 8-14 bits
        assert_eq!(write_u32(&mut buf, 0b1_0000000).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0000_0001]);

        // 15-21 bits
        assert_eq!(write_u32(&mut buf, 0b1_0000000_0000000).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_0000000_1111111).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1111_1111, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111_0000000).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1000_0000, 0b1111_1111, 0b0000_0001]);

        // 22-28 bits
        assert_eq!(write_u32(&mut buf, 0b1_0000000_0000000_0000000).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_0000100_0000010_0000001).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111_1111111_1111111).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001]);

        // 29-32 bits    
        assert_eq!(write_u32(&mut buf, 0b1_0000000_0000000_0000000_0000000).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_0001000_0000100_0000010_0000001).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b1000_1000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111_1111111_1111111_1111111).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001]);

        // 32 bits    
        assert_eq!(write_u32(&mut buf, 0b1000_0000000_0000000_0000000_0000000).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_1000]);
        assert_eq!(write_u32(&mut buf, 0b1000_0001000_0000100_0000010_0000001).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b1000_1000, 0b0000_1000]);
        assert_eq!(write_u32(&mut buf, 0b1111_1111111_1111111_1111111_1111111).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111]);
    }

    #[test]
    fn test_write_u64() {
        fn write_u64(buf: &mut [u8], value: u64) -> Result<usize, Error> {
            let mut w = Writer::new(buf);            
            w.write_var_u64(value)?;
            Ok(w.pos())
        }
        
        let mut buf = [0xffu8; 8];

        // 0-7 bits
        assert_eq!(write_u64(&mut buf, 0b000000).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0000]);
        assert_eq!(write_u64(&mut buf, 0b000001).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1111111).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0111_1111]);

        // 8-14 bits
        assert_eq!(write_u64(&mut buf, 0b1_0000000).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_1111111).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0000_0001]);

        // 15-21 bits
        assert_eq!(write_u64(&mut buf, 0b1_0000000_0000000).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_0000000_1111111).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1111_1111, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_1111111_0000000).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1000_0000, 0b1111_1111, 0b0000_0001]);

        // 22-28 bits
        assert_eq!(write_u64(&mut buf, 0b1_0000000_0000000_0000000).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_0000100_0000010_0000001).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_1111111_1111111_1111111).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001]);

        // 29-32 bits    
        assert_eq!(write_u64(&mut buf, 0b1_0000000_0000000_0000000_0000000).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_0001000_0000100_0000010_0000001).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b1000_1000, 0b0000_0001]);
        assert_eq!(write_u64(&mut buf, 0b1_1111111_1111111_1111111_1111111).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001]);

        // 32 bits    
        assert_eq!(write_u64(&mut buf, 0b1000_0000000_0000000_0000000_0000000).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_1000]);
        assert_eq!(write_u64(&mut buf, 0b1000_0001000_0000100_0000010_0000001).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b1000_1000, 0b0000_1000]);
        assert_eq!(write_u64(&mut buf, 0b1111_1111111_1111111_1111111_1111111).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111]);

        // TODO create tests for 64 bits
    }    

    #[test]
    fn test_write_i32() {
        fn write_i32(buf: &mut [u8], value: i32) -> Result<usize, Error> {
            let mut w = Writer::new(buf);            
            w.write_var_i32(value)?;
            Ok(w.pos())
        }        
        let mut buf = [0xffu8; 8];

        // 0-7 bits
        assert_eq!(write_i32(&mut buf, 0b0000000).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0000]);
        assert_eq!(write_i32(&mut buf, 0b0000001).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0001]);
        assert_eq!(write_i32(&mut buf, 0b0111111).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0011_1111]);
        assert_eq!(write_i32(&mut buf, -1).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0111_1111]);
        assert_eq!(write_i32(&mut buf, -64).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0100_0000]);

        // 8-14 bits
        assert_eq!(write_i32(&mut buf, 0b1_0000000).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0000_0001]);
        assert_eq!(write_i32(&mut buf, 0b1_1111111).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0000_0001]);
        assert_eq!(write_i32(&mut buf, -127).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0001, 0b0111_1111]);
        assert_eq!(write_i32(&mut buf, -128).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0111_1111]);
        assert_eq!(write_i32(&mut buf, -129).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0111_1110]);
    }


    #[test]
    fn test_write_i64() {
        fn write_i64(buf: &mut [u8], value: i64) -> Result<usize, Error> {
            let mut w = Writer::new(buf);            
            w.write_var_i64(value)?;
            Ok(w.pos())
        }        
        let mut buf = [0xffu8; 8];

        // 0-7 bits
        assert_eq!(write_i64(&mut buf, 0b0000000).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0000]);
        assert_eq!(write_i64(&mut buf, 0b0000001).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0001]);
        assert_eq!(write_i64(&mut buf, 0b0111111).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0011_1111]);
        assert_eq!(write_i64(&mut buf, -1).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0111_1111]);
        assert_eq!(write_i64(&mut buf, -64).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0100_0000]);

        // 8-14 bits
        assert_eq!(write_i64(&mut buf, 0b1_0000000).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0000_0001]);
        assert_eq!(write_i64(&mut buf, 0b1_1111111).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0000_0001]);
        assert_eq!(write_i64(&mut buf, -127).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0001, 0b0111_1111]);
        assert_eq!(write_i64(&mut buf, -128).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0111_1111]);
        assert_eq!(write_i64(&mut buf, -129).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0111_1110]);
    }    
}