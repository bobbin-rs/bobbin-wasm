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
pub struct LablelIndex(pub u32);

#[derive(Debug)]
pub enum ExternalIndex {
    Func(FuncIndex),
    Table(TableIndex),
    Mem(MemIndex),
    Global(GlobalIndex),
}

#[derive(Debug)]
pub struct ResizableLimits {
    pub flags: u32,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug)]
pub struct Initializer {
    pub opcode: u8,
    pub immediate: u32,
    pub end: u8,
}