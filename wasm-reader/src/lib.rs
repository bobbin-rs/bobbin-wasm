#![allow(unused_imports, dead_code)]
#![feature(field_init_shorthand)]

extern crate wasm_leb128;
extern crate byteorder;

pub mod buf;

use buf::Buf;

pub type StateResult = Result<State, Error>;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Init,
    Header { magic: u32, version: u32},
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
        Reader { buf: Buf::new(buf), state: State::Init }
    }

    pub fn next(&mut self) -> StateResult {
        let state = match self.state {
            State::Init => {
                let magic = try!(self.buf.read_u32()); 
                let version = try!(self.buf.read_u32());
                State::Header { magic, version }
            },
            State::Header{..} | State::Section => {
                let id = try!(self.buf.read_var_u7());
                let payload_len = try!(self.buf.read_var_u32());
                match id {
                    1 => State::SectionType { payload_len },
                    3 => State::SectionFunction { payload_len },
                    5 => State::SectionMemory { payload_len },
                    7 => State::SectionExport { payload_len },
                    10 => State::SectionCode { payload_len },
                    _ => unimplemented!(),
                }
            },
            State::SectionType{..} => {
                let count = try!(self.buf.read_var_u32());
                State::SectionTypeEntries { count: count }
            },
            _ => unimplemented!()
        };
        self.state = state;
        Ok(state)
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
