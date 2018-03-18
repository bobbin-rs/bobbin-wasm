use {TypeValue, FuncIndex, LocalIndex, GlobalIndex, TypeIndex};
use floathex;

use ::core::fmt;
use ::core::convert::TryFrom;


pub const BR_TABLE_ENTRY_SIZE: u32 = 12;

macro_rules! opcodes {
    ( $( ($tr:ident, $t1:ident, $t2:ident, $m:expr, $code:expr, $name:ident, $text:expr), )*) => {
        impl TryFrom<u8> for Opcode {
            type Error = Error;
            fn try_from(other: u8) -> Result<Self, Self::Error> {
                match other {
                    $(
                        $code => Ok(Opcode { code: $code, tr: $tr, t1: $t1, t2: $t2, m: $m, text: $text }),
                    )*
                    _ => Err(Error::InvalidOpCode(other)),
                }
            }
        }

        $(
            pub const $name: u8 = $code;
        )*
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidOpCode(u8),
}

pub type Depth = u8;
pub type BranchCount = u32;
// pub type CallIndirectCount = u32;

pub const ___: TypeValue = TypeValue::Void;
pub const I32: TypeValue = TypeValue::I32;
pub const I64: TypeValue = TypeValue::I64;
pub const F32: TypeValue = TypeValue::F32;
pub const F64: TypeValue = TypeValue::F64;
pub const ANYFUNC: TypeValue = TypeValue::AnyFunc;
pub const FUNC: TypeValue = TypeValue::Func;
pub const VOID: TypeValue = TypeValue::Void;

#[derive(Debug)]
pub struct Opcode {
    pub code: u8,
    pub tr: TypeValue,
    pub t1: TypeValue,
    pub t2: TypeValue,
    pub m: u8,
    pub text: &'static str,
}

impl Opcode {
    pub fn is_unop(&self) -> bool {
        self.t1 != TypeValue::Void && self.t2 == TypeValue::Void
    }

    pub fn is_binop(&self) -> bool {
        self.t1 != TypeValue::Void && self.t2 != TypeValue::Void
    }

    pub fn immediate_type(&self) -> ImmediateType {
        ImmediateType::from(self.code)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImmediateType {
    None,
    BlockSignature,
    BranchDepth,
    BranchTable,
    I32,
    I64,
    F32,
    F64,
    Local,
    Global,
    Call,
    CallIndirect,
    LoadStore,
    Memory,
}

impl From<u8> for ImmediateType {
    fn from(other: u8) -> ImmediateType {
        use self::ImmediateType::*;
        match other {
            BLOCK | LOOP | IF => BlockSignature,
            BR | BR_IF => BranchDepth,
            BR_TABLE => BranchTable,
            GET_LOCAL | SET_LOCAL | TEE_LOCAL => Local,
            GET_GLOBAL | SET_GLOBAL => Global,
            CALL => Call,
            CALL_INDIRECT => CallIndirect,

            I32_CONST => I32,
            F32_CONST => F32,
            I64_CONST => I64,
            F64_CONST => F64,

            I32_LOAD | I32_STORE |
            I32_LOAD8_S ... I32_LOAD16_U |
            I32_STORE8 ... I32_STORE16 => LoadStore,

            F32_LOAD | F32_STORE => LoadStore,

            I64_LOAD | I64_STORE |
            I64_LOAD8_S ... I64_LOAD32_U |
            I64_STORE8 ... I64_STORE32 => LoadStore,

            F64_LOAD | F64_STORE => LoadStore,
            
            MEM_SIZE | MEM_GROW => Memory,
            _ => None,
        }
    }
}

pub enum Immediate<'a> {
    None,
    Block { signature: TypeValue },
    Branch { depth: Depth },
    BranchTable { table: &'a [Depth] },
    BranchTableStart { count: BranchCount },
    BranchTableDepth { n: u32, depth: Depth },
    BranchTableDefault { depth: Depth },
    Local { index: LocalIndex },
    Global { index: GlobalIndex },
    Call { index: FuncIndex },
    CallIndirect { index: TypeIndex, reserved: u32 },
    I32Const { value: i32 },
    F32Const { value: f32 },
    I64Const { value: i64 },
    F64Const { value: f64 },
    LoadStore { align: u32, offset: u32 },
    Memory { reserved: u8 },
}


impl<'a> fmt::Debug for Immediate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Immediate::*;
        match *self {
            None => Ok(()),
            Block { signature } => if signature != TypeValue::Void {
                write!(f, " {}", signature)
            } else {
                Ok(())
            },
            Branch { depth } => write!(f, " {}", depth),
            BranchTable { table: _ } => {
                // write!(f, "[")?;
                // for (i, d) in table.iter().enumerate() {
                //     if i != 0 { write!(f, ", ")?; }
                //     write!(f, "{}", d)?;
                // }
                // write!(f, "]")?;
                Ok(())
            }
            BranchTableStart { count } => write!(f, " [{}]", count),
            BranchTableDepth { n: _, depth } => write!(f, " {}", depth),
            BranchTableDefault { depth } => write!(f, " {}", depth),
            Local { ref index } => write!(f, " {}", index.0),
            Global { ref index } => write!(f, " {}", index.0),
            Call { ref index } => write!(f, " {}", index.0),
            CallIndirect { ref index, reserved } => write!(f, " {} {}", index.0, reserved),
            I32Const { value } => write!(f, " {}", value),
            F32Const { value } => {
                write!(f, " ")?;
                floathex::f32_hex(f, value)
            },
            I64Const { value } => write!(f, " {}", value),
            F64Const { value } => {
                write!(f, " ")?;
                floathex::f64_hex(f, value)
            },
            LoadStore { align, offset } => write!(f, " {} {}", align, offset),
            Memory { reserved: _ } => Ok(()),
        }

    }
}

pub struct Instruction<'a> { 
    pub offset: u32, 
    pub data: &'a [u8], 
    pub op: &'a Opcode, 
    pub imm: Immediate<'a>
}

