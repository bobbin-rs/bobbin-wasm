use buf::Buf;
use {Error, Section};

pub struct MemorySection<'a>(pub Section<'a>);

impl<'a> MemorySection<'a> {
    pub fn name(&self) -> &str {
        "MEMORY"
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
    pub fn iter(&self) -> MemorySectionIter<'a> {
        MemorySectionIter { buf: self.0.buf.clone(), count: None }
    }
}

pub struct MemorySectionIter<'a> {
    buf: Buf<'a>,
    count: Option<u32>,
}

impl<'a> MemorySectionIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<MemoryItem>, Error> {
        if self.count.is_none() {
            self.count = Some(try!(self.buf.read_var_u32()));
        }
        if self.count.unwrap() == 0 {
            return Ok(None)
        }
        let flags = try!(self.buf.read_var_u32());
        let initial = try!(self.buf.read_var_u32());
        let maximum = if flags & 0x1 != 0 {
            Some(try!(self.buf.read_var_u32()))
        } else {
            None
        };
        self.count = Some(self.count.unwrap() - 1);
        Ok(Some(MemoryItem { flags: flags, initial: initial, maximum: maximum }))
    }
}

impl<'a> Iterator for MemorySectionIter<'a> {
    type Item = MemoryItem;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MemoryItem {
    flags: u32,
    initial: u32,
    maximum: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = MemorySection(Section {id: 1, buf: Buf::new_slice(&BASIC, 0x17, 0x3)});
        assert_eq!(s.len(), 0x3);
        assert_eq!(s.count(), 1);
        let mut iter = s.iter();

        assert_eq!(iter.next(), Some(MemoryItem { flags: 0x0, initial: 1, maximum: None}));
        assert_eq!(iter.next(), None);
    }
}