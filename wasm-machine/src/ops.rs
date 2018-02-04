use Error;
use core::ops::Range;
pub type OpResult = Result<(), Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    Void = 0x40,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
}

#[derive(Debug)]
pub struct BranchTable<'a> {
    table: &'a [u32],
    default: u32,
}

pub trait WasmI32 : ControlFlow + Core + Int32 {}

pub trait ControlFlow {
    fn op_unreachable(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_nop(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_block(&mut self, _span: Range<usize>, _sig: &BlockType) -> OpResult { Err(Error::Unimplemented) }
    fn op_loop(&mut self, _span: Range<usize>, _sig: &BlockType) -> OpResult { Err(Error::Unimplemented) }
    fn op_if(&mut self, _span: Range<usize>, _sig: &BlockType) -> OpResult { Err(Error::Unimplemented) }
    fn op_else(&mut self, _span: Range<usize>, _sig: &BranchTable) -> OpResult { Err(Error::Unimplemented) }
    fn op_end(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_br(&mut self, _span: Range<usize>, _depth: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_br_if(&mut self, _span: Range<usize>, _depth: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_br_table(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_br_return(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
}

pub trait Core {
    fn op_get_local(&mut self, _span: Range<usize>, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_set_local(&mut self, _span: Range<usize>, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_tee_local(&mut self, _span: Range<usize>, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_get_global(&mut self, _span: Range<usize>, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_set_global(&mut self, _span: Range<usize>, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_select(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_call(&mut self, _span: Range<usize>, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_call_indirect(&mut self, _span: Range<usize>, _type_index: u32, _reserved: u8) -> OpResult { Err(Error::Unimplemented) }
    fn op_mem_grow(&mut self, _span: Range<usize>, _reserved: u8) -> OpResult { Err(Error::Unimplemented) }
    fn op_mem_size(&mut self, _span: Range<usize>, _reserved: u8) -> OpResult { Err(Error::Unimplemented) }
}

pub trait Int32 {
    fn op_i32_load(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_store(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load8_s(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load16_s(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load8_u(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load16_u(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_store8(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_store16(&mut self, _span: Range<usize>, _flags: u32, _offset: u32) -> OpResult { Err(Error::Unimplemented) }

    fn op_i32_const(&mut self, _span: Range<usize>, _value: i32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_add(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_sub(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_mul(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_div_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_div_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rem_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rem_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }

    fn op_i32_and(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_or(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_xor(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_shl(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_shr_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_shr_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rotl(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rotr(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_clz(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_ctz(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_popcnt(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_eqz(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }

    fn op_i32_eq(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_ne(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_lt_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_lt_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_le_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_le_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_gt_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_gt_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_ge_s(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_ge_u(&mut self, _span: Range<usize>) -> OpResult { Err(Error::Unimplemented) }
}