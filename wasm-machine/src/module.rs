use {SectionType, TypeValue, Value, Cursor};
use opcode::{I32_CONST, GET_GLOBAL};

use core::slice;

#[derive(Debug, PartialEq, Eq)]
pub enum ExportIndex {
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

pub enum ExportItem<'a> {
    Function(Function<'a>),
    Table(Table<'a>),
    Memory(Memory<'a>),
    Global(Global<'a>),
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
    pub module: &'a Module<'a>,
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

pub struct Function<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    pub signature_type_index: u32,
}

impl<'a> Function<'a> {
    pub fn signature_type(&self) -> Option<Type<'a>> {
        self.module.signature_type(self.signature_type_index)
    }
}

pub struct Table<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
}

pub struct Memory<'a> {
    pub module: &'a Module<'a>,    
    pub index: u32,
    pub flags: u32,
    pub minimum: u32,
    pub maximum: Option<u32>,
}

pub struct Global<'a> {
    pub module: &'a Module<'a>,    
    pub index: u32,
    pub global_type: TypeValue,
    pub mutability: u8,
    pub init_opcode: u8,
    pub init_parameter: u32,
}

impl<'a> Global<'a> {
    pub fn init_value(&self) -> Value {
        match self.init_opcode {
            I32_CONST => Value::from(self.init_parameter),
            // F32_CONST => Value::from(self.init_parameter as f32),
            GET_GLOBAL => self.module.global(self.init_parameter).unwrap().init_value(),
            _ => unimplemented!(),
        }
    }
}

pub struct Export<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    pub identifier: &'a [u8],
    pub export_index: ExportIndex,
}

impl<'a> Export<'a> {
    pub fn export_item(&self) -> Option<ExportItem<'a>> {
        use ExportIndex::*;
        match self.export_index {
            Function(index) => self.module.function(index).map(ExportItem::Function),
            Table(index) => self.module.table(index).map(ExportItem::Table),
            Memory(index) => self.module.linear_memory(index).map(ExportItem::Memory),
            Global(index) => self.module.global(index).map(ExportItem::Global),
        }
    }
}

pub struct Start<'a> {
    pub module: &'a Module<'a>,
    pub function_index: u32,
}

impl<'a> Start<'a> {
    pub fn function(&self) -> Option<Function<'a>> {
        self.module.function(self.function_index)
    }
}

pub struct Element<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    // pub table_index: u32,
    // pub offset_opcode: u8,
    // pub offset_parameter: u32,a
}

impl<'a> Element<'a> {
}

pub struct Code<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    pub body: &'a [u8],
}

impl<'a> Code<'a> {
}

pub struct Data<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    pub memory_index: u32,
    pub offset_opcode: u8,
    pub offset_parameter: u32,
    pub data: &'a [u8],
}

impl<'a> Data<'a> {
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
        self.function(index).and_then(|f| self.signature_type(f.signature_type_index))
    }

    pub fn signature_type(&self, index: u32) -> Option<Type> {
        self.section(SectionType::Type).unwrap().types().nth(index as usize)
    }

    pub fn function(&self, index: u32) -> Option<Function> {
        self.section(SectionType::Function).unwrap().functions().nth(index as usize)
    }

    pub fn table(&self, index: u32) -> Option<Table> {
        self.section(SectionType::Table).unwrap().tables().nth(index as usize)
    }

    pub fn linear_memory(&self, index: u32) -> Option<Memory> {
        self.section(SectionType::Table).unwrap().linear_memories().nth(index as usize)
    }

    pub fn global(&self, index: u32) -> Option<Global> {
        self.section(SectionType::Global).unwrap().globals().nth(index as usize)
    }

    pub fn start(&self) -> Option<Function> {
        self.section(SectionType::Start).and_then(|s| self.function(Cursor::new(s.buf).read_u32()))
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
            FunctionIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            FunctionIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn tables(&self) -> TableIter<'a> {
        if let SectionType::Table = self.section_type {
            TableIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            TableIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn linear_memories(&self) -> LinearMemoryIter<'a> {
        if let SectionType::LinearMemory = self.section_type {
            LinearMemoryIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            LinearMemoryIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }        

    pub fn globals(&self) -> GlobalIter<'a> {
        if let SectionType::Global = self.section_type {
            GlobalIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            GlobalIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }    

    pub fn exports(&self) -> ExportIter<'a> {
        if let SectionType::Export = self.section_type {
            ExportIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            ExportIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn elements(&self) -> ElementIter<'a> {
        if let SectionType::Element = self.section_type {
            ElementIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            ElementIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn codes(&self) -> CodeIter<'a> {
        if let SectionType::Code = self.section_type {
            CodeIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            CodeIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn data(&self) -> DataIter<'a> {
        if let SectionType::Data = self.section_type {
            DataIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            DataIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
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

    fn next(&mut self) -> Option<Self::Item> {
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

    fn next(&mut self) -> Option<Self::Item> {
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
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for FunctionIter<'a> {
    type Item = Function<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            self.index += 1;
            Some(Function { module: self.module, index, signature_type_index: self.buf.read_u32() })
        } else {
            None
        }
    }
}

pub struct TableIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for TableIter<'a> {
    type Item = Table<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            self.index += 1;
            Some(Table { module: self.module, index })
        } else {
            None
        }
    }
}


pub struct LinearMemoryIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for LinearMemoryIter<'a> {
    type Item = Memory<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let flags = self.buf.read_u32();
            let minimum = self.buf.read_u32();
            let maximum = if flags == 1 {
                Some(self.buf.read_u32())
            } else {
                None
            };
            self.index += 1;
            Some(Memory { module: self.module, index, flags, minimum, maximum })
        } else {
            None
        }
    }
}

pub struct GlobalIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for GlobalIter<'a> {
    type Item = Global<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let global_type = TypeValue::from(self.buf.read_i8());
            let mutability = self.buf.read_u8();
            let init_opcode = self.buf.read_u8();
            let init_parameter = self.buf.read_u32();
            self.index += 1;
            Some(Global { module: self.module, index, global_type, mutability, init_opcode, init_parameter })
        } else {
            None
        }
    }
}


pub struct ExportIter<'a> {
    module: &'a Module<'a>,
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
            Some(Export { module: self.module, index, identifier, export_index })
        } else {
            None
        }
    }
}

pub struct ElementIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            self.index += 1;
            Some(Element { module: self.module, index })
        } else {
            None
        }
    }
}

pub struct CodeIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for CodeIter<'a> {
    type Item = Code<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let body_len = self.buf.read_u32();
            let body = self.buf.slice(body_len as usize);
            self.index += 1;            
            Some(Code { module: self.module, index, body })
        } else {
            None
        }
    }
}

pub struct DataIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for DataIter<'a> {
    type Item = Data<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let memory_index = self.buf.read_u32();
            let offset_opcode = self.buf.read_u8();            
            let offset_parameter = self.buf.read_u32();
            let data_len = self.buf.read_u32();
            let data = self.buf.slice(data_len as usize);
            self.index += 1;
            Some(Data { module: self.module, index, memory_index, offset_opcode, offset_parameter, data })
        } else {
            None
        }
    }
}
