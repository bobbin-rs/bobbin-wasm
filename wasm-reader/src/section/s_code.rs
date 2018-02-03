use buf::Buf;
use {Error, Section};

pub struct CodeSection<'a>(pub Section<'a>);

impl<'a> CodeSection<'a> {
    pub fn name(&self) -> &str {
        "CODE"
    }
    pub fn start(&self) -> usize {
        self.0.buf.pos()
    }
    pub fn end(&self) -> usize {
        self.0.buf.pos() + self.0.buf.remaining() 
    }
    pub fn len(&self) -> usize {
        self.0.buf.remaining()
    }
    pub fn count(&self) -> u32 {
        self.0.buf.clone().read_var_u32().unwrap()
    }
    pub fn iter(&self) -> CodeSectionIter<'a> {
        CodeSectionIter { buf: self.0.buf.clone(), count: None, state: CodeSectionState::Init }
    }
}

#[derive(Debug)]
pub enum CodeSectionState {
    Init,
    Local { local_count: u32, body_size: u32 },
    Body { body_size: u32},
}

#[derive(Debug)]
pub enum CodeItem<'a> {
    Local(u32, i8),
    Body(Buf<'a>),
}

pub struct CodeSectionIter<'a> {
    buf: Buf<'a>,
    count: Option<u32>,
    state: CodeSectionState,
}

impl<'a> CodeSectionIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<CodeItem<'a>>, Error> {
        if self.count.is_none() {
            self.count = Some(try!(self.buf.read_var_u32()));
        }
        if self.count.unwrap() == 0 {
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
                    let code = try!(self.buf.slice_buf(len));
                    let end = try!(self.buf.read_u8());
                    if end != 0x0b {
                        return Err(Error::MissingCodeEnd);
                    }
                    self.state = CodeSectionState::Init;
                    self.count = Some(self.count.unwrap() - 1);
                    return Ok(Some(CodeItem::Body(code)));
                }
            }
        }
    }
}

impl<'a> Iterator for CodeSectionIter<'a> {
    type Item = CodeItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_code() {
        let s = CodeSection(Section {id: 1, buf: Buf::new_slice(&BASIC, 0x23, 0x16)});
        assert_eq!(s.len(), 0x16);
        assert_eq!(s.count(), 1);
        let mut iter = s.iter();

        if let Some(CodeItem::Body(body)) = iter.next() {
            assert_eq!(body.remaining(), 18);
            assert_eq!(body.as_ref(), &[
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
        assert!(iter.next().is_none());
    }
}