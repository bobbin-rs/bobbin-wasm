use buf::Buf;
use {Error, Section};

pub struct ExportSection<'a>(pub Section<'a>);

impl<'a> ExportSection<'a> {
    pub fn name(&self) -> &str {
        "EXPORT"
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
    pub fn iter(&self) -> ExportSectionIter<'a> {
        ExportSectionIter { buf: self.0.buf.clone(), count: None }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportItem<'a> {
    pub field: &'a [u8],
    pub kind: u8,
    pub index: u32,
}

pub struct ExportSectionIter<'a> {
    buf: Buf<'a>,
    count: Option<u32>,
}

impl<'a> ExportSectionIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<ExportItem<'a>>, Error> {
        if self.count.is_none() {
            self.count = Some(try!(self.buf.read_var_u32()));
        }
        if self.count.unwrap() == 0 {
            return Ok(None)
        }
        let field_len = try!(self.buf.read_var_u32());
        let field = try!(self.buf.slice(field_len as usize));
        let kind = try!(self.buf.read_u8());
        let index = try!(self.buf.read_var_u32());
        self.count = Some(self.count.unwrap() - 1);
        Ok(Some(ExportItem { 
            field: field,
            kind: kind,
        index: index}))
    }
}

impl<'a> Iterator for ExportSectionIter<'a> {
    type Item = ExportItem<'a>;
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
        let s = ExportSection(Section {id: 1, buf: Buf::new_slice(&BASIC, 0x1c, 0x5)});
        assert_eq!(s.len(), 0x5);
        assert_eq!(s.count(), 1);
        let mut iter = s.iter();

        assert_eq!(iter.next(), Some(ExportItem { 
            field: b"f",
            kind: 0,
            index: 0,
        }));
        assert_eq!(iter.next(), None);
    }
}