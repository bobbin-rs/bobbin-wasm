use buf::Buf;
use Error;

pub struct ExportSection<'a>{
    pub id: u8,
    pub start: usize,
    pub end: usize,
    pub data: &'a [u8],
}

impl<'a> ExportSection<'a> {
    pub fn new(id: u8, start: usize, end: usize, data: &'a [u8]) -> Self {
        ExportSection { id: id, start: start, end: end, data: data}
    }

    pub fn name(&self) -> &str {
        "EXPORT"
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

    pub fn iter(&self) -> Result<ExportSectionIter<'a>, Error> {
        let mut buf = self.buf();
        let count = try!(buf.read_var_u32());
        Ok(ExportSectionIter { buf: buf, count: count })
    }
}


pub struct ExportSectionIter<'a> {
    buf: Buf<'a>,
    count: u32,
}

impl<'a> ExportSectionIter<'a> {
    pub fn next(&mut self) -> Result<Option<ExportItem>, Error> {
        if self.count == 0 {
            return Ok(None)
        }
        let field_len = try!(self.buf.read_var_u32());
        let field = try!(self.buf.slice(field_len as usize));
        let kind = try!(self.buf.read_u8());
        let index = try!(self.buf.read_var_u32());
        self.count -= 1;
        Ok(Some(ExportItem { 
            field: field,
            kind: kind,
        index: index}))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportItem<'a> {
    pub field: &'a [u8],
    pub kind: u8,
    pub index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = ExportSection(&BASIC[0x1c..0x21]);
        assert_eq!(s.len(), 0x5);
        assert_eq!(s.count().unwrap(), 1);
        let mut iter = s.iter().unwrap();

        assert_eq!(iter.next().unwrap(), Some(ExportItem { 
            field_len: 1,
            field_str: b"f",
            kind: 0,
            index: 0,
        }));
        assert_eq!(iter.next().unwrap(), None);
    }
}