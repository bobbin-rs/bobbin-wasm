#![no_std]
#![allow(unused_imports, dead_code)]
#![feature(field_init_shorthand)]

extern crate wasm_leb128;
extern crate byteorder;

pub mod buf;
pub mod smallvec;
pub mod section;
pub mod module;

use buf::Buf;
use smallvec::SmallVec;
use section::*;

#[derive(Debug)]
pub enum Error {
    BufferTooShort,
    Leb128Error(wasm_leb128::Error),
    UnknownSectionCode,
    MissingCodeEnd,
    Unspecified,
}

impl From<wasm_leb128::Error> for Error {
    fn from(other: wasm_leb128::Error) -> Error {
        Error::Leb128Error(other)
    }
}

pub struct ModuleHeader {
    pub magic: u32,
    pub version: u32,
}

pub struct Reader<'a> {
    buf: Buf<'a>,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf: Buf::new(buf) }
    }
    pub fn pos(&self) -> usize {
        self.buf.pos()
    }

    pub fn remaining(&self) -> usize {
        self.buf.remaining()
    }

    pub fn read_module_header(&mut self) -> Result<ModuleHeader, Error> {
        Ok(ModuleHeader {
            magic: try!(self.buf.read_u32()),
            version: try!(self.buf.read_u32()),
        })
    }

    pub fn read_section(&mut self) -> Result<Section, Error> {
        let id = try!(self.buf.read_var_u7());
        let payload_len = try!(self.buf.read_var_u32());
        let payload_data = try!(self.buf.slice(payload_len as usize));
        match id {
            0 => Ok(Section::Name(NameSection(payload_data))),
            1 => Ok(Section::Type(TypeSection(payload_data))),
            2 => Ok(Section::Import(ImportSection(payload_data))),
            3 => Ok(Section::Function(FunctionSection(payload_data))),
            4 => Ok(Section::Table(TableSection(payload_data))),
            5 => Ok(Section::Memory(MemorySection(payload_data))),
            6 => Ok(Section::Global(GlobalSection(payload_data))),
            7 => Ok(Section::Export(ExportSection(payload_data))),
            8 => Ok(Section::Start(StartSection(payload_data))),
            9 => Ok(Section::Element(ElementSection(payload_data))),
            10 => Ok(Section::Code(CodeSection(payload_data))),
            11 => Ok(Section::Data(DataSection(payload_data))),
            _ => Err(Error::UnknownSectionCode),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../testdata/basic.wasm");

    #[test]
    fn test_reader() {
        let mut r = Reader::new(BASIC);
        {
            let h = r.read_module_header().unwrap();
            assert_eq!(h.magic, 0x6d736100);
            assert_eq!(h.version, 0xd);
        }
        {
            assert_eq!(r.pos(), 8);
            let s = r.read_section().unwrap();
            assert_eq!(s.id(), 1);
        }
        {
            assert_eq!(r.pos(), 0x11);
            let s = r.read_section().unwrap();
            assert_eq!(s.id(), 3);
        }
        {
            assert_eq!(r.pos(), 0x15);
            let s = r.read_section().unwrap();
            assert_eq!(s.id(), 5);
        }
        {
            assert_eq!(r.pos(), 0x1a);
            let s = r.read_section().unwrap();
            assert_eq!(s.id(), 7);
        }
        {
            assert_eq!(r.pos(), 0x21);
            let s = r.read_section().unwrap();
            assert_eq!(s.id(), 10);
        }
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn test_buf() {
        let mut r = Buf::new(BASIC);
        // MagicNumber
        assert_eq!(r.read_u32().unwrap(), 0x6d736100);
        // Version
        assert_eq!(r.read_u32().unwrap(), 0xd);
        
        // Section 

        // Section Id
        assert_eq!(r.read_var_u7().unwrap(), 1);
        // Section Payload Len
        assert_eq!(r.read_var_u32().unwrap(), 7);
        // Type Count
        assert_eq!(r.read_var_u32().unwrap(), 1);
        // Form: Func
        assert_eq!(r.read_var_i7().unwrap(), -0x20);
        // Parameter Count
        assert_eq!(r.read_var_u32().unwrap(), 2);
        // Parameter 1 Type
        assert_eq!(r.read_var_i7().unwrap(), -0x01);
        // Parameter 2 Type
        assert_eq!(r.read_var_i7().unwrap(), -0x01);
        // Result Count
        assert_eq!(r.read_var_u32().unwrap(), 1);
        // Result 1 Type
        assert_eq!(r.read_var_i7().unwrap(), -0x01);
    }
}
