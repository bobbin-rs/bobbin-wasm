use {SectionType, TypeValue, Cursor};

use core::slice;

#[derive(Debug, PartialEq, Eq)]
pub enum ExportIndex {
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

impl From<(u8, u32)> for ExportIndex {
    fn from(other: (u8, u32)) -> Self {
        use ExportIndex::*;
        match other.0 {
            0x00 => Function(other.1),
            0x01 => Table(other.1),
            0x02 => Memory(other.1),
            0x03 => Global(other.1),
            _ => panic!("Invalid Kind: {:02x}", other.0)
        }
    }
}

pub struct Module<'a> {
    buf: &'a [u8],
}

pub struct Section<'a> {
    module: &'a Module<'a>,
    pub index: u32,
    pub section_type: SectionType,
    pub buf: &'a [u8],
}

pub struct Type<'a> {
    pub index: u32,
    pub parameters: &'a [u8],
    pub returns: &'a [u8],
}

impl<'a> Type<'a> {
    pub fn parameters(&self) -> TypeValuesIter<'a> {
        TypeValuesIter { index: 0, buf: self.parameters }
    }

    pub fn returns(&self) -> TypeValuesIter<'a> {
        TypeValuesIter { index: 0, buf: self.returns }
    }

    pub fn return_type(&self) -> Option<TypeValue> {
        self.returns.first().map(|t| TypeValue::from(*t as i8))
    }
}

pub struct Function {
    pub index: u32,
    pub signature_type_index: u32,
}

pub struct Memory {
    pub index: u32,
    pub flags: u32,
    pub minimum: u32,
    pub maximum: Option<u32>,
}

pub struct Global {
    pub index: u32,
    pub global_type: i8,
    pub mutability: u8,
    pub init_opcode: u8,
    pub init_parameter: u32,
}

pub struct Export<'a> {
    pub index: u32,
    pub identifier: &'a [u8],
    pub export_index: ExportIndex,
}

pub struct Start {
    pub function_index: u32,
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
        SectionIter { module: self, index: 0, buf: Cursor::new(self.buf) }
    }

    pub fn section(&self, st: SectionType) -> Option<Section> {
        self.iter().find(|s| s.section_type == st)
    }

    pub fn function_signature_type(&self, index: u32) -> Option<Type> {
        let f = self.section(SectionType::Function).unwrap().functions().nth(index as usize).unwrap();
        self.section(SectionType::Type).unwrap().types().nth(f.signature_type_index as usize)
    }

    pub fn global(&self, index: u32) -> Option<Global> {
        self.section(SectionType::Global).unwrap().globals().nth(index as usize)
    }
}

impl<'a> Section<'a> {
    pub fn types(&self) -> TypeIter<'a> {
        if let SectionType::Type = self.section_type {
            TypeIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            TypeIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn functions(&self) -> FunctionIter<'a> {
        if let SectionType::Function = self.section_type {
            FunctionIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            FunctionIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn globals(&self) -> GlobalIter<'a> {
        if let SectionType::Global = self.section_type {
            GlobalIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            GlobalIter { index: 0, buf: Cursor::new(&[]) }
        }
    }    

    pub fn exports(&self) -> ExportIter<'a> {
        if let SectionType::Export = self.section_type {
            ExportIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            ExportIter { index: 0, buf: Cursor::new(&[]) }
        }
    }    
}


pub struct SectionIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<Section<'a>> {
        if self.buf.len() > 0 {
            let index = self.index;
            let section_type = SectionType::from(self.buf.read_u8());
            let len = self.buf.read_u32() as usize;
            let buf = self.buf.slice(len);
            self.index += 1;
            Some(Section { module: self.module, index, section_type, buf })
        } else {
            None
        }
    }
}

pub struct TypeIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for TypeIter<'a> {
    type Item = Type<'a>;

    fn next(&mut self) -> Option<Type<'a>> {
        if self.buf.len() > 0 {
            let index = self.index;
            let p_len = self.buf.read_u32();
            let p_buf = self.buf.slice(p_len as usize);
            let r_len = self.buf.read_u32();
            let r_buf = self.buf.slice(r_len as usize);
            self.index += 1;
            Some(Type { index, parameters: p_buf, returns: r_buf })
        } else {
            None
        }
    }
}

pub struct TypeValuesIter<'a> {
    index: u32,
    buf: &'a [u8],
}

impl<'a> Iterator for TypeValuesIter<'a> {
    type Item = TypeValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let i = self.index;
            let t = TypeValue::from(self.buf[i as usize] as i8);
            self.index += 1;
            Some(t)
        } else {
            None
        }
    }
}

pub struct FunctionIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for FunctionIter<'a> {
    type Item = Function;

    fn next(&mut self) -> Option<Function> {
        if self.buf.len() > 0 {
            let index = self.index;
            self.index += 1;
            Some(Function { index, signature_type_index: self.buf.read_u32() })
        } else {
            None
        }
    }
}

pub struct GlobalIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for GlobalIter<'a> {
    type Item = Global;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let global_type = self.buf.read_i8();
            let mutability = self.buf.read_u8();
            let init_opcode = self.buf.read_u8();
            let init_parameter = self.buf.read_u32();
            self.index += 1;
            Some(Global { index, global_type, mutability, init_opcode, init_parameter })
        } else {
            None
        }
    }
}


pub struct ExportIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for ExportIter<'a> {
    type Item = Export<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let identifier = self.buf.slice_identifier();
            let kind = self.buf.read_u8();
            let export_index = ExportIndex::from((kind, self.buf.read_u32()));
            self.index += 1;
            Some(Export { index, identifier, export_index })
        } else {
            None
        }
    }
}