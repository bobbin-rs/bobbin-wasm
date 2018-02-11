use {SectionType, TypeValue, Cursor};
use types::{ResizableLimits};

use core::{slice, str, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum ExportIndex {
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

// pub enum ExportItem {
//     Function(Function),
//     Table(Table),
//     Memory(Memory),
//     Global(Global),
// }

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
    name: &'a str,
    version: u32,
    buf: &'a [u8],
}

impl<'a> Module<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        let name = "";
        let version = 0;
        Module { name, version, buf }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn set_name(&mut self, name: &'a str, buf: &'a [u8]) {
        self.name = name;
        self.buf = buf;
    }

    pub fn set_version(&mut self, version: u32){
        self.version = version;
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
        SectionIter { index: 0, buf: Cursor::new(self.buf) }
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
    pub index: u32,
    pub section_type: SectionType,
    pub buf: &'a [u8],
}

impl<'a> Section<'a> {
    pub fn new(index: u32, section_type: SectionType, buf: &'a [u8]) -> Self {
        Section { index, section_type, buf }
    }

    pub fn types(&self) -> TypeIter<'a> {
        if let SectionType::Type = self.section_type {
            TypeIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            TypeIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn imports(&self) -> ImportIter<'a> {
        if let SectionType::Import = self.section_type {
            ImportIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            ImportIter { index: 0, buf: Cursor::new(&[]) }
        }
    }    

    pub fn functions(&self) -> FunctionIter<'a> {
        if let SectionType::Function = self.section_type {
            FunctionIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            FunctionIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn tables(&self) -> TableIter<'a> {
        if let SectionType::Table = self.section_type {
            TableIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            TableIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn linear_memories(&self) -> MemoryIter<'a> {
        if let SectionType::Memory = self.section_type {
            MemoryIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            MemoryIter { index: 0, buf: Cursor::new(&[]) }
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

    pub fn start(&self) -> Start {
        let function_index = Cursor::new(self.buf).read_u32();
        Start { function_index }
    }

    pub fn elements(&self) -> ElementIter<'a> {
        if let SectionType::Element = self.section_type {
            ElementIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            ElementIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn bodies(&self) -> BodyIter<'a> {
        if let SectionType::Code = self.section_type {
            BodyIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            BodyIter { index: 0, buf: Cursor::new(&[]) }
        }
    }

    pub fn data(&self) -> DataIter<'a> {
        if let SectionType::Data = self.section_type {
            DataIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
        } else {
            DataIter { index: 0, buf: Cursor::new(&[]) }
        }
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
                SectionType::Import => {
                    for i in self.imports() {
                        i.fmt(f)?;
                    }
                },                
                SectionType::Function => {
                    for func in self.functions() {
                        func.fmt(f)?;
                    }
                },
                SectionType::Table => {
                    for t in self.tables() {
                        t.fmt(f)?;
                    }
                },                
                SectionType::Memory => {
                    for m in self.linear_memories() {
                        m.fmt(f)?;
                    }
                },      
                SectionType::Global => {
                    for g in self.globals() {
                        g.fmt(f)?;
                    }
                },                          
                SectionType::Export => {
                    for e in self.exports() {
                        e.fmt(f)?;
                    }
                },
                SectionType::Start => {
                    self.start().fmt(f)?;
                }                
                SectionType::Code => {
                    for b in self.bodies() {
                        b.fmt(f)?;
                    }
                },
                SectionType::Data => {
                    for d in self.data() {
                        d.fmt(f)?;
                    }
                },

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

pub struct Import<'a> {
    pub index: u32,
    pub module: &'a [u8],
    pub export: &'a [u8],
    pub external_index: u32,    
}

impl<'a> fmt::Debug for Import<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Import module={:?} export={:?} index={:?}>", indent, 
                str::from_utf8(self.module).unwrap(),
                str::from_utf8(self.export).unwrap(),
                self.external_index,
            )?;
        })
    }
}

pub struct Function {
    pub index: u32,
    pub signature_type_index: u32,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Function index={} signature_type={}>", indent, self.index, self.signature_type_index)?;
        })
    }
}

