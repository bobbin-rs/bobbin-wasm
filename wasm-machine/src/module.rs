use byteorder::{ByteOrder, LittleEndian};
use core::slice;

use {SectionType};

pub struct Module<'a> {
    buf: &'a [u8],
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
        SectionIter { buf: self.buf }
    }
}

pub struct Section<'a> {
    pub section_type: SectionType,
    pub len: usize,
    pub payload: &'a [u8],
}

impl<'a> Section<'a> {

}

pub struct Type<'a> {
    buf: &'a [u8],
}

pub struct Function<'a> {
    buf: &'a [u8],
}

pub struct SectionIter<'a> {
    buf: &'a [u8],
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<Section<'a>> {
        if self.buf.len() > 0 {
            let section_type = SectionType::from(self.buf[0]);
            let len = LittleEndian::read_u16(&self.buf[1..]) as usize;
            let payload = &self.buf[2..len];
            self.buf = &self.buf[len..];
            Some(Section { section_type, len, payload })
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
