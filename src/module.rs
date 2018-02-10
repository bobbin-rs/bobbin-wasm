use {SectionType, TypeValue, Value, Cursor};
use opcode::{I32_CONST, GET_GLOBAL};

use core::{slice, str, fmt};

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
    name: [u8; 64],
    name_len: usize,
    version: u32,
    buf: &'a [u8],
}

impl<'a> Module<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        let name = [0u8; 64];
        let name_len = 0;
        let version = 0;
        Module { name, name_len, version, buf }
    }

    pub fn set_name(&mut self, name: &str) {
        let mut n = 0;
        for c in name.as_bytes() {
            self.name[n] = *c;
            n += 1;
        }
        self.name_len = n;
    }

    pub fn set_version(&mut self, version: u32){
        self.version = version;
    }

    pub fn name(&self) -> &str {
        str::from_utf8(&self.name[..self.name_len]).unwrap()
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

    pub fn with_function_signature_type<T, F: FnOnce(Option<Type>)->T>(&self, index: u32, f: F) -> T {
        f(self.function_signature_type(index))
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

impl<'a> fmt::Debug for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            writeln!(f, "<Module name={:?} version={}>", self.name(), self.version)?;
            for s in self.iter() {
                s.fmt(f)?;
            }
            writeln!(f, "</Module>")?;
        })
    }
}


pub struct Section<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    pub section_type: SectionType,
    pub buf: &'a [u8],
}

impl<'a> Section<'a> {
    pub fn new(module: &'a Module<'a>, index: u32, section_type: SectionType, buf: &'a [u8]) -> Self {
        Section { module, index, section_type, buf }
    }

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

    pub fn linear_memories(&self) -> MemoryIter<'a> {
        if let SectionType::Memory = self.section_type {
            MemoryIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            MemoryIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
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

    pub fn bodies(&self) -> BodyIter<'a> {
        if let SectionType::Code = self.section_type {
            BodyIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            BodyIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn data(&self) -> DataIter<'a> {
        if let SectionType::Data = self.section_type {
            DataIter { module: self.module, index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            DataIter { module: self.module, index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn start(&self) -> Start {
        let function_index = Cursor::new(self.buf).read_u32();
        Start { module: self.module, function_index }
    }
}

impl<'a> fmt::Debug for Section<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "  ";
            writeln!(f, "{}<Section index={} type={:?} size={}>", indent, self.index, self.section_type, self.buf.len())?;
            if self.buf.len() > 0 {
                let indent = "    ";
                write!(f, "{}", indent)?;
                for (_i, b) in self.buf.iter().enumerate() {
                    write!(f, "{:02x} ", *b)?;
                }
                writeln!(f, "")?;
            }
            match self.section_type {
                SectionType::Type => {
                    for t in self.types() {
                        t.fmt(f)?;
                    }
                },
                SectionType::Function => {
                    for func in self.functions() {
                        func.fmt(f)?;
                    }
                },
                SectionType::Export => {
                    for e in self.exports() {
                        e.fmt(f)?;
                    }
                },
                SectionType::Code => {
                    for b in self.bodies() {
                        b.fmt(f)?;
                    }
                },
                SectionType::Start => {
                    self.start().fmt(f)?;
                }
                _ => {},
            }
            writeln!(f, "{}</Section>", indent)?;
        })
    }
}


pub struct Type<'a> {
    pub index: u32,
    pub parameters: &'a [u8],
    pub returns: &'a [u8],
}


impl<'a> fmt::Debug for Type<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Type index=\"{}\">", indent, self.index)?;
            for p in self.parameters {
                let indent = "      ";
                writeln!(f, "{}<parameter>{:?}</parameter>", indent, TypeValue::from(*p as i8))?;
            }
            for r in self.returns {
                let indent = "      ";
                writeln!(f, "{}<return>{:?}</return>", indent, TypeValue::from(*r as i8))?;
            }
            writeln!(f, "{}</Type>", indent)?;
        })
    }
}

impl<'a> Type<'a> {
    pub fn new(index: u32, parameters: &'a [u8], returns: &'a [u8]) -> Self {
        Type { index, parameters, returns }
    }
    
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

impl<'a> fmt::Debug for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Function index={} signature_type={}>", indent, self.index, self.signature_type_index)?;
        })
    }
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

// impl<'a> WriteTo for Table<'a> {
//     fn write_to(&self, w: &mut Writer) -> WasmResult<()> {
//         unimplemented!()
//     }
// }
// impl<'a> Table<'a> {
// }

pub struct Memory<'a> {
    pub module: &'a Module<'a>,    
    pub index: u32,
    pub flags: u32,
    pub minimum: u32,
    pub maximum: Option<u32>,
}

impl<'a> Memory<'a> {
    
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

impl<'a> fmt::Debug for Export<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Export id={:?} index={:?}>", indent, 
                str::from_utf8(self.identifier).unwrap(),
                self.export_index,
            )?;
        })
    }
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

impl<'a> fmt::Debug for Start<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Start index={:?}>", indent, self.function_index)?;
        })
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

pub struct Body<'a> {
    pub module: &'a Module<'a>,
    pub index: u32,
    pub buf: &'a [u8],
}

impl<'a> fmt::Debug for Body<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Body size={}>", indent, self.buf.len())?;
            writeln!(f, "{}</Body>", indent)?;
        })
    }
}

impl<'a> Body<'a> {
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
            let p_len = self.buf.read_u8();
            let p_buf = self.buf.slice(p_len as usize);
            let r_len = self.buf.read_u8();
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


pub struct MemoryIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for MemoryIter<'a> {
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

pub struct BodyIter<'a> {
    module: &'a Module<'a>,
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for BodyIter<'a> {
    type Item = Body<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let buf_len = self.buf.read_u32();
            let buf = self.buf.slice(buf_len as usize);
            self.index += 1;            
            Some(Body { module: self.module, index, buf })
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
