use SectionType;
use cursor::Cursor;

use core::fmt;

pub struct FixupCount(usize);
pub struct FixupLen(usize);
pub struct FixupPos(usize);

pub struct Module<'a> {
    buf: Cursor<'a>,
    magic: u32,
    version: u32,
}

impl<'a> From<&'a [u8]> for Module<'a> {
    fn from(buf: &'a [u8]) -> Self {
        let mut buf = Cursor::new(buf);        
        let magic = buf.read_u32();
        let version = buf.read_u32();
        Module { buf, magic, version }
    }
}

impl<'a> Module<'a> {
    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn version(&self) -> u32 {
       self.version
    }

    pub fn sections(&self) -> SectionIter {
        SectionIter { buf: self.buf.clone() }
    }
    
}

pub struct SectionIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            let section_header = self.buf.read_section_header();
            Some(match section_header.section_type {
                SectionType::Custom => Section::Custom(CustomSection { section_header }),
                SectionType::Type => Section::Type(TypeSection { section_header }),
                SectionType::Import => Section::Import(ImportSection { section_header }),
                SectionType::Function => Section::Function(FunctionSection { section_header }),
                SectionType::Table => Section::Table(TableSection { section_header }),
                SectionType::Memory => Section::Memory(MemorySection { section_header }),
                SectionType::Global => Section::Global(GlobalSection { section_header }),
                SectionType::Export => Section::Export(ExportSection { section_header }),
                SectionType::Start => Section::Start(StartSection { section_header }),
                SectionType::Element => Section::Element(ElementSection { section_header }),
                SectionType::Code => Section::Code(CodeSection { section_header }),
                SectionType::Data => Section::Data(DataSection { section_header }),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SectionHeader<'a> {
    pub section_type: SectionType,
    pub buf: Cursor<'a>,
}

impl<'a> SectionHeader<'a> {
    pub fn count(&self) -> u32 {
        self.buf.clone().read_var_u32()
    }
    pub fn body(&self) -> Cursor<'a> {
        let mut buf = self.buf.clone();
        buf.read_var_u32();
        buf.rest()
    }
}

#[derive(Debug)]
pub enum Section<'a> {
    Custom(CustomSection<'a>),
    Type(TypeSection<'a>),
    Import(ImportSection<'a>),
    Function(FunctionSection<'a>),
    Table(TableSection<'a>),
    Memory(MemorySection<'a>),
    Global(GlobalSection<'a>),
    Export(ExportSection<'a>),
    Start(StartSection<'a>),
    Element(ElementSection<'a>),
    Code(CodeSection<'a>),
    Data(DataSection<'a>),
}

#[derive(Debug)]
pub struct CustomSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct TypeSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> TypeSection<'a> {
    pub fn iter(&self) -> SignatureIter<'a> {
        let mut buf = self.section_header.body();
        let _form = buf.read_var_i32();
        SignatureIter { buf: buf }
    }    
}

#[derive(Debug)]
pub struct FunctionSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> FunctionSection<'a> {
    pub fn iter(&self) -> FunctionIter<'a> {
        FunctionIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct ImportSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> ImportSection<'a> {
    pub fn iter(&self) -> ImportIter<'a> {
        ImportIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct TableSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> TableSection<'a> {
    pub fn iter(&self) -> TableIter<'a> {
        TableIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct MemorySection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> MemorySection<'a> {
    pub fn iter(&self) -> MemoryIter<'a> {
        MemoryIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct GlobalSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> GlobalSection<'a> {
    pub fn iter(&self) -> GlobalIter<'a> {
        GlobalIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct ExportSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> ExportSection<'a> {    
    pub fn iter(&self) -> ExportIter<'a> {
        ExportIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct StartSection<'a> {
    pub section_header: SectionHeader<'a>
}

#[derive(Debug)]
pub struct ElementSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> ElementSection<'a> {    
    pub fn iter(&self) -> ElementIter<'a> {
        ElementIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct CodeSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> CodeSection<'a> {    
    pub fn iter(&self) -> CodeIter<'a> {
        CodeIter { buf: self.section_header.body() }
    }    
}

#[derive(Debug)]
pub struct DataSection<'a> {
    pub section_header: SectionHeader<'a>
}

impl<'a> DataSection<'a> {    
    pub fn iter(&self) -> DataIter<'a> {
        DataIter { buf: self.section_header.body() }
    }    
}


pub struct SignatureIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for SignatureIter<'a> {
    type Item = Signature<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_signature())
        } else {
            None
        }
    }
}


pub struct ImportIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for ImportIter<'a> {
    type Item = ::module::Import<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_import())
        } else {
            None
        }
    }
}

pub struct FunctionIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for FunctionIter<'a> {
    type Item = ::module::Function;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_function())
        } else {
            None
        }
    }
}

pub struct TableIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for TableIter<'a> {
    type Item = ::module::Table;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_table())
        } else {
            None
        }
    }
}


