use wasm_leb128::{write_u1, write_u7, write_i7, write_u32, write_i32};
use byteorder::{ByteOrder, LittleEndian};
use core::ops::Deref;
use core::{mem, ptr, slice, str};
use reader::Reader;
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
        info!("split_mut: pos: {}", self.pos);
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

    pub fn split_reader(&mut self) -> Reader<'a> {
        Reader::new(self.split())
    }

    pub fn alloc_stack<T: Copy>(&mut self, len: usize) -> Stack<'a, T> {
        assert!(self.pos == 0, "Allocation can only happen with an empty writer.");
        self.pos += len * mem::size_of::<T>();
        info!("alloc_stack: len = {} pos={}", len, self.pos);
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

}