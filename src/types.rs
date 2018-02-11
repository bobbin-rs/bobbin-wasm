#[derive(Debug)]
pub struct Identifier<'a>(pub &'a [u8]);

#[derive(Debug)]
pub struct TypeIndex(pub u32);
#[derive(Debug)]
pub struct FuncIndex(pub u32);
#[derive(Debug)]
pub struct TableIndex(pub u32);
#[derive(Debug)]
pub struct MemIndex(pub u32);
#[derive(Debug)]
pub struct GlobalIndex(pub u32);
#[derive(Debug)]
pub struct LocalIndex(pub u32);
#[derive(Debug)]
pub struct LabelIndex(pub u32);

#[derive(Debug)]
pub enum ExternalIndex {
    Func(FuncIndex),
    Table(TableIndex),
    Mem(MemIndex),
    Global(GlobalIndex),
}

impl ExternalIndex {
    pub fn kind(&self) -> u8 {
        use ExternalIndex::*;
        match *self {
            Func(_) => 0x00,
            Table(_) => 0x01,
            Mem(_) => 0x02,
            Global(_) => 0x03,
        }
    }
    pub fn index(&self) -> u32 {
        use ExternalIndex::*;
        match *self {
            Func(FuncIndex(n)) => n,
            Table(TableIndex(n)) => n,
            Mem(MemIndex(n)) => n,
            Global(GlobalIndex(n)) => n,
        }        
    }
}

#[derive(Debug)]
pub struct Limits {
    pub flags: u32,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug)]
pub struct Initializer {
    pub opcode: u8,
    pub immediate: i32,
    pub end: u8,
}