pub struct MemoryIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for MemoryIter<'a> {
    type Item = ::module::Memory;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_memory())
        } else {
            None
        }
    }
}


pub struct GlobalIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for GlobalIter<'a> {
    type Item = ::module::Global;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_global())
        } else {
            None
        }
    }
}


pub struct ExportIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for ExportIter<'a> {
    type Item = ::module::Export<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_export())
        } else {
            None
        }
    }
}


pub struct ElementIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = ::module::Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_element())
        } else {
            None
        }
    }
}

pub struct CodeIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for CodeIter<'a> {
    type Item = ::module::Body<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_body())
        } else {
            None
        }
    }
}

pub struct DataIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for DataIter<'a> {
    type Item = ::module::Data<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {
            Some(self.buf.read_data())
        } else {
            None
        }
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Signature<'a> {
    parameters: &'a [u8],
    returns: &'a [u8],
}

impl<'a> Signature<'a> {
    pub fn parameters(&self) -> &[u8] {
        self.parameters
    }

    pub fn returns(&self) -> &[u8] {
        self.returns
    }
}

impl<'a> fmt::Display for Signature<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        for (i, p) in self.parameters.iter().enumerate() {
            if i != 0 { write!(f, ", ")?; }
            write!(f,"{}", TypeValue::from(*p))?;
        }
        write!(f, ") -> ")?;
        if self.returns.len() == 0 {
            write!(f, "nil")?;
        } else {
            for (i, r) in self.returns.iter().enumerate() {
                if i != 0 { write!(f, ", ")?; }
                write!(f,"{}", TypeValue::from(*r))?;
            }            
        }
        Ok(())
    }
}

pub trait ModuleRead<'a> {
    fn read_identifier(&mut self) -> Identifier<'a>;
    fn read_initializer(&mut self) -> Initializer;
    fn read_section_type(&mut self) -> SectionType;
    fn read_signature(&mut self) -> Signature<'a>;
    fn read_type_value(&mut self) -> TypeValue;
    fn read_type_values(&mut self) -> &'a [u8];
    fn read_bytes(&mut self) -> &'a [u8];
    fn read_global_type(&mut self) -> GlobalType;
    fn read_limits(&mut self) -> Limits;
    fn read_section_header(&mut self) -> SectionHeader<'a>;
    fn read_type(&mut self) -> Type<'a>;
    fn read_function(&mut self) -> Function;
    fn read_table(&mut self) -> Table;
    fn read_memory(&mut self) -> Memory;
    fn read_import_desc(&mut self) -> ImportDesc;
    fn read_export_desc(&mut self) -> ExportDesc;
    fn read_data(&mut self) -> Data<'a>;
    fn read_element(&mut self) -> Element<'a>;
    fn read_global(&mut self) -> Global;
    fn read_export(&mut self) -> Export<'a>;
    fn read_import(&mut self) -> Import<'a>;
    fn read_body(&mut self) -> Body<'a>;
}

use {TypeValue};
use types::*;
use module::{GlobalType, Type, Function, Table, Memory, ImportDesc, ExportDesc, Data, Element, Global, Export, Import, Body};

