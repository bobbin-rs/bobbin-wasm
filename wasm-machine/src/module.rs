use {Reader, SectionType};

pub struct Module<'a> {
    r: Reader<'a>,
    types: [Option<(u32, u32)>; 64],
    funcs: [Option<u32>; 64],
}

impl<'a> Module<'a> {
    pub fn new(r: Reader<'a>) -> Self {
        Module { 
            r,
            types: [None; 64],
            funcs: [None; 64],
        }
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

    pub fn join_reader(&mut self, r: Reader) {
        self.r.join_reader(r)
    }
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

pub struct Section<'a> {
    m: &'a Module<'a>,
    pub off: u32,
    pub sid: SectionType,
    pub len: u32,
    pub cnt: u32,
}

impl<'a> Section<'a> {
    pub fn iter_types(&'a self) -> TypeIter<'a> {
        assert!(self.sid == SectionType::Type);
        TypeIter { m: self.m, s: self, pos: self.off, n: 0 }
    }

    // pub fn iter_functions(&'a self) -> FunctionIter<'a> {
    //     assert!(self.sid == SectionType::Type);
    //     FunctionIter { m: self, s: self, pos: self.off }
    // }    
}


pub struct TypeIter<'a> {
    m: &'a Module<'a>,
    s: &'a Section<'a>,
    pos: u32,
    n: u32,
}

impl<'a> Iterator for TypeIter<'a> {
    type Item = Type<'a>;
    fn next(&mut self) -> Option<Type<'a>> {
        if self.n < self.s.cnt {           
            let pos = self.pos as u32;
            let n = self.n;            
            let len = self.m.read_u32_at(self.pos as usize);
            self.n += 1;
            self.pos += len;
            Some(Type{ m: self.m, s: self.s, pos, n })
        } else {
            None
        }
    }
}

pub struct Type<'a> {
    m: &'a Module<'a>,
    s: &'a Section<'a>,
    pos: u32,
    n: u32,    
}

// impl<'a> Type<'a> {
//     pub fn params(&self) -> &[u8] {

//     }
// }

// pub struct FunctionIter<'a> {
//     m: &'a Module<'a>,
//     pos: usize,
//     s: &'a Section<'a>,    
// }