// impl<'a> WriteTo for Instruction<'a> {
//     fn write_to(&self, w: &mut Writer) -> WasmResult<()> {

//         Ok(())
//     }
// }

impl<'a> fmt::Debug for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08x}: {:?} {:?}", self.offset, self.op.text, self.imm)
    }
}

/*
 *   tr: result type
 *   t1: type of the 1st parameter
 *   t2: type of the 2nd parameter
 *    m: memory size of the operation, if any
 * code: opcode
 * NAME: used to generate the opcode enum
 * text: a string of the opcode name in the AST format
 *
 *  tr  t1    t2   m  code  NAME text
 *  ============================ */
opcodes!{
  (___, ___, ___, 0, 0x00, UNREACHABLE, "unreachable"),
  (___, ___, ___, 0, 0x01, NOP, "nop"),
  (___, ___, ___, 0, 0x02, BLOCK, "block"),
  (___, ___, ___, 0, 0x03, LOOP, "loop"),
  (___, ___, ___, 0, 0x04, IF, "if"),
  (___, ___, ___, 0, 0x05, ELSE, "else"),
  (___, ___, ___, 0, 0x0b, END, "end"),
  (___, ___, ___, 0, 0x0c, BR, "br"),
  (___, ___, ___, 0, 0x0d, BR_IF, "br_if"),
  (___, ___, ___, 0, 0x0e, BR_TABLE, "br_table"),
  (___, ___, ___, 0, 0x0f, RETURN, "return"),
  (___, ___, ___, 0, 0x10, CALL, "call"),
  (___, ___, ___, 0, 0x11, CALL_INDIRECT, "call_indirect"),
  (___, ___, ___, 0, 0x1a, DROP, "drop"),
  (___, ___, ___, 0, 0x1b, SELECT, "select"),
  (___, ___, ___, 0, 0x20, GET_LOCAL, "get_local"),
  (___, ___, ___, 0, 0x21, SET_LOCAL, "set_local"),
  (___, ___, ___, 0, 0x22, TEE_LOCAL, "tee_local"),
  (___, ___, ___, 0, 0x23, GET_GLOBAL, "get_global"),
  (___, ___, ___, 0, 0x24, SET_GLOBAL, "set_global"),
  (I32, I32, ___, 4, 0x28, I32_LOAD, "i32.load"),
  (I64, I32, ___, 8, 0x29, I64_LOAD, "i64.load"),
  (F32, I32, ___, 4, 0x2a, F32_LOAD, "f32.load"),
  (F64, I32, ___, 8, 0x2b, F64_LOAD, "f64.load"),
  (I32, I32, ___, 1, 0x2c, I32_LOAD8_S, "i32.load8_s"),
  (I32, I32, ___, 1, 0x2d, I32_LOAD8_U, "i32.load8_u"),
  (I32, I32, ___, 2, 0x2e, I32_LOAD16_S, "i32.load16_s"),
  (I32, I32, ___, 2, 0x2f, I32_LOAD16_U, "i32.load16_u"),
  (I64, I32, ___, 1, 0x30, I64_LOAD8_S, "i64.load8_s"),
  (I64, I32, ___, 1, 0x31, I64_LOAD8_U, "i64.load8_u"),
  (I64, I32, ___, 2, 0x32, I64_LOAD16_S, "i64.load16_s"),
  (I64, I32, ___, 2, 0x33, I64_LOAD16_U, "i64.load16_u"),
  (I64, I32, ___, 4, 0x34, I64_LOAD32_S, "i64.load32_s"),
  (I64, I32, ___, 4, 0x35, I64_LOAD32_U, "i64.load32_u"),
  (___, I32, I32, 4, 0x36, I32_STORE, "i32.store"),
  (___, I32, I64, 8, 0x37, I64_STORE, "i64.store"),
  (___, I32, F32, 4, 0x38, F32_STORE, "f32.store"),
  (___, I32, F64, 8, 0x39, F64_STORE, "f64.store"),
  (___, I32, I32, 1, 0x3a, I32_STORE8, "i32.store8"),
  (___, I32, I32, 2, 0x3b, I32_STORE16, "i32.store16"),
  (___, I32, I64, 1, 0x3c, I64_STORE8, "i64.store8"),
  (___, I32, I64, 2, 0x3d, I64_STORE16, "i64.store16"),
  (___, I32, I64, 4, 0x3e, I64_STORE32, "i64.store32"),
  (I32, ___, ___, 0, 0x3f, MEM_SIZE, "mem_size"),
  (I32, I32, ___, 0, 0x40, MEM_GROW, "mem_grow"),
  (I32, ___, ___, 0, 0x41, I32_CONST, "i32.const"),
  (I64, ___, ___, 0, 0x42, I64_CONST, "i64.const"),
  (F32, ___, ___, 0, 0x43, F32_CONST, "f32.const"),
  (F64, ___, ___, 0, 0x44, F64_CONST, "f64.const"),
  (I32, I32, ___, 0, 0x45, I32_EQZ, "i32.eqz"),
  (I32, I32, I32, 0, 0x46, I32_EQ, "i32.eq"),
  (I32, I32, I32, 0, 0x47, I32_NE, "i32.ne"),
  (I32, I32, I32, 0, 0x48, I32_LT_S, "i32.lt_s"),
  (I32, I32, I32, 0, 0x49, I32_LT_U, "i32.lt_u"),
  (I32, I32, I32, 0, 0x4a, I32_GT_S, "i32.gt_s"),
  (I32, I32, I32, 0, 0x4b, I32_GT_U, "i32.gt_u"),
  (I32, I32, I32, 0, 0x4c, I32_LE_S, "i32.le_s"),
  (I32, I32, I32, 0, 0x4d, I32_LE_U, "i32.le_u"),
  (I32, I32, I32, 0, 0x4e, I32_GE_S, "i32.ge_s"),
  (I32, I32, I32, 0, 0x4f, I32_GE_U, "i32.ge_u"),
  (I32, I64, ___, 0, 0x50, I64_EQZ, "i64.eqz"),
  (I32, I64, I64, 0, 0x51, I64_EQ, "i64.eq"),
  (I32, I64, I64, 0, 0x52, I64_NE, "i64.ne"),
  (I32, I64, I64, 0, 0x53, I64_LT_S, "i64.lt_s"),
  (I32, I64, I64, 0, 0x54, I64_LT_U, "i64.lt_u"),
  (I32, I64, I64, 0, 0x55, I64_GT_S, "i64.gt_s"),
  (I32, I64, I64, 0, 0x56, I64_GT_U, "i64.gt_u"),
  (I32, I64, I64, 0, 0x57, I64_LE_S, "i64.le_s"),
  (I32, I64, I64, 0, 0x58, I64_LE_U, "i64.le_u"),
  (I32, I64, I64, 0, 0x59, I64_GE_S, "i64.ge_s"),
  (I32, I64, I64, 0, 0x5a, I64_GE_U, "i64.ge_u"),
  (I32, F32, F32, 0, 0x5b, F32_EQ, "f32.eq"),
  (I32, F32, F32, 0, 0x5c, F32_NE, "f32.ne"),
  (I32, F32, F32, 0, 0x5d, F32_LT, "f32.lt"),
  (I32, F32, F32, 0, 0x5e, F32_GT, "f32.gt"),
  (I32, F32, F32, 0, 0x5f, F32_LE, "f32.le"),
  (I32, F32, F32, 0, 0x60, F32_GE, "f32.ge"),
  (I32, F64, F64, 0, 0x61, F64_EQ, "f64.eq"),
  (I32, F64, F64, 0, 0x62, F64_NE, "f64.ne"),
  (I32, F64, F64, 0, 0x63, F64_LT, "f64.lt"),
  (I32, F64, F64, 0, 0x64, F64_GT, "f64.gt"),
  (I32, F64, F64, 0, 0x65, F64_LE, "f64.le"),
  (I32, F64, F64, 0, 0x66, F64_GE, "f64.ge"),
  (I32, I32, ___, 0, 0x67, I32_CLZ, "i32.clz"),
  (I32, I32, ___, 0, 0x68, I32_CTZ, "i32.ctz"),
  (I32, I32, ___, 0, 0x69, I32_POPCNT, "i32.popcnt"),
  (I32, I32, I32, 0, 0x6a, I32_ADD, "i32.add"),
  (I32, I32, I32, 0, 0x6b, I32_SUB, "i32.sub"),
  (I32, I32, I32, 0, 0x6c, I32_MUL, "i32.mul"),
  (I32, I32, I32, 0, 0x6d, I32_DIV_S, "i32.div_s"),
  (I32, I32, I32, 0, 0x6e, I32_DIV_U, "i32.div_u"),
  (I32, I32, I32, 0, 0x6f, I32_REM_S, "i32.rem_s"),
  (I32, I32, I32, 0, 0x70, I32_REM_U, "i32.rem_u"),
  (I32, I32, I32, 0, 0x71, I32_AND, "i32.and"),
  (I32, I32, I32, 0, 0x72, I32_OR, "i32.or"),
  (I32, I32, I32, 0, 0x73, I32_XOR, "i32.xor"),
  (I32, I32, I32, 0, 0x74, I32_SHL, "i32.shl"),
  (I32, I32, I32, 0, 0x75, I32_SHR_S, "i32.shr_s"),
  (I32, I32, I32, 0, 0x76, I32_SHR_U, "i32.shr_u"),
  (I32, I32, I32, 0, 0x77, I32_ROTL, "i32.rotl"),
  (I32, I32, I32, 0, 0x78, I32_ROTR, "i32.rotr"),
  (I64, I64, I64, 0, 0x79, I64_CLZ, "i64.clz"),
  (I64, I64, I64, 0, 0x7a, I64_CTZ, "i64.ctz"),
  (I64, I64, I64, 0, 0x7b, I64_POPCNT, "i64.popcnt"),
  (I64, I64, I64, 0, 0x7c, I64_ADD, "i64.add"),
  (I64, I64, I64, 0, 0x7d, I64_SUB, "i64.sub"),
  (I64, I64, I64, 0, 0x7e, I64_MUL, "i64.mul"),
  (I64, I64, I64, 0, 0x7f, I64_DIV_S, "i64.div_s"),
  (I64, I64, I64, 0, 0x80, I64_DIV_U, "i64.div_u"),
  (I64, I64, I64, 0, 0x81, I64_REM_S, "i64.rem_s"),
  (I64, I64, I64, 0, 0x82, I64_REM_U, "i64.rem_u"),
  (I64, I64, I64, 0, 0x83, I64_AND, "i64.and"),
  (I64, I64, I64, 0, 0x84, I64_OR, "i64.or"),
  (I64, I64, I64, 0, 0x85, I64_XOR, "i64.xor"),
  (I64, I64, I64, 0, 0x86, I64_SHL, "i64.shl"),
  (I64, I64, I64, 0, 0x87, I64_SHR_S, "i64.shr_s"),
  (I64, I64, I64, 0, 0x88, I64_SHR_U, "i64.shr_u"),
  (I64, I64, I64, 0, 0x89, I64_ROTL, "i64.rotl"),
  (I64, I64, I64, 0, 0x8a, I64_ROTR, "i64.rotr"),
  (F32, F32, F32, 0, 0x8b, F32_ABS, "f32.abs"),
  (F32, F32, F32, 0, 0x8c, F32_NEG, "f32.neg"),
  (F32, F32, F32, 0, 0x8d, F32_CEIL, "f32.ceil"),
  (F32, F32, F32, 0, 0x8e, F32_FLOOR, "f32.floor"),
  (F32, F32, F32, 0, 0x8f, F32_TRUNC, "f32.trunc"),
  (F32, F32, F32, 0, 0x90, F32_NEAREST, "f32.nearest"),
  (F32, F32, F32, 0, 0x91, F32_SQRT, "f32.sqrt"),
  (F32, F32, F32, 0, 0x92, F32_ADD, "f32.add"),
  (F32, F32, F32, 0, 0x93, F32_SUB, "f32.sub"),
  (F32, F32, F32, 0, 0x94, F32_MUL, "f32.mul"),
  (F32, F32, F32, 0, 0x95, F32_DIV, "f32.div"),
  (F32, F32, F32, 0, 0x96, F32_MIN, "f32.min"),
  (F32, F32, F32, 0, 0x97, F32_MAX, "f32.max"),
  (F32, F32, F32, 0, 0x98, F32_COPYSIGN, "f32.copysign"),
  (F64, F64, F64, 0, 0x99, F64_ABS, "f64.abs"),
  (F64, F64, F64, 0, 0x9a, F64_NEG, "f64.neg"),
  (F64, F64, F64, 0, 0x9b, F64_CEIL, "f64.ceil"),
  (F64, F64, F64, 0, 0x9c, F64_FLOOR, "f64.floor"),
  (F64, F64, F64, 0, 0x9d, F64_TRUNC, "f64.trunc"),
  (F64, F64, F64, 0, 0x9e, F64_NEAREST, "f64.nearest"),
  (F64, F64, F64, 0, 0x9f, F64_SQRT, "f64.sqrt"),
  (F64, F64, F64, 0, 0xa0, F64_ADD, "f64.add"),
  (F64, F64, F64, 0, 0xa1, F64_SUB, "f64.sub"),
  (F64, F64, F64, 0, 0xa2, F64_MUL, "f64.mul"),
  (F64, F64, F64, 0, 0xa3, F64_DIV, "f64.div"),
  (F64, F64, F64, 0, 0xa4, F64_MIN, "f64.min"),
  (F64, F64, F64, 0, 0xa5, F64_MAX, "f64.max"),
  (F64, F64, F64, 0, 0xa6, F64_COPYSIGN, "f64.copysign"),
  (I32, I64, ___, 0, 0xa7, I32_WRAP_I64, "i32.wrap/i64"),
  (I32, F32, ___, 0, 0xa8, I32_TRUNC_S_F32, "i32.trunc_s/f32"),
  (I32, F32, ___, 0, 0xa9, I32_TRUNC_U_F32, "i32.trunc_u/f32"),
  (I32, F64, ___, 0, 0xaa, I32_TRUNC_S_F64, "i32.trunc_s/f64"),
  (I32, F64, ___, 0, 0xab, I32_TRUNC_U_F64, "i32.trunc_u/f64"),
  (I64, I32, ___, 0, 0xac, I64_EXTEND_S_I32, "i64.extend_s/i32"),
  (I64, I32, ___, 0, 0xad, I64_EXTEND_U_I32, "i64.extend_u/i32"),
  (I64, F32, ___, 0, 0xae, I64_TRUNC_S_F32, "i64.trunc_s/f32"),
  (I64, F32, ___, 0, 0xaf, I64_TRUNC_U_F32, "i64.trunc_u/f32"),
  (I64, F64, ___, 0, 0xb0, I64_TRUNC_S_F64, "i64.trunc_s/f64"),
  (I64, F64, ___, 0, 0xb1, I64_TRUNC_U_F64, "i64.trunc_u/f64"),
  (F32, I32, ___, 0, 0xb2, F32_CONVERT_S_I32, "f32.convert_s/i32"),
  (F32, I32, ___, 0, 0xb3, F32_CONVERT_U_I32, "f32.convert_u/i32"),
  (F32, I64, ___, 0, 0xb4, F32_CONVERT_S_I64, "f32.convert_s/i64"),
  (F32, I64, ___, 0, 0xb5, F32_CONVERT_U_I64, "f32.convert_u/i64"),
  (F32, F64, ___, 0, 0xb6, F32_DEMOTE_F64, "f32.demote/f64"),
  (F64, I32, ___, 0, 0xb7, F64_CONVERT_S_I32, "f64.convert_s/i32"),
  (F64, I32, ___, 0, 0xb8, F64_CONVERT_U_I32, "f64.convert_u/i32"),
  (F64, I64, ___, 0, 0xb9, F64_CONVERT_S_I64, "f64.convert_s/i64"),
  (F64, I64, ___, 0, 0xba, F64_CONVERT_U_I64, "f64.convert_u/i64"),
  (F64, F32, ___, 0, 0xbb, F64_PROMOTE_F32, "f64.promote/f32"),
  (I32, F32, ___, 0, 0xbc, I32_REINTERPRET_F32, "i32.reinterpret/f32"),
  (I64, F64, ___, 0, 0xbd, I64_REINTERPRET_F64, "i64.reinterpret/f64"),
  (F32, I32, ___, 0, 0xbe, F32_REINTERPRET_I32, "f32.reinterpret/i32"),
  (F64, I64, ___, 0, 0xbf, F64_REINTERPRET_I64, "f64.reinterpret/i64"),

/* Interpreter-only opcodes */
  (___, ___, ___, 0, 0xe0, INTERP_ALLOCA, "alloca"),
  (___, ___, ___, 0, 0xe1, INTERP_BR_UNLESS, "br_unless"),
  (___, ___, ___, 0, 0xe2, INTERP_CALL_HOST, "call_host"),
  (___, ___, ___, 0, 0xe3, INTERP_DATA, "data"),
  (___, ___, ___, 0, 0xe4, INTERP_DROP_KEEP, "drop_keep"),
}