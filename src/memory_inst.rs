use Error;

use byteorder::{ByteOrder, LittleEndian};

use core::marker::PhantomData;
use core::cell::{Cell, UnsafeCell};
use core::slice;
use core::fmt;

use page_table::PageTable;

pub const PAGE_SIZE: usize = 65536;
pub const MINI_SIZE: usize = 4096;

pub struct MemoryInst<'a> {
    buf: *mut u8,
    buf_len: usize,
    page_table: UnsafeCell<PageTable>,
    num_pages: Cell<usize>,
    min_pages: usize,
    max_pages: usize,
    _phantom: PhantomData<&'a [u8]>,
}

impl<'a> MemoryInst<'a> {
    pub fn new(buf: &'a mut [u8], min_pages: usize, _max_pages: Option<usize>) -> MemoryInst {        
        let buf_len = buf.len();
        // let buf_pages = buf.len() / PAGE_SIZE;
        let mini_pages = buf.len() / MINI_SIZE;
        assert!(mini_pages <= 255);
        let buf = buf.as_mut_ptr();
        let page_table = UnsafeCell::new(PageTable::new(mini_pages as u8));
        let num_pages = Cell::new(min_pages);
        // let max_pages = if let Some(max_pages) = max_pages {
        //     if max_pages < buf_pages { max_pages } else { buf_pages }
        // } else {
        //     buf_pages
        // };
        // Allow 
        let max_pages = 64;
        MemoryInst { buf: buf as *mut u8, buf_len, page_table, num_pages, min_pages, max_pages, _phantom: PhantomData }
    }

    pub fn len(&self) -> usize {
        self.num_pages.get() * PAGE_SIZE        
    }

    pub fn cap(&self) -> usize {
        self.max_pages * PAGE_SIZE
    }

    pub fn num_pages(&self) -> usize {
        self.num_pages.get()
    }

    pub fn reset(&self) {
        self.num_pages.set(self.min_pages);
    }

    pub fn current_memory(&self) -> i32 {
        self.num_pages.get() as i32
    }

    pub fn grow_memory(&self, pages: i32) -> i32 {
        info!("grow_memory({})", pages);
        let prev = self.current_memory();
        let next = prev + pages;
        if next <= self.max_pages as i32 {
            self.num_pages.set(next as usize);
            info!("   num_pages: {}", self.num_pages());
            info!("   len: {}", self.len());
            prev
        } else {
            info!("   max_pages: {}", self.max_pages);
            -1
        }
    }

    pub fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf, self.buf_len) }
    }

    pub fn as_mut(&self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.buf, self.buf_len) }
    }

    fn check_access(&self, index: usize, len: usize) -> Result<(), Error> {
        info!("check_access({}, {}) < {}", index, len, self.len());
        if index + len <= self.len() { Ok(()) } else { Err(Error::OutOfBounds) }
    }

    fn mini_page(&self, addr: usize) -> u8 {
        (addr / MINI_SIZE) as u8
    }

    fn offset(&self, addr: usize) -> usize {
        (addr % MINI_SIZE)
    }

    fn page_table(&self) -> &mut PageTable {
        unsafe { &mut *self.page_table.get() }
    }

    pub fn map_addr(&self, v_addr: usize) -> Result<usize, Error> {
        info!("map_addr({:08x})", v_addr);

        // NOTE: This is here because LLVM allocates static memory starting at 0x100_000, which
        // is too large for the page table to handle.

        let a_addr = match v_addr {
            addr @ _ if addr < 0x00_1000 => addr,
            addr @ _ if addr >= 0x10_0000 => (addr - 0x10_0000) + 0x1000,
            _ => return Err(Error::ReservedMemoryArea),
        };

        info!("   a_addr: {:08x}", a_addr);

        let v_page = self.mini_page(a_addr);        
        info!("   v_page: {:04x}", v_page);
        if let Some(p_page) = self.page_table().get(v_page) {
            info!("   p_page: {:04x}", p_page);
            info!("   p_offset: {:04x}", self.offset(a_addr));
            let p_addr = ((p_page as usize) * MINI_SIZE) + self.offset(a_addr);
            info!("   p_addr: {:08x}", p_addr);
            Ok(p_addr)
        } else {
            Err(Error::OutOfMemory)
        }     
    }

    pub fn get(&self, index: usize) -> u8 {
        self.as_ref()[self.map_addr(index).unwrap()]
    }

    pub fn set(&self, index: usize, value: u8) {
        self.as_mut()[self.map_addr(index).unwrap()] = value
    }

    pub fn load(&self, index: usize) -> Result<i32, Error> {        
        Ok({
            self.check_access(index, 4)?;
            let index = self.map_addr(index)?;
            LittleEndian::read_i32(&self.as_ref()[index..])
        })
    }

    pub fn load16_s(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 2)?;
            let index = self.map_addr(index)?;
            LittleEndian::read_i16(&self.as_ref()[index..]) as i16 as i32
        })
    }

    pub fn load16_u(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 2)?;
            let index = self.map_addr(index)?;
            LittleEndian::read_i16(&self.as_ref()[index..]) as u16 as i32
        })
    }


    pub fn load8_u(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 1)?;
            let index = self.map_addr(index)?;
            self.as_mut()[index] as u8 as i32
        })
    }    

    pub fn load8_s(&self, index: usize) -> Result<i32, Error> {
        Ok({
            self.check_access(index, 1)?;
            let index = self.map_addr(index)?;
            self.as_mut()[index] as i8 as i32
        })
    }    

    pub fn store(&self, index: usize, value: i32) -> Result<(), Error> {
        Ok({
            self.check_access(index, 4)?;
            let index = self.map_addr(index)?;
            LittleEndian::write_i32(&mut self.as_mut()[index..], value)
        })
    }

    pub fn store16(&self, index: usize, value: i32) -> Result<(), Error> {
        Ok({
            self.check_access(index, 2)?;
            let index = self.map_addr(index)?;
            LittleEndian::write_i16(&mut self.as_mut()[index..], value as i16)
        })
    }
    pub fn store8(&self, index: usize, value: i32) -> Result<(), Error> {
        Ok({
            self.check_access(index, 1)?;
            let index = self.map_addr(index)?;
            self.as_mut()[index] = value as u8;
        })
    }
}

impl<'a> fmt::Debug for MemoryInst<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryInst {{ len: {} / {} pages: {} / {} }}",
             self.len(), self.cap(), self.num_pages(), self.max_pages
        )
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
            mem.store(i * 4, i as i32).unwrap();
        }

        for i in 0..4 {
            assert_eq!(mem.load(i * 4).unwrap(), i as i32);
        }

    }
}