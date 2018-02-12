use Error;

use byteorder::{ByteOrder, LittleEndian};

use core::marker::PhantomData;
use core::cell::Cell;
use core::slice;

pub const PAGE_SIZE: usize = 64;

pub struct MemoryInst<'a> {
    buf: *mut u8,
    len: Cell<usize>,
    min: usize,
    max: usize,
    _phantom: PhantomData<&'a [u8]>,
}

impl<'a> MemoryInst<'a> {
    pub fn new(buf: &'a mut [u8], min: usize, max: Option<usize>) -> MemoryInst {        
        let buf_pages = buf.len() / PAGE_SIZE;
        let buf = buf.as_mut_ptr();
        let len = Cell::new(min);
        let max = if let Some(max) = max {
            if max < buf_pages { max } else { buf_pages }
        } else {
            buf_pages
        };
        MemoryInst { buf: buf as *mut u8, len, min, max, _phantom: PhantomData }
    }

    pub fn len(&self) -> usize {
        self.len.get() * PAGE_SIZE
    }

    pub fn reset(&self) {
        self.len.set(self.min);
    }

    pub fn current_memory(&self) -> i32 {
        self.len.get() as i32
    }

    pub fn grow_memory(&mut self, pages: i32) -> i32 {
        let prev = self.current_memory();
        let next = prev + pages;
        if next <= self.max as i32 {
            self.len.set(next as usize);
            prev
        } else {
            -1
        }
    }

    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf, self.len.get() * PAGE_SIZE) }
    }

    fn as_mut(&self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.buf, self.len.get() * PAGE_SIZE) }
    }

    fn check_access(&self, index: usize, len: usize) -> Result<(), Error> {
        if index + len <= self.len() { Ok(()) } else { Err(Error::OutOfBounds) }
    }

    pub fn load_32(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 4)?;
            LittleEndian::read_i32(&self.as_ref()[index..])
        })
    }

    pub fn load_16_s(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 2)?;
            LittleEndian::read_i16(&self.as_ref()[index..]) as i16 as i32
        })
    }

    pub fn load_16_u(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 2)?;
            LittleEndian::read_i16(&self.as_ref()[index..]) as u16 as i32
        })
    }


    pub fn load8_u(&self, index: usize) -> Result<(), Error> {
        Ok({
            self.check_access(index, 1)?;
            self.as_mut()[index] as u8 as i32;
        })
    }    

    pub fn load8_s(&self, index: usize) -> Result<(), Error> {
        Ok({
            self.check_access(index, 1)?;
            self.as_mut()[index] as i8 as i32;
        })
    }    

    pub fn store_32(&self, index: usize, value: i32) -> Result<(), Error> {
        Ok({
            self.check_access(index, 4)?;
            LittleEndian::write_i32(&mut self.as_mut()[index..], value)
        })
    }

    pub fn store_16(&self, index: usize, value: i16) -> Result<(), Error> {
        Ok({
            self.check_access(index, 2)?;
            LittleEndian::write_i16(&mut self.as_mut()[index..], value)
        })
    }
    pub fn store_8(&self, index: usize, value: i8) -> Result<(), Error> {
        Ok({
            self.check_access(index, 1)?;
            self.as_mut()[index] = value as u8;
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory() {
        let mut buf = [0u8; 256];
        let mem = MemoryInst::new(&mut buf, 1, Some(4));

        for i in 0..4 {
            mem.store_32(i * 4, i as i32).unwrap();
        }

        for i in 0..4 {
            assert_eq!(mem.load_32(i * 4).unwrap(), i as i32);
        }

    }
}