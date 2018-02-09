
pub struct Identifier(&'a [u8]);

pub struct TypeIndex(u32);
pub struct FuncIndex(u32);
pub struct TableIndex(u32);
pub struct MemIndex(u32);
pub struct GlobalIndex(u32);
pub struct LocalIndex(u32);
pub struct LablelIndex(u32);

pub enum ExternalIndex {
    Func(FuncIndex),
    Table(TableIndex),
    Mem(MemIndex),
    Global(GlobalIndex),
}

pub struct ResizableLimits {
    flags: u32,
    min: u32,
    max: Option<u32>,
}

pub struct Initializer {
    opcode: u8,
    immediate: u32,
    end: u8,
}