#![allow(unused_imports, dead_code)]

extern crate wasm_leb128;
extern crate byteorder;

use wasm_leb128::{read_u7, read_u32};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub enum Error {
    BufferTooShort,
    Unspecified,
}

impl From<wasm_leb128::Error> for Error {
    fn from(_other: wasm_leb128::Error) -> Error {
        Error::Unspecified
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReaderState {
    MagicNumber,
    Version,
    SectionId,
    SectionPayloadLen,
    Done,
    Error,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReaderEvent {
    MagicNumber(u32),
    Version(u32),
    SectionId(u8),
    SectionPayloadLen(u32),
    Done,
    Error,
}

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
    state: ReaderState,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf: buf, pos: 0, state: ReaderState::MagicNumber }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn slice_of(&self, size: usize) -> Result<&'a [u8], Error> {
        if size < self.remaining() { 
            Ok(&self.buf[self.pos..self.pos+4])
        } else {
            Err(Error::BufferTooShort)
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let v = LittleEndian::read_u32(try!(self.slice_of(4)));
        self.pos += 4;
        Ok(v)
    }

    pub fn read_var_u7(&mut self) -> Result<u8, Error> {
        let (v, n) = try!(read_u7(&self.buf[self.pos..]));
        self.pos += n;
        Ok(v)
    }

    pub fn read_var_u32(&mut self) -> Result<u32, Error> {
        let (v, n) = try!(read_u32(&self.buf[self.pos..]));
        self.pos += n;
        Ok(v)
    }

    pub fn next(&mut self) -> Result<ReaderEvent, Error> {
        use ReaderState::*;
        let (event, state) = match self.state {
            MagicNumber => (ReaderEvent::MagicNumber(try!(self.read_u32())), Version),
            Version => (ReaderEvent::Version(try!(self.read_u32())), SectionId),            
            SectionId => (ReaderEvent::SectionId(try!(self.read_var_u7())), SectionPayloadLen),
            SectionPayloadLen => (ReaderEvent::Done, Done),
            Done => (ReaderEvent::Done, Done),
            Error => (ReaderEvent::Error, Error)
        };
        self.state = state;
        Ok(event)
    }
}


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
    fn test_reader() {
        use self::ReaderEvent::*;
        let mut r = Reader::new(BASIC);
        assert_eq!(r.next().unwrap(), MagicNumber(0x6d736100));
        assert_eq!(r.next().unwrap(), Version(0xd));
        assert_eq!(r.next().unwrap(), SectionId(1));

    }
}
