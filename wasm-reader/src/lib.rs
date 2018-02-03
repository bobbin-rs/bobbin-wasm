#![no_std]
#![allow(unused_imports, dead_code)]

extern crate wasm_leb128;
extern crate byteorder;

pub mod buf;
pub mod section;
pub mod opcode;

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

pub struct Section<'a> {
    pub id: u8,
    pub buf: Buf<'a>,
}

impl<'a> Section<'a> {
    pub fn name(&self) -> &str {
        section_name(self.id)
    }
    pub fn start(&self) -> usize {
        self.buf.pos()
    }
    pub fn end(&self) -> usize {
        self.buf.pos() + self.buf.remaining()
    }
    pub fn len(&self) -> usize {
        self.buf.remaining()
    }
}

pub struct Reader<'a> {
    buf: Buf<'a>,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, Error> {
        let r = Reader { buf: Buf::new(buf) };
        r.validate()
    }

    pub fn validate(mut self) -> Result<Self, Error> {
        if try!(self.buf.read_u32()) != 0x6d736100 { return Err(Error::InvalidHeader) }
        if try!(self.buf.read_u32()) != 0x0000000d { return Err(Error::InvalidHeader) }
        Ok(self)
    }

    pub fn sections(&self) -> SectionIter<'a> {
        SectionIter { buf: self.buf.clone()}
    }

    pub fn section(&self, id: u8) -> Option<Section<'a>> {
        self.sections().find(|s| s.id == id)
    }

    pub fn type_section(&self) -> Option<TypeSection<'a>> {
        self.section(1).map(TypeSection)
    }

    pub fn function_section(&self) -> Option<FunctionSection<'a>> {
        self.section(3).map(FunctionSection)

    }

    pub fn memory_section(&self) -> Option<MemorySection<'a>> {
        self.section(5).map(MemorySection)
    }

    pub fn export_section(&self) -> Option<ExportSection<'a>> {
        self.section(7).map(ExportSection)
    }

    pub fn code_section(&self) -> Option<CodeSection<'a>> {
        self.section(10).map(CodeSection)
    }
}

pub struct SectionIter<'a> {
    buf: Buf<'a>,
}

impl<'a> SectionIter<'a> {
    fn try_next(&mut self) -> Result<Option<Section<'a>>, Error> {
        if self.buf.remaining() == 0 { return Ok(None) }
        let id = try!(self.buf.read_var_u7());
        let payload_len = try!(self.buf.read_var_u32());
        Ok(Some(Section { id: id, buf: try!(self.buf.slice_buf(payload_len as usize)) }))
    }
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}

pub fn section_name(t: u8) -> &'static str {
    match t {
        0x1 => "TYPE",
        0x2 => "IMPORT",
        0x3 => "FUNCTION",
        0x4 => "TABLE",
        0x5 => "MEMORY",
        0x6 => "GLOBAL",
        0x7 => "EXPORT",
        0x8 => "START",
        0x9 => "ELEMENT",
        0x10 => "CODE",
        0x11 => "DATA",
        _ => panic!("unrecognized type: {:?}", t),
    }
}

pub fn type_name(t: i8) -> &'static str {
    match t {
        -0x01 => "i32",
        -0x02 => "i64",
        -0x03 => "f32",
        -0x04 => "f64",
        -0x10 => "anyfunc",
        -0x20 => "func",
        -0x40 => "empty_block",
        _ => panic!("unrecognized type: {:?}", t),
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
        assert_eq!(sections.next().unwrap().id, 1);
        assert_eq!(sections.next().unwrap().id, 3);
        assert_eq!(sections.next().unwrap().id, 5);
        assert_eq!(sections.next().unwrap().id, 7);
        assert_eq!(sections.next().unwrap().id, 10);
    }
    
    #[test]
    fn test_sections() {
        let r = Reader::new(BASIC).unwrap();
        let s = r.type_section().unwrap();
        assert_eq!(s.count(), 1);
        let s = r.function_section().unwrap();
        assert_eq!(s.count(), 1);
        let s = r.memory_section().unwrap();
        assert_eq!(s.count(), 1);
        let s = r.export_section().unwrap();
        assert_eq!(s.count(), 1);
        let s = r.code_section().unwrap();
        assert_eq!(s.count(), 1);
    }

}
