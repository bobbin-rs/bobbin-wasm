use buf::Buf;
use Error;

pub struct MemorySection<'a>{
    pub id: u8,
    pub start: usize,
    pub end: usize,
    pub data: &'a [u8],
}

impl<'a> MemorySection<'a> {
    pub fn new(id: u8, start: usize, end: usize, data: &'a [u8]) -> Self {
        MemorySection { id: id, start: start, end: end, data: data}
    }

    pub fn name(&self) -> &str {
        "MEMORY"
    }


    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn buf(&self) -> Buf<'a> {
        Buf::new(self.data)
    }

    pub fn count(&self) -> Result<u32, Error> {
        self.buf().read_var_u32()
    }

    pub fn iter(&self) -> Result<MemorySectionIter<'a>, Error> {
        let mut buf = self.buf();
        let count = try!(buf.read_var_u32());
        Ok(MemorySectionIter { buf: buf, count: count })
    }
}


pub struct MemorySectionIter<'a> {
    buf: Buf<'a>,
    count: u32,
}

impl<'a> MemorySectionIter<'a> {
    pub fn next(&mut self) -> Result<Option<MemoryItem>, Error> {
        if self.count == 0 {
            return Ok(None)
        }
        let flags = try!(self.buf.read_var_u32());
        let initial = try!(self.buf.read_var_u32());
        let maximum = if flags & 0x1 != 0 {
            Some(try!(self.buf.read_var_u32()))
        } else {
            None
        };
        self.count -= 1;
        Ok(Some(MemoryItem { flags: flags, initial: initial, maximum: maximum }))
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
        let s = MemorySection(&BASIC[0x17..0x1a]);
        assert_eq!(s.len(), 0x3);
        assert_eq!(s.count().unwrap(), 1);
        let mut iter = s.iter().unwrap();

        assert_eq!(iter.next().unwrap(), Some(MemoryItem { flags: 0x0, initial: 1, maximum: None}));
        assert_eq!(iter.next().unwrap(), None);
    }
}