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

    pub fn section(&self, id: u8) -> Result<Option<(u8, usize, usize, &'a [u8])>, Error> {
        let mut sections = self.sections();
        while let Ok(Some((s_id, s_start, s_end, s_body))) = sections.next() {
            if s_id == id {
                return Ok(Some((s_id, s_start, s_end, s_body)))
            }
        }
        Ok(None)
    }

    pub fn type_section(&self) -> Result<Option<TypeSection<'a>>, Error> {
        self.section(1).map(|o| o.map(|(id, start, end, data)| TypeSection::new(id, start, end, data)))
    }

    pub fn function_section(&self) -> Result<Option<FunctionSection<'a>>, Error> {
        self.section(3).map(|o| o.map(|(id, start, end, data)| FunctionSection::new(id, start, end, data)))
    }

    pub fn memory_section(&self) -> Result<Option<MemorySection<'a>>, Error> {
        self.section(5).map(|o| o.map(|(id, start, end, data)| MemorySection::new(id, start, end, data)))
    }

    pub fn export_section(&self) -> Result<Option<ExportSection<'a>>, Error> {
        self.section(7).map(|o| o.map(|(id, start, end, data)| ExportSection::new(id, start, end, data)))
    }

    pub fn code_section(&self) -> Result<Option<CodeSection<'a>>, Error> {
        self.section(10).map(|o| o.map(|(id, start, end, data)| CodeSection::new(id, start, end, data)))
    }
}

pub struct SectionIter<'a> {
    buf: Buf<'a>
}

impl<'a> SectionIter<'a> {
    pub fn next(&mut self) -> Result<Option<(u8, usize, usize, &'a [u8])>, Error> {
        if self.buf.remaining() == 0 {
            return Ok(None)
        }
        let id = try!(self.buf.read_var_u7());
        let payload_len = try!(self.buf.read_var_u32());
        let p_start = self.buf.pos() + 8;
        let payload_data = try!(self.buf.slice(payload_len as usize));
        let p_end = self.buf.pos() + 8;
        Ok(Some((id, p_start, p_end, payload_data)))
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
