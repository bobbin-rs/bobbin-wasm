#![allow(unused_imports, dead_code)]

extern crate wasm_leb128;
extern crate byteorder;

use wasm_leb128::{read_u7, read_u32};

use byteorder::{ByteOrder, LittleEndian};

pub struct ModuleReader<'a> {
    buf: &'a [u8],
}

impl<'a> ModuleReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        ModuleReader { buf: buf }
    }

    pub fn magic(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[0..4])
    }

    pub fn version(&self) -> u32 {
        LittleEndian::read_u32(&self.buf[4..8])
    }
}

pub struct SectionReader<'a> {
    buf: &'a [u8],
}

impl<'a> SectionReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        SectionReader { buf: buf }
    }   
    pub fn id(&self) -> u8 {
        read_u7(self.buf).unwrap().0
    }
    pub fn payload_len(&self) -> u32 {
        read_u32(self.buf).unwrap().0
    }
    // pub fn name_len(&self) -> Option<u32> {
    //     if self.id() == 0 {
    //         Some(read_u32(self.buf[]))
    //     }
    // }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../testdata/basic.wasm");

    #[test]
    fn test_module() {
        let r = ModuleReader::new(BASIC);
        assert_eq!(r.magic(), 0x6d736100);
        assert_eq!(r.version(), 0x0d);
    }

    #[test]
    fn test_section() {
        let s = SectionReader::new(&BASIC[8..]);
        assert_eq!(s.id(), 1);
        assert_eq!(s.payload_len(), 1);
    }
}
