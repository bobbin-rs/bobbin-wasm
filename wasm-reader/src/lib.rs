#![allow(unused_imports, dead_code)]
#![feature(field_init_shorthand)]

extern crate wasm_leb128;
extern crate byteorder;

pub mod buf;

use buf::Buf;

pub type ReaderResult = Result<Event, Error>;
pub type StateResult = Result<(Event, State), Error>;

#[derive(Debug)]
pub enum Error {
    BufferTooShort,
    Leb128Error(wasm_leb128::Error),
    Unspecified,
}

impl From<wasm_leb128::Error> for Error {
    fn from(other: wasm_leb128::Error) -> Error {
        Error::Leb128Error(other)
    }
}

pub enum State {
    Header,
    Section,
    SectionType { payload_len: u32 },
    SectionTypeEntries { count: u32 },
    SectionFunction { payload_len: u32 },
    SectionMemory { payload_len: u32 },
    SectionExport { payload_len: u32 },
    SectionCode { payload_len: u32 },
    End,
    Error,
}


pub enum Event {
    Header { magic: u32, version: u32 },
    SectionType,
    SectionTypeEnd,
    SectionFunction,
    SectionMemory,
    SectionExport,
    SectionCode,
    End,
    Error,
}


pub struct Reader<'a> {
    buf: Buf<'a>,
    state: State,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf: Buf::new(buf), state: State::Header }
    }

    pub fn next(&mut self) -> ReaderResult {
        let (event, state) = match self.state {
            State::Header => try!(self.header()),
            State::Section => try!(self.section()),
            State::SectionType{..} => try!(self.section_type()),
            State::SectionFunction{..} => try!(self.section_function()),
            State::SectionMemory{..} => try!(self.section_memory()),
            State::SectionExport{..} => try!(self.section_export()),
            State::SectionCode{..} => try!(self.section_code()),
            _ => try!(self.end()),
        };
        self.state = state;
        Ok(event)
    }

    pub fn header(&mut self) -> StateResult {
        let magic = try!(self.buf.read_u32());
        let version = try!(self.buf.read_u32());
        Ok(( Event::Header { magic, version }, State::Section ))
    }

    pub fn section(&mut self) -> StateResult {
        let id = try!(self.buf.read_var_u7());
        let payload_len = try!(self.buf.read_var_u32());
        Ok(match id {
            1 => (Event::SectionType, State::SectionType { payload_len }),
            3 => (Event::SectionFunction, State::SectionFunction { payload_len }),
            5 => (Event::SectionMemory, State::SectionMemory { payload_len }),
            7 => (Event::SectionExport, State::SectionExport { payload_len }),
            10 => (Event::SectionCode, State::SectionCode { payload_len }),
            _ => unimplemented!(),
        })
    }

    pub fn section_type(&mut self) -> StateResult {
        unimplemented!()
        // let count = try!(self.buf.read_var_u32());
        // Ok((Event::SectionTypeEntries{ count }, State::SectionTypeEntries{ count }))
    }

    pub fn section_function(&mut self) -> StateResult {
        unimplemented!()
    }
    pub fn section_memory(&mut self) -> StateResult {
        unimplemented!()
    }
    pub fn section_export(&mut self) -> StateResult {
        unimplemented!()
    }
    pub fn section_code(&mut self) -> StateResult {
        unimplemented!()
    }

    pub fn end(&mut self) -> StateResult {
        Ok(( Event::End, State::End))
    }

    pub fn error(&mut self) -> StateResult {
        Ok(( Event::Error, State::Error))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../testdata/basic.wasm");

    #[test]
    fn test_reader() {
        let mut r = Reader::new(BASIC);
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
