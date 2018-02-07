use {SectionType, Cursor};

use core::slice;

pub struct Module<'a> {
    buf: &'a [u8],
}

pub struct Section<'a> {
    pub section_type: SectionType,
    pub buf: &'a [u8],
}

pub struct Type<'a> {
    pub parameters: &'a [u8],
    pub returns: &'a [u8],
}

pub struct Function {
    pub signature: u32,
}

pub struct Global {
    pub global_type: i8,
    pub mutability: u8,
    pub init_opcode: u8,
    pub init_parameter: u32,
}

pub struct Export<'a> {
    pub identifier: &'a [u8],
    pub kind: u8,
    pub index: u32,
}

impl<'a> Module<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Module { buf }
    }

    pub fn extend(&mut self, buf: &[u8]) {
        let a_ptr = self.buf.as_ptr();
        let a_len = self.buf.len();
        let b_ptr = buf.as_ptr();
        let b_len = buf.len();

        unsafe {
            assert!(a_ptr.offset(a_len as isize) == b_ptr);
            self.buf = slice::from_raw_parts(a_ptr, a_len + b_len)
        }
    }

    pub fn iter(&self) -> SectionIter {
        SectionIter { buf: Cursor::new(self.buf) }
    }

    pub fn section(&self, st: SectionType) -> Option<Section> {
        self.iter().find(|s| s.section_type == st)
    }

    pub fn function_signature_type(&self, index: usize) -> Option<Type> {
        let f = self.section(SectionType::Function).unwrap().functions().nth(index).unwrap();
        self.section(SectionType::Type).unwrap().types().nth(f.signature as usize)
    }

    pub fn global(&self, index: usize) -> Option<Global> {
        self.section(SectionType::Global).unwrap().globals().nth(index)
    }
}

impl<'a> Section<'a> {
    pub fn types(&self) -> TypeIter<'a> {
        if let SectionType::Type = self.section_type {
            TypeIter { buf: Cursor::new(&self.buf[4..]) }
        } else {
            TypeIter { buf: Cursor::new(&[]) }
        }
    }

    pub fn functions(&self) -> FunctionIter<'a> {
        if let SectionType::Function = self.section_type {
            FunctionIter { buf: Cursor::new(&self.buf[4..]) }
        } else {
            FunctionIter { buf: Cursor::new(&[]) }
        }
    }

    pub fn globals(&self) -> GlobalIter<'a> {
        if let SectionType::Global = self.section_type {
            GlobalIter { buf: Cursor::new(&self.buf[4..]) }
        } else {
            GlobalIter { buf: Cursor::new(&[]) }
        }
    }    

    pub fn exports(&self) -> ExportIter<'a> {
        if let SectionType::Export = self.section_type {
            ExportIter { buf: Cursor::new(&self.buf[4..]) }
        } else {
            ExportIter { buf: Cursor::new(&[]) }
        }
    }    
}


pub struct SectionIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<Section<'a>> {
        if self.buf.len() > 0 {
            let section_type = SectionType::from(self.buf.read_u8());
            let len = self.buf.read_u32() as usize;
            let buf = self.buf.slice(len);
            Some(Section { section_type, buf })
        } else {
            None
        }
    }
}

pub struct TypeIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for TypeIter<'a> {
    type Item = Type<'a>;

    fn next(&mut self) -> Option<Type<'a>> {
        if self.buf.len() > 0 {
            let p_len = self.buf.read_u32();
            let p_buf = self.buf.slice(p_len as usize);
            let r_len = self.buf.read_u32();
            let r_buf = self.buf.slice(r_len as usize);
            Some(Type { parameters: p_buf, returns: r_buf })
        } else {
            None
        }
    }
}

pub struct FunctionIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for FunctionIter<'a> {
    type Item = Function;

    fn next(&mut self) -> Option<Function> {
        if self.buf.len() > 0 {
            Some(Function { signature: self.buf.read_u32() })
        } else {
            None
        }
    }
}

pub struct GlobalIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for GlobalIter<'a> {
    type Item = Global;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let global_type = self.buf.read_i8();
            let mutability = self.buf.read_u8();
            let init_opcode = self.buf.read_u8();
            let init_parameter = self.buf.read_u32();
            Some(Global { global_type, mutability, init_opcode, init_parameter })
        } else {
            None
        }
    }
}


pub struct ExportIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for ExportIter<'a> {
    type Item = Export<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let identifier = self.buf.slice_identifier();
            let kind = self.buf.read_u8();
            let index = self.buf.read_u32();
            Some(Export { identifier, kind, index })
        } else {
            None
        }
    }
}


// pub struct Section<'a> {
//     m: &'a Module<'a>,
//     pub off: u32,
//     pub sid: SectionType,
//     pub len: u32,
//     pub cnt: u32,
// }

// impl<'a> Section<'a> {
//     pub fn iter_types(&'a self) -> TypeIter<'a> {
//         assert!(self.sid == SectionType::Type);
//         TypeIter { m: self.m, s: self, pos: self.off, n: 0 }
//     }

//     // pub fn iter_functions(&'a self) -> FunctionIter<'a> {
//     //     assert!(self.sid == SectionType::Type);
//     //     FunctionIter { m: self, s: self, pos: self.off }
//     // }    
// }


// pub struct TypeIter<'a> {
//     m: &'a Module<'a>,
//     s: &'a Section<'a>,
//     pos: u32,
//     n: u32,
// }

// impl<'a> Iterator for TypeIter<'a> {
//     type Item = Type<'a>;
//     fn next(&mut self) -> Option<Type<'a>> {
//         if self.n < self.s.cnt {           
//             let pos = self.pos as u32;
//             let n = self.n;            
//             let len = self.m.read_u32_at(self.pos as usize);
//             self.n += 1;
//             self.pos += len;
//             Some(Type{ m: self.m, s: self.s, pos, n })
//         } else {
//             None
//         }
//     }
// }

// pub struct Type<'a> {
//     m: &'a Module<'a>,
//     s: &'a Section<'a>,
//     pos: u32,
//     n: u32,    
// }

// // impl<'a> Type<'a> {
// //     pub fn params(&self) -> &[u8] {

// //     }
// // }

// // pub struct FunctionIter<'a> {
// //     m: &'a Module<'a>,
// //     pos: usize,
// //     s: &'a Section<'a>,    
// // }
