use SectionType;
use cursor::Cursor;

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
            let section_type = SectionType::from(self.buf.read_var_u7());
            let size = self.buf.read_var_u32();            
            let buf = self.buf.split(size as usize);            
            let section_header = SectionHeader { section_type, buf };
            Some(match section_type {
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
#[derive(Debug)]
pub struct ImportSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct FunctionSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct TableSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct MemorySection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct GlobalSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct ExportSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct StartSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct ElementSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct CodeSection<'a> {
    pub section_header: SectionHeader<'a>
}
#[derive(Debug)]
pub struct DataSection<'a> {
    pub section_header: SectionHeader<'a>
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
            let header = section.section_header;
            assert_eq!(header.section_type, SectionType::Type);            
            assert_eq!(header.buf.pos(), 0x0a);
            assert_eq!(header.buf.len(), 0x05);
        } else {
            panic!("Unexpected Section Type: {:?}", section)
        }

        let section = sections.next().unwrap();
        if let Section::Function(section) = section {
            let header = section.section_header;
            assert_eq!(header.section_type, SectionType::Function);
            assert_eq!(header.buf.pos(), 0x11);
            assert_eq!(header.buf.len(), 0x02);
        } else {
            panic!("Unexpected Section Type: {:?}", section)
        }        

        let section = sections.next().unwrap();
        if let Section::Export(section) = section {
            let header = section.section_header;
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