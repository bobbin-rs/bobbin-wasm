use buf::Buf;
use Error;

pub struct FunctionSection<'a>{
    pub id: u8,
    pub start: usize,
    pub end: usize,
    pub data: &'a [u8],
}

impl<'a> FunctionSection<'a> {
    pub fn new(id: u8, start: usize, end: usize, data: &'a [u8]) -> Self {
        FunctionSection { id: id, start: start, end: end, data: data}
    }

    pub fn name(&self) -> &str {
        "FUNCTION"
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

    pub fn iter(&self) -> Result<FunctionSectionIter<'a>, Error> {
        let mut buf = self.buf();
        let count = try!(buf.read_var_u32());
        Ok(FunctionSectionIter { buf: buf, count: count })
    }
}

pub struct FunctionSectionIter<'a> {
    buf: Buf<'a>,
    count: u32,
}

impl<'a> FunctionSectionIter<'a> {
    pub fn next(&mut self) -> Result<Option<u32>, Error> {
        if self.count == 0 {
            return Ok(None)
        }
        self.count -= 1;
        Ok(Some(try!(self.buf.read_var_u32())))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = FunctionSection(&BASIC[0x13..0x15]);
        assert_eq!(s.len(), 0x2);
        assert_eq!(s.count().unwrap(), 1);
        let mut iter = s.iter().unwrap();

        assert_eq!(iter.next().unwrap(), Some(0));
        assert_eq!(iter.next().unwrap(), None);
    }
}