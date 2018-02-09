
pub struct Identifier<'a>(pub &'a [u8]);

pub struct TypeIndex(pub u32);
pub struct FuncIndex(pub u32);
pub struct TableIndex(pub u32);
pub struct MemIndex(pub u32);
pub struct GlobalIndex(pub u32);
pub struct LocalIndex(pub u32);
pub struct LablelIndex(pub u32);

pub enum ExternalIndex {
    Func(FuncIndex),
    Table(TableIndex),
    Mem(MemIndex),
    Global(GlobalIndex),
}

pub struct ResizableLimits {
    pub flags: u32,
    pub min: u32,
    pub max: Option<u32>,
}

pub struct Initializer {
    pub opcode: u8,
    pub immediate: u32,
    pub end: u8,
}