use {Reader, SectionType};

pub struct Module<'a> {
    r: Reader<'a>,
}

impl<'a> Module<'a> {
    pub fn new(r: Reader<'a>) -> Self {
        Module { r }
    }

    pub fn read_u8_at(&self, offset: usize) -> u8 {
        self.r.read_u8_at(offset).unwrap()
    }

    pub fn read_u32_at(&self, offset: usize) -> u32 {
        self.r.read_u32_at(offset).unwrap()
    }    

    pub fn iter(&'a self) -> SectionIter<'a> {
        SectionIter { m: self, pos: 0 }
    }

}

pub struct Section<'a> {
    m: &'a Module<'a>,
    pub off: u32,
    pub sid: SectionType,
    pub len: u32,
    pub cnt: u32,
}

pub struct SectionIter<'a> {
    m: &'a Module<'a>,
    pos: usize,
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;
    fn next(&mut self) -> Option<Section<'a>> {
        if self.pos < self.m.r.len() {            
            let off = self.pos as u32;
            let sid = SectionType::from(self.m.read_u8_at(self.pos));
            let len = self.m.read_u32_at(self.pos + 1);
            let cnt = self.m.read_u32_at(self.pos + 5);

            self.pos += len as usize;
            Some(Section{ m: self.m, off, sid, len, cnt })
        } else {
            None
        }
    }
}