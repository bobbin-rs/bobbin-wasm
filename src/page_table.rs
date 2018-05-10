// WASM pages are 64KB
// Compilers often reserve a number of pages (16?) at the beginning of the address space.
// and use additional pages which are fairly sparse
//
// This is a simple proof of concept mapping memory addresses to 4096 byte pages.
// Other page sizes may be useful depending on the application and should eventually be supported.
//
// The page table maps from virtual page to physical page. In the map, 0xff indicates that the page
// is not assigned. Values from 0x00 to 0xfe are valid, allowing 255 x 4096 (983,040) bytes of memory.

use byteorder::{ByteOrder, LittleEndian};

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OutOfMemory,
    InvalidAlignment,
}

pub struct PageTable {
    map: [u8; 256],
    pages: u8,
    mapped: u8,
}

impl PageTable {
    pub fn new(pages: u8) -> Self {
        PageTable { map: [0xff; 256], pages, mapped: 0 }
    }

    pub fn pages(&self) -> u8 {
        self.pages
    }

    pub fn mapped(&self) -> u8 {
        self.mapped
    }

    pub fn get(&mut self, virt: u8) -> Option<u8> {
        match self.map[virt as usize] {
            0xff => if self.mapped() < self.pages() {
                let p = self.mapped;
                self.map[virt as usize] = p;
                self.mapped += 1;
                Some(p)
            } else {
                None
            },
            p @ _ => Some(p)
        }
    }

    pub fn allocate(&mut self, virt: u8) -> Option<u8> {
        if self.mapped < self.pages {            
            let page = self.mapped;
            self.mapped += 1;
            self.map[virt as usize] = page;
            Some(page)
        } else {
            None
        }
    }

}

pub struct Memory<'a> {
    page_table: PageTable,
    buf: &'a mut [u8],
}

impl<'a> Memory<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Memory {
            page_table: PageTable::new((buf.len() / PAGE_SIZE) as u8),
            buf,
        }
    }

    fn page(&self, addr: usize) -> u8 {
        (addr / PAGE_SIZE) as u8
    }

    fn offset(&self, addr: usize) -> usize {
        (addr % PAGE_SIZE)
    }

    pub fn pages(&self) -> usize {
        self.page_table.pages() as usize
    }

    pub fn mapped(&self) -> usize {
        self.page_table.mapped() as usize
    }

    pub fn map_addr(&mut self, v_addr: usize) -> Result<usize, Error> {
        let v_page = self.page(v_addr);        
        if let Some(p_page) = self.page_table.get(v_page) {
            Ok(((p_page as usize) * PAGE_SIZE) + self.offset(v_addr))
        } else {
            Err(Error::OutOfMemory)
        }     
    }

    pub fn get_u8(&mut self, v_addr: usize) -> Result<u8, Error> {      
        Ok(self.buf[self.map_addr(v_addr)?])
    }

    pub fn set_u8(&mut self, v_addr: usize, value: u8) -> Result<(), Error> {      
        Ok(self.buf[self.map_addr(v_addr)?] = value)
    }

    pub fn get_u16(&mut self, v_addr: usize) -> Result<u16, Error> {
        if v_addr & 0b1 != 0 { return Err(Error::InvalidAlignment) }
        let p_addr = self.map_addr(v_addr)?;
        Ok(LittleEndian::read_u16(&self.buf[p_addr..]))
    }

    pub fn set_u16(&mut self, v_addr: usize, value: u16) -> Result<(), Error> {      
        if v_addr & 0b1 != 0 { return Err(Error::InvalidAlignment) }
        let p_addr = self.map_addr(v_addr)?;
        Ok(LittleEndian::write_u16(&mut self.buf[p_addr..], value))
    }

    pub fn get_u32(&mut self, v_addr: usize) -> Result<u32, Error> {
        if v_addr & 0b11 != 0 { return Err(Error::InvalidAlignment) }
        let p_addr = self.map_addr(v_addr)?;
        Ok(LittleEndian::read_u32(&self.buf[p_addr..]))
    }

    pub fn set_u32(&mut self, v_addr: usize, value: u32) -> Result<(), Error> {      
        if v_addr & 0b11 != 0 { return Err(Error::InvalidAlignment) }
        let p_addr = self.map_addr(v_addr)?;
        Ok(LittleEndian::write_u32(&mut self.buf[p_addr..], value))
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table() {
        for i in 0..255 {
            let mut pt = PageTable::new(i);
            assert_eq!(pt.pages(), i);
            assert_eq!(pt.mapped(), 0);

            for j in 0..i {
                assert_eq!(pt.mapped(), j);
                let p = pt.allocate(j);
                assert_eq!(p, Some(j));
                assert_eq!(pt.get(j), p);
                
            }

            assert_eq!(pt.allocate(i), None);
            assert_eq!(pt.mapped(), i);
        }
    }

    #[test]
    fn test_memory() {
        let mut buf = [0u8; PAGE_SIZE * 2];
        let mut mem = Memory::new(&mut buf);
        assert_eq!(mem.pages(), 2);
        assert_eq!(mem.mapped(), 0);

        for i in PAGE_SIZE..(PAGE_SIZE * 3) {
            mem.set_u8(i, i as u8).unwrap();
            assert_eq!(mem.get_u8(i).unwrap(), i as u8);
        }

        assert!(mem.set_u8(PAGE_SIZE * 3, 0xff).is_err());

        assert_eq!(mem.get_u16(PAGE_SIZE).unwrap(), 0x0100);
        assert_eq!(mem.get_u32(PAGE_SIZE).unwrap(), 0x03020100);

        mem.set_u16(PAGE_SIZE, 0xabcd).unwrap();
        assert_eq!(mem.get_u16(PAGE_SIZE).unwrap(), 0xabcd);
        mem.set_u32(PAGE_SIZE, 0xabcd0123).unwrap();
        assert_eq!(mem.get_u32(PAGE_SIZE).unwrap(), 0xabcd0123);

        assert_eq!(mem.get_u8(PAGE_SIZE + 0).unwrap(), 0x23);
        assert_eq!(mem.get_u8(PAGE_SIZE + 1).unwrap(), 0x01);
        assert_eq!(mem.get_u8(PAGE_SIZE + 2).unwrap(), 0xcd);
        assert_eq!(mem.get_u8(PAGE_SIZE + 3).unwrap(), 0xab);

        assert_eq!(mem.get_u16(PAGE_SIZE + 1), Err(Error::InvalidAlignment));
        assert_eq!(mem.set_u16(PAGE_SIZE + 1, 0x0), Err(Error::InvalidAlignment));

        assert_eq!(mem.get_u32(PAGE_SIZE + 1), Err(Error::InvalidAlignment));
        assert_eq!(mem.get_u32(PAGE_SIZE + 2), Err(Error::InvalidAlignment));
        assert_eq!(mem.get_u32(PAGE_SIZE + 3), Err(Error::InvalidAlignment));
        assert_eq!(mem.set_u32(PAGE_SIZE + 1, 0), Err(Error::InvalidAlignment));
        assert_eq!(mem.set_u32(PAGE_SIZE + 2, 0), Err(Error::InvalidAlignment));
        assert_eq!(mem.set_u32(PAGE_SIZE + 3, 0), Err(Error::InvalidAlignment));
    }
}
