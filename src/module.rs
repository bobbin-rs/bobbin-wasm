use {SectionType};
use cursor::Cursor;
use opcode::*;
use types::*;
use wasm_read::WasmRead;

use core::ops::Range; 
use core::convert::TryFrom;
use core::fmt;

pub struct FixupCount(usize);
pub struct FixupLen(usize);
pub struct FixupPos(usize);

pub struct Module<'a> {
    buf: Cursor<'a>,
    magic: u32,
    version: u32,
}

impl<'a> Module<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, Error> {
        let mut buf = Cursor::new(buf);        
        let magic = buf.read_u32();
        let version = buf.read_u32();
        Ok(Module { buf, magic, version })
    }
}

impl<'a> AsRef<[u8]> for Module<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf.as_ref()
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

    pub fn section(&self, section_type: SectionType) -> Option<Section> {
        self.sections().find(|s| s.section_type() == section_type)
    }

    pub fn type_section(&self) -> Option<TypeSection> {
        if let Some(Section::Type(section)) = self.section(SectionType::Type) {
            Some(section)
        } else {
            None
        }
    }

    pub fn import_section(&self) -> Option<ImportSection> {
        if let Some(Section::Import(section)) = self.section(SectionType::Import) {
            Some(section)
        } else {
            None
        }
    }

    pub fn function_section(&self) -> Option<FunctionSection> {
        if let Some(Section::Function(section)) = self.section(SectionType::Function) {
            Some(section)
        } else {
            None
        }
    }

    pub fn table_section(&self) -> Option<TableSection> {
        if let Some(Section::Table(section)) = self.section(SectionType::Table) {
            Some(section)
        } else {
            None
        }
    }

    pub fn memory_section(&self) -> Option<MemorySection> {
        if let Some(Section::Memory(section)) = self.section(SectionType::Memory) {
            Some(section)
        } else {
            None
        }
    }

    pub fn global_section(&self) -> Option<GlobalSection> {
        if let Some(Section::Global(section)) = self.section(SectionType::Global) {
            Some(section)
        } else {
            None
        }
    }    

    pub fn export_section(&self) -> Option<ExportSection> {
        if let Some(Section::Export(section)) = self.section(SectionType::Export) {
            Some(section)
        } else {
            None
        }
    }    

    pub fn start_section(&self) -> Option<StartSection> {
        if let Some(Section::Start(section)) = self.section(SectionType::Start) {
            Some(section)
        } else {
            None
        }
    }    

    pub fn element_section(&self) -> Option<ElementSection> {
        if let Some(Section::Element(section)) = self.section(SectionType::Element) {
            Some(section)
        } else {
            None
        }
    }    
                
    pub fn code_section(&self) -> Option<CodeSection> {
        if let Some(Section::Code(section)) = self.section(SectionType::Code) {
            Some(section)
        } else {
            None
        }
    }

    pub fn data_section(&self) -> Option<DataSection> {
        if let Some(Section::Data(section)) = self.section(SectionType::Data) {
            Some(section)
        } else {
            None
        }
    }    

    pub fn global(&self, index: usize) -> Option<Global> {
        if let Some(global_section) = self.global_section() {
            return global_section.iter().nth(index)
        }
        None
    }
    
    pub fn signature_type(&self, index: usize) -> Option<Signature> {
        if let Some(type_section) = self.type_section() {
            return type_section.iter().nth(index)
        }
        None
    }

    pub fn function_signature_type(&self, index: usize) -> Option<Signature> {
        if let Some(function_section) = self.function_section() {
            if let Some(function) = function_section.iter().nth(index) {
                if let Some(type_section) = self.type_section() {
                    return type_section.iter().nth(function as usize)
                }
            }
        }
        None
    }

    // pub fn instantiate<'buf, 'mem>(self, buf: &'buf mut [u8], memory: &'mem MemoryInst<'mem>) -> Result<(ModuleInst<'buf, 'mem>, &'buf mut [u8]), Error> {
    //     ModuleInst::new(buf, self, memory)
    // }
    
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

impl<'a> Section<'a> {
    pub fn section_type(&self) -> SectionType {
        self.header().section_type
    }

    pub fn header(&self) -> &SectionHeader {
        match self {
            &Section::Custom(ref s) => &s.section_header,
            &Section::Type(ref s) => &s.section_header,
            &Section::Import(ref s) => &s.section_header,
            &Section::Function(ref s) => &s.section_header,
            &Section::Table(ref s) => &s.section_header,
            &Section::Memory(ref s) => &s.section_header,
            &Section::Global(ref s) => &s.section_header,
            &Section::Export(ref s) => &s.section_header,
            &Section::Start(ref s) => &s.section_header,
            &Section::Element(ref s) => &s.section_header,
            &Section::Code(ref s) => &s.section_header,
            &Section::Data(ref s) => &s.section_header,
        }
    }
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
    pub fn form(&self) -> ValueType {
        ValueType::from(self.section_header.body().read_var_u7())
    }