pub struct Table {
    pub index: u32,
    pub element_type: TypeValue,
    pub limits: ResizableLimits,
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Table type={:?} min={} max={:?}>", indent,
                self.element_type, self.limits.min, self.limits.max
            )?;
        })
    }
}

pub struct Memory {
    pub index: u32,
    pub limits: ResizableLimits,
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Memory min={} max={:?}>", indent, 
                self.limits.min, self.limits.max)?;
        })
    }
}

pub struct Global {
    pub index: u32,
    pub global_type: TypeValue,
    pub mutability: u8,
    pub opcode: u8,
    pub immediate: u32,
}

impl fmt::Debug for Global {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Global type={:?} mutability={} opcode=0x{:02x} immediate=0x{:08x}>", 
                indent, self.global_type, self.mutability, self.opcode, self.immediate)?;
        })
    }
}

// impl Global {
//     pub fn init_value(&self) -> Value {
//         match self.init_opcode {
//             I32_CONST => Value::from(self.init_parameter),
//             // F32_CONST => Value::from(self.init_parameter as f32),
//             GET_GLOBAL => self.module.global(self.init_parameter).unwrap().init_value(),
//             _ => unimplemented!(),
//         }
//     }
// }

pub struct Export<'a> {
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

pub struct Start {
    pub function_index: u32,
}

impl fmt::Debug for Start {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Start index={:?}>", indent, self.function_index)?;
        })
    }
}

pub struct Element {
    pub index: u32,
    // pub table_index: u32,
    // pub offset_opcode: u8,
    // pub offset_parameter: u32,a
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Element>", indent)?;
        })
    }
}

pub struct Body<'a> {
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

pub struct Data<'a> {
    pub index: u32,
    pub memory_index: u32,
    pub offset_opcode: u8,
    pub offset_parameter: u32,
    pub data: &'a [u8],
}


impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Data/>", indent)?;
        })
    }
}

pub struct SectionIter<'a> {
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
            Some(Section { index, section_type, buf })
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


pub struct ImportIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for ImportIter<'a> {
    type Item = Import<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let module = self.buf.slice_identifier();
            let export = self.buf.slice_identifier();
            let external_index = self.buf.read_u32();
            self.index += 1;
            Some(Import { index, module, export, external_index })
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

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            self.index += 1;
            Some(Function { index, signature_type_index: self.buf.read_u32() })
        } else {
            None
        }
    }
}

pub struct TableIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for TableIter<'a> {
    type Item = Table;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let element_type = self.buf.read_type();
            let limits = self.buf.read_limits();
            self.index += 1;
            Some(Table { index, element_type, limits })
        } else {
            None
        }
    }
}


pub struct MemoryIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for MemoryIter<'a> {
    type Item = Memory;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            let limits = self.buf.read_limits();
            self.index += 1;
            Some(Memory { index, limits })
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
            let global_type = TypeValue::from(self.buf.read_i8());
            let mutability = self.buf.read_u8();
            let opcode = self.buf.read_u8();
            let immediate = self.buf.read_u32();
            self.index += 1;
            Some(Global { index, global_type, mutability, opcode, immediate })
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


pub struct ElementIter<'a> {
    index: u32,
    buf: Cursor<'a>,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let index = self.index;
            self.index += 1;
            Some(Element { index })
        } else {
            None
        }
    }
}

pub struct BodyIter<'a> {
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
            Some(Body { index, buf })
        } else {
            None
        }
    }
}

pub struct DataIter<'a> {
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
            Some(Data { index, memory_index, offset_opcode, offset_parameter, data })
        } else {
            None
        }
    }
}

trait ModuleRead {
    fn read_type(&mut self) -> TypeValue;
    fn read_limits(&mut self) -> ResizableLimits;
}

impl<'a> ModuleRead for Cursor<'a> {
    fn read_type(&mut self) -> TypeValue {
        TypeValue::from(self.read_i8())
    }
    fn read_limits(&mut self) -> ResizableLimits {
        let flags = self.read_u32();
        let min = self.read_u32();
        let max = match flags {
            0 => None,
            1 => Some(self.read_u32()),
            _ => panic!("Unexpected Flags"),
        };
        ResizableLimits { flags, min, max }
    }
}