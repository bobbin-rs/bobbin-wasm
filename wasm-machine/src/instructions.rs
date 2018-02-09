use types::*;
use opcode::Opcode;
use TypeValue;

pub type Depth = u32;
pub type BranchCount = u32;
pub struct MemArg { offset: u32, align: u32 };


pub enum Instruction {
    Block { op: Opcode, sig: TypeValue },
    Branch { op: Opcode, depth: Depth },
    BranchTableCount { op: Opcode, count: BranchCount },
    BranchTableDepth { op: Opcode, depth: Depth },
    BranchTableDefault { op: Opcode, depth: Depth },
    Local { op: Opcode, index: LocalIndex },
    Global { op: Opcode, index: GlobalIndex },
    Call { op: Opcode, index: FuncIndex },
    CallIndirect { op: Opcode, index: TypeIndex },
    I32Const { op: Opcode, value: i32 },
    F32Const { op: Opcode, value: f32 },
    I64Const { op: Opcode, value: i64 },
    F64Const { op: Opcode, value: f64 },
    I32LoadStore { op: Opcode, memarg: MemArg },
    F32LoadStore { op: Opcode, memarg: MemArg },
    I64LoadStore { op: Opcode, memarg: MemArg },
    F64LoadStore { op: Opcode, memarg: MemArg },
    Memory { op: Opcode, arg: reserved: u8 },
}