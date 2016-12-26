use buf::Buf;
use Error;

pub struct CodeSection<'a>(pub &'a [u8]);

impl<'a> CodeSection<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn buf(&self) -> Buf<'a> {
        Buf::new(self.0)
    }

    pub fn count(&self) -> Result<u32, Error> {
        self.buf().read_var_u32()
    }

    pub fn iter(&self) -> Result<CodeSectionIter<'a>, Error> {
        let mut buf = self.buf();
        let count = try!(buf.read_var_u32());
        Ok(CodeSectionIter { buf: buf, count: count, state: CodeSectionState::Init })
    }
}

#[derive(Debug)]
pub enum CodeSectionState {
    Init,
    Local { local_count: u32, body_size: u32 },
    Body { body_size: u32},
}

pub struct CodeSectionIter<'a> {
    buf: Buf<'a>,
    count: u32,
    state: CodeSectionState,
}

impl<'a> CodeSectionIter<'a> {
    pub fn next(&mut self) -> Result<Option<CodeItem>, Error> {
        if self.count == 0 {
            return Ok(None)
        }
        loop {
            match self.state {
                CodeSectionState::Init => {
                    let body_size = try!(self.buf.read_var_u32());
                    let p_start = self.buf.pos() as u32;
                    let local_count = try!(self.buf.read_var_u32());
                    let p_end = self.buf.pos() as u32;
                    let body_size = body_size - (p_end - p_start);
                    if local_count > 0 {
                        self.state = CodeSectionState::Local { local_count: local_count, body_size: body_size }
                    } else {
                        self.state = CodeSectionState::Body { body_size: body_size }
                    }                    
                },
                CodeSectionState::Local { local_count, body_size } => {
                    let p_start = self.buf.pos() as u32;
                    let count = try!(self.buf.read_var_u32());
                    let local_type = try!(self.buf.read_var_i7());
                    let local = CodeItem::Local(count, local_type);
                    let p_end = self.buf.pos() as u32;
                    let body_size = body_size - (p_end - p_start);
                    if local_count > 0 {
                        self.state = CodeSectionState::Local { local_count: local_count - 1, body_size: body_size };
                    } else {
                        self.state = CodeSectionState::Body { body_size: body_size };
                    }
                    return Ok(Some(local));
                }
                CodeSectionState::Body { body_size } => {
                    let len = body_size as usize - 1;
                    let code = try!(self.buf.slice(len));
                    let end = try!(self.buf.read_u8());
                    if end != 0x0b {
                        return Err(Error::MissingCodeEnd);
                    }
                    self.state = CodeSectionState::Init;
                    self.count -= 1;
                    return Ok(Some(CodeItem::Body(code)));
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CodeItem<'a> {
    Local(u32, i8),
    Body(&'a [u8]),
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = CodeSection(&BASIC[0x23..0x39]);
        assert_eq!(s.len(), 0x16);
        assert_eq!(s.count().unwrap(), 1);
        let mut iter = s.iter().unwrap();

        if let Some(CodeItem::Body(body)) = iter.next().unwrap() {
            assert_eq!(body.len(), 18);
            assert_eq!(body, &[
                0x41, 0x00,
                0x41, 0x00,
                0x28, 0x02, 0x00,
                0x41, 0x01,
                0x6a,
                0x36, 0x02, 0x00,
                0x20, 0x00,
                0x20, 0x01,
                0x6a,
            ][..])
        } else {
            panic!("Expected body");
        }
        assert_eq!(iter.next().unwrap(), None);
    }
}