    pub fn iter(&self) -> SignatureIter<'a> {
        SignatureIter { buf: self.section_header.body() }
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
    type Item = Import<'a>;

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
    type Item = Index;

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
    type Item = TableType;

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
    type Item = MemoryType;

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
    type Item = Global;

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
    type Item = Export<'a>;

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
    type Item = Element<'a>;

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
    type Item = Body<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > 0 {            
            Some(self.buf.read_body())
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Body<'a> {
    pub range: Range<u32>,
    pub locals: Cursor<'a>,
    pub expr: Cursor<'a>,
}

impl<'a> Body<'a> {
    pub fn locals(&self) -> LocalIter {
        LocalIter { buf: self.locals.clone() }
    }
    pub fn iter(&self) -> InstrIter<'a> { 
        InstrIter { buf: self.expr.clone() }
    }
}

#[derive(Debug)]
pub struct Local {
    pub n: u32,
    pub t: ValueType,
}

pub struct LocalIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for LocalIter<'a> {
    type Item = Local;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.buf.len() > 0 {
            Some(self.buf.read_local())
        } else {
            None
        }
    }
}

pub struct InstrIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for InstrIter<'a> {
    type Item = Instr<'a>;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.buf.len() > 0 {
            Some(self.buf.read_instr())
        } else {
            None
        }
    }
}

pub struct Instr<'a> { 
    pub range: Range<u32>,
    pub opcode: u8,
    pub imm: Immediate<'a> 
}

impl<'a> fmt::Debug for Instr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op = Opcode::try_from(self.opcode).unwrap();
        write!(f, "{:08x}: {}{:?}", self.range.start, op.text, self.imm)
    }
}

pub struct DataIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for DataIter<'a> {
    type Item = Data<'a>;

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
    pub form: ValueType,
    pub parameters: &'a [ValueType],
    pub returns: &'a [ValueType],
}

impl<'a> Signature<'a> {
    pub fn form(&self) -> ValueType { 
        self.form
    }

    pub fn parameters(&self) -> &[ValueType] {
        self.parameters
    }

    pub fn returns(&self) -> &[ValueType] {
        self.returns
    }
    
    // pub fn parameters(&self) -> ValueTypeIter {
    //     ValueTypeIter { buf: Cursor::new(self.parameters) }
    // }

    // pub fn returns(&self) -> ValueTypeIter {
    //     ValueTypeIter { buf: Cursor::new(self.returns) }
    // }

    // pub fn return_type(&self) -> Option<ValueType> {
    //     self.returns().nth(0).map(|t| ValueType::from(t))
    // }
}

impl<'a> fmt::Display for Signature<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        for (i, p) in self.parameters.iter().enumerate() {
            if i != 0 { write!(f, ", ")?; }
            write!(f,"{}", p)?;
        }
        write!(f, ") -> ")?;
        if self.returns.len() == 0 {
            write!(f, "nil")?;
        } else {
            for (i, r) in self.returns.iter().enumerate() {
                if i != 0 { write!(f, ", ")?; }
                write!(f,"{}", r)?;
            }            
        }
        Ok(())
    }
}

pub struct ValueTypeIter<'a> {
    buf: Cursor<'a>,
}

impl<'a> Iterator for ValueTypeIter<'a> {
    type Item = ValueType;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.buf.len() > 0 {
            Some(self.buf.read_type_value())
        } else {
            None
        }
    }
}


// #[cfg(test)]
// mod test {
//     use super::*;
//     use MAGIC_COOKIE;

//     #[test]
//     fn test_basic() {
//         let basic = include_bytes!("../local_test/basic.wasm");
//         let m = Module::from(&basic[..]);
//         assert_eq!(m.magic(), MAGIC_COOKIE);
//         assert_eq!(m.version(), 0x1);

//         let mut sections = m.sections();
        
//         let section = sections.next().unwrap();
//         if let Section::Type(section) = section {
//             let header = &section.section_header;
//             assert_eq!(header.section_type, SectionType::Type);            
//             assert_eq!(header.buf.pos(), 0x0a);
//             assert_eq!(header.buf.len(), 0x05);
//             let sig = section.iter().nth(0).unwrap();
//             assert_eq!(sig.parameters(), &[]);
//             assert_eq!(sig.returns(), &[0x7f]);

//         } else {
//             panic!("Unexpected Section Type: {:?}", section)
//         }

//         let section = sections.next().unwrap();
//         if let Section::Function(section) = section {
//             let header = &section.section_header;
//             assert_eq!(header.section_type, SectionType::Function);
//             assert_eq!(header.buf.pos(), 0x11);
//             assert_eq!(header.buf.len(), 0x02);            
//             let func = section.iter().nth(0).unwrap();
//             assert_eq!(func.signature_type_index, 0);
//         } else {
//             panic!("Unexpected Section Type: {:?}", section)
//         }        

//         let section = sections.next().unwrap();
//         if let Section::Export(section) = section {
//             let header = &section.section_header;
//             assert_eq!(header.section_type, SectionType::Export);
//             assert_eq!(header.buf.pos(), 0x15);
//             assert_eq!(header.buf.len(), 0x08);
//         } else {
//             panic!("Unexpected Section Type: {:?}", section)
//         }        

//         let section = sections.next().unwrap();
//         if let Section::Code(section) = section {
//             let header = section.section_header;
//             assert_eq!(header.section_type, SectionType::Code);
//             assert_eq!(header.buf.pos(), 0x1f);
//             assert_eq!(header.buf.len(), 0x07);
//         } else {
//             panic!("Unexpected Section Type: {:?}", section)
//         }        
//     }
// }