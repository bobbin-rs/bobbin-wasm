use buf::Buf;
use {Error, Section};

pub struct FunctionSection<'a>(pub Section<'a>);

impl<'a> FunctionSection<'a> {
    pub fn name(&self) -> &str {
        "FUNCTION"
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
    pub fn iter(&self) -> FunctionSectionIter<'a> {
        FunctionSectionIter { buf: self.0.buf.clone(), count: None }
    }
    pub fn get(&self, index: u32) -> Option<u32> {
        let mut iter = self.iter();
        let mut i = 0;
        while let Some(f) = iter.next() {            
            if i == index {
                return Some(f)
            }
            i += 1;
        }
        None
    }
}

pub struct FunctionSectionIter<'a> {
    buf: Buf<'a>,
    count: Option<u32>,
}

impl<'a> FunctionSectionIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<u32>, Error> {
        if self.count.is_none() {
            self.count = Some(try!(self.buf.read_var_u32()));
        }
        if self.count.unwrap() == 0 {
            return Ok(None)
        }
        self.count = Some(self.count.unwrap() - 1);
        Ok(Some(try!(self.buf.read_var_u32())))
    }
}

impl<'a> Iterator for FunctionSectionIter<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = FunctionSection(Section {id: 1, buf: Buf::new_slice(&BASIC, 0x13, 0x2)});
        assert_eq!(s.len(), 0x2);
        assert_eq!(s.count(), 1);
        let mut iter = s.iter();

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }
}