impl<'a> ModuleRead<'a> for Cursor<'a> {
    fn read_identifier(&mut self) -> Identifier<'a> {
        let len = self.read_var_u32();        
        Identifier(self.slice(len as usize))
    }

    fn read_initializer(&mut self) -> Initializer {
        let opcode = self.read_u8();
        let immediate = self.read_i32();
        let end = self.read_u8();
        Initializer { opcode, immediate, end }
    }

    fn read_section_type(&mut self) -> SectionType {
        SectionType::from(self.read_var_u7())
    }

    fn read_signature(&mut self) -> Signature<'a> {
        let p_len = self.read_var_u32();
        let parameters = self.slice(p_len as usize);
        let r_len = self.read_var_u32();
        let returns = self.slice(r_len as usize);
        Signature { parameters, returns }
    }

    fn read_type_value(&mut self) -> TypeValue {
        TypeValue::from(self.read_var_u7())
    }

    fn read_type_values(&mut self) -> &'a [u8] {
        let data_len = self.read_var_u7();
        self.slice(data_len as usize)
    }

    fn read_bytes(&mut self) -> &'a [u8] {
        let data_len = self.read_var_u32();
        self.slice(data_len as usize)
    }

    fn read_global_type(&mut self) -> GlobalType {
        let type_value = self.read_type_value();
        let mutability = self.read_var_u7();
        GlobalType { type_value, mutability }
    }

    fn read_limits(&mut self) -> Limits {
        let flags = self.read_var_u32();
        let min = self.read_var_u32();
        let max = match flags {
            0 => None,
            1 => Some(self.read_var_u32()),
            _ => panic!("Unexpected Flags"),
        };
        Limits { flags, min, max }
    }

    fn read_section_header(&mut self) -> SectionHeader<'a> {
        let section_type = SectionType::from(self.read_var_u7());
        let size = self.read_var_u32();            
        let buf = self.split(size as usize);            
        SectionHeader { section_type, buf }        
    }

    fn read_type(&mut self) -> Type<'a> {
        let _form = self.read_var_i7();
        let parameters = self.read_type_values();
        let returns = self.read_type_values();
        Type { parameters, returns }
    }

    fn read_function(&mut self) -> Function {
        let signature_type_index = self.read_var_u32();
        Function { signature_type_index } 
    }

    fn read_table(&mut self) -> Table {
        let element_type = self.read_type_value();
        let limits = self.read_limits();
        Table { element_type, limits }
    }

    fn read_memory(&mut self) -> Memory {
        let limits = self.read_limits();
        Memory { limits }
    }

    fn read_import_desc(&mut self) -> ImportDesc {
        let kind = self.read_var_u7();
        match kind {
            0x00 => ImportDesc::Type(self.read_var_u32()),
            0x01 => ImportDesc::Table(self.read_table()),
            0x02 => ImportDesc::Memory(self.read_memory()),
            0x03 => ImportDesc::Global(self.read_global_type()),
            _ => panic!("Invalid import type: {:02x}", kind),
        }        
    }

    fn read_export_desc(&mut self) -> ExportDesc {
        let kind = self.read_var_u7();
        let index = self.read_var_u32();

        match kind {
            0x00 => ExportDesc::Function(index),
            0x01 => ExportDesc::Table(index),
            0x02 => ExportDesc::Memory(index),
            0x03 => ExportDesc::Global(index),
            _ => panic!("Invalid export type: {:02x}", kind),
        }
    }

    fn read_data(&mut self) -> Data<'a> {
        let memory_index = self.read_var_u32();
        let offset = self.read_initializer();
        let data = self.read_bytes();
        Data { memory_index, offset, data }
    }

    fn read_element(&mut self) -> Element<'a> {
        let table_index = self.read_var_u32();
        let offset = self.read_initializer();
        let data = self.read_bytes();
        Element { table_index, offset, data }
    }

    fn read_global(&mut self) -> Global {
        let global_type = self.read_global_type();
        let init = self.read_initializer();
        Global { global_type, init }
    }

    fn read_export(&mut self) -> Export<'a> {
        let identifier = self.read_identifier();
        let export_desc = self.read_export_desc();
        Export { identifier, export_desc }
    }

    fn read_import(&mut self) -> Import<'a> {
        let module = self.read_identifier();
        let export = self.read_identifier();
        let desc = self.read_import_desc();
        Import { module, export, desc }    
    }    

    fn read_body(&mut self) -> Body<'a> {
        let buf = self.read_bytes();
        Body { buf }
    }
    
}

#[cfg(test)]
mod test {
    use super::*;
    use MAGIC_COOKIE;

    

    #[test]
    fn test_basic() {
        let basic = include_bytes!("../../local_test/basic.wasm");
        let m = Module::from(&basic[..]);
        assert_eq!(m.magic(), MAGIC_COOKIE);
        assert_eq!(m.version(), 0x1);

        let mut sections = m.sections();
        
        let section = sections.next().unwrap();
        if let Section::Type(section) = section {
            let header = &section.section_header;
            assert_eq!(header.section_type, SectionType::Type);            
            assert_eq!(header.buf.pos(), 0x0a);
            assert_eq!(header.buf.len(), 0x05);
            let sig = section.iter().nth(0).unwrap();
            assert_eq!(sig.parameters(), &[]);
            assert_eq!(sig.returns(), &[0x7f]);

        } else {
            panic!("Unexpected Section Type: {:?}", section)
        }

        let section = sections.next().unwrap();
        if let Section::Function(section) = section {
            let header = &section.section_header;
            assert_eq!(header.section_type, SectionType::Function);
            assert_eq!(header.buf.pos(), 0x11);
            assert_eq!(header.buf.len(), 0x02);            
            let func = section.iter().nth(0).unwrap();
            assert_eq!(func.signature_type_index, 0);
        } else {
            panic!("Unexpected Section Type: {:?}", section)
        }        

        let section = sections.next().unwrap();
        if let Section::Export(section) = section {
            let header = &section.section_header;
            assert_eq!(header.section_type, SectionType::Export);
            assert_eq!(header.buf.pos(), 0x15);
            assert_eq!(header.buf.len(), 0x08);
        } else {
            panic!("Unexpected Section Type: {:?}", section)
        }        

        let section = sections.next().unwrap();
        if let Section::Code(section) = section {
            let header = section.section_header;
            assert_eq!(header.section_type, SectionType::Code);
            assert_eq!(header.buf.pos(), 0x1f);
            assert_eq!(header.buf.len(), 0x07);
        } else {
            panic!("Unexpected Section Type: {:?}", section)
        }        
    }
}