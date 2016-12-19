pub enum Section<'a> {
    Name(NameSection<'a>),
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

pub struct NameSection<'a>(pub &'a [u8]);
pub struct TypeSection<'a>(pub &'a [u8]);
pub struct ImportSection<'a>(pub &'a [u8]);
pub struct FunctionSection<'a>(pub &'a [u8]);
pub struct TableSection<'a>(pub &'a [u8]);
pub struct MemorySection<'a>(pub &'a [u8]);
pub struct GlobalSection<'a>(pub &'a [u8]);
pub struct ExportSection<'a>(pub &'a [u8]);
pub struct StartSection<'a>(pub &'a [u8]);
pub struct ElementSection<'a>(pub &'a [u8]);
pub struct CodeSection<'a>(pub &'a [u8]);
pub struct DataSection<'a>(pub &'a [u8]);

impl<'a> Section<'a> {
    pub fn id(&self) -> u8 {
        use self::Section::*;
        match self {
            &Name{..} => 0,
            &Type{..} => 1,
            &Import{..} => 2,
            &Function{..} => 3,
            &Table{..} => 4,
            &Memory{..} => 5,
            &Global{..} => 6,
            &Export{..} => 7,
            &Start{..} => 8,
            &Element{..} => 9,
            &Code{..} => 10,
            &Data{..} => 11,
        }
    }
}