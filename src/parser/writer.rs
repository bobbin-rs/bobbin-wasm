use parser::Error;
use parser::util::*;

pub struct Writer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Writer { buf, pos: 0 }
    }

    pub fn cap(&self) -> usize {
        self.buf.len()
    }

    pub fn write_u8(&mut self, val: u8) -> Result<(), Error> {
        let pos = self.pos + 1;
        if pos > self.cap() { 
            Err(Error::UnexpectedEof)
        } else {
            self.buf[self.pos] = val;
            self.pos = pos;
            Ok(())
        }
    }
}

pub trait Write<T> {
    fn write(&mut self, val: T) -> Result<(), Error>;
}

impl<'a> Write<u32> for Writer<'a> {
    fn write(&mut self, val: u32) -> Result<(), Error> {
        Ok({
            self.write_u8((val >> 0) as u8)?;
            self.write_u8((val >> 8) as u8)?;
            self.write_u8((val >> 16) as u8)?;
            self.write_u8((val >> 24) as u8)?;
        })
    }
}

impl<'a,'b, T> Write<&'b [T]> for Writer<'a> {
    fn write(&mut self, value: &'b [T]) -> Result<(), Error> {
        Ok({
            let value: &[u8] = into_byte_slice(value);
            self.write_u8(value.len() as u8)?;
            for v in value {
                self.write_u8(*v)?;
            }
        })
    }
}