#![no_std]
#![allow(unused_imports, dead_code)]
#![feature(field_init_shorthand)]

extern crate wasm_leb128;
extern crate byteorder;

pub mod buf;
pub mod section;

use buf::Buf;
use section::*;

#[derive(Debug)]
pub enum Error {
    InvalidHeader,
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
    buf: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 2 { return Err(Error::BufferTooShort) }
        if buf[..8] != [0x00, 0x61, 0x73, 0x6d, 0x0d, 0x00, 0x00, 0x00] {
            return Err(Error::InvalidHeader);
        }
        Ok(Reader { buf: buf })
    }

    pub fn sections(&self) -> SectionIter<'a> {
        SectionIter { buf: Buf::new(&self.buf[8..]) }
    }

    pub fn section(&self, id: u8) -> Result<Option<(u8, &'a [u8])>, Error> {
        let mut sections = self.sections();
        while let Ok(Some((s_id, s_body))) = sections.next() {
            if s_id == id {
                return Ok(Some((s_id, s_body)))
            }
        }
        Ok(None)
    }

    pub fn type_section(&self) -> Result<Option<TypeSection<'a>>, Error> {
        self.section(1).map(|o| o.map(|(_,data)| TypeSection(data)))
    }

    pub fn function_section(&self) -> Result<Option<FunctionSection<'a>>, Error> {
        self.section(3).map(|o| o.map(|(_,data)| FunctionSection(data)))
    }

    pub fn memory_section(&self) -> Result<Option<MemorySection<'a>>, Error> {
        self.section(5).map(|o| o.map(|(_,data)| MemorySection(data)))
    }

    pub fn export_section(&self) -> Result<Option<ExportSection<'a>>, Error> {
        self.section(7).map(|o| o.map(|(_,data)| ExportSection(data)))
    }

    pub fn code_section(&self) -> Result<Option<CodeSection<'a>>, Error> {
        self.section(10).map(|o| o.map(|(_,data)| CodeSection(data)))
    }
}

pub struct SectionIter<'a> {
    buf: Buf<'a>
}

impl<'a> SectionIter<'a> {
    pub fn next(&mut self) -> Result<Option<(u8, &'a [u8])>, Error> {
        if self.buf.remaining() == 0 {
            return Ok(None)
        }
        let id = try!(self.buf.read_var_u7());
        let payload_len = try!(self.buf.read_var_u32());
        let payload_data = try!(self.buf.slice(payload_len as usize));
        Ok(Some((id, payload_data)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../testdata/basic.wasm");

    #[test]
    fn test_reader() {
        let r = Reader::new(BASIC).unwrap();
        let mut sections = r.sections();
        assert_eq!(sections.next().unwrap().unwrap().0, 1);
        assert_eq!(sections.next().unwrap().unwrap().0, 3);
        assert_eq!(sections.next().unwrap().unwrap().0, 5);
        assert_eq!(sections.next().unwrap().unwrap().0, 7);
        assert_eq!(sections.next().unwrap().unwrap().0, 10);
    }
    
    #[test]
    fn test_sections() {
        let r = Reader::new(BASIC).unwrap();
        let s = r.type_section().unwrap().unwrap();
        assert_eq!(s.count().unwrap(), 1);
        let s = r.function_section().unwrap().unwrap();
        assert_eq!(s.count().unwrap(), 1);
        let s = r.memory_section().unwrap().unwrap();
        assert_eq!(s.count().unwrap(), 1);
        let s = r.export_section().unwrap().unwrap();
        assert_eq!(s.count().unwrap(), 1);
        let s = r.code_section().unwrap().unwrap();
        assert_eq!(s.count().unwrap(), 1);
    }

}
