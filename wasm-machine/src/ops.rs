use Error;

pub type OpResult = Result<(), Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockSignature {
    I32,
    I64,
    F32,
    F64,
    Void,
}

#[derive(Debug)]
pub struct BranchTable<'a> {
    table: &'a [u32],
    default: u32,
}

#[derive(Debug)]
pub struct LoadStore {
    flags: u32,
    offset: u32,
}

pub trait ControlFlow {
    fn op_unreachable(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_nop(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_block(&mut self, _sig: &BlockSignature) -> OpResult { Err(Error::Unimplemented) }
    fn op_loop(&mut self, _sig: &BlockSignature) -> OpResult { Err(Error::Unimplemented) }
    fn op_if(&mut self, _sig: &BlockSignature) -> OpResult { Err(Error::Unimplemented) }
    fn op_else(&mut self, _sig: &BranchTable) -> OpResult { Err(Error::Unimplemented) }
    fn op_end(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_br(&mut self, _depth: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_br_if(&mut self, _depth: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_br_table(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_br_return(&mut self) -> OpResult { Err(Error::Unimplemented) }
}

pub trait Core {
    fn op_get_local(&mut self, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_set_local(&mut self, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_tee_local(&mut self, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_get_global(&mut self, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_set_global(&mut self, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_select(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_call(&mut self, _id: u32) -> OpResult { Err(Error::Unimplemented) }
    fn op_call_indirect(&mut self, _signature: u32, _reserved: bool) -> OpResult { Err(Error::Unimplemented) }
    fn op_mem_grow(&mut self, _reserved: bool) -> OpResult { Err(Error::Unimplemented) }
    fn op_mem_size(&mut self, _reserved: bool) -> OpResult { Err(Error::Unimplemented) }
}

pub trait Int32 {
    fn op_i32_load(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_store(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load8_s(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load16_s(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load8_u(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_load16_u(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_store8(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_store16(&mut self, _imm: &LoadStore) -> OpResult { Err(Error::Unimplemented) }

    fn op_i32_const(&mut self, _value: i32) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_add(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_sub(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_mul(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_div_s(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_div_u(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rem_s(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rem_u(&mut self) -> OpResult { Err(Error::Unimplemented) }

    fn op_i32_and(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_or(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_xor(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_shl(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_shr_s(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_shr_u(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rotl(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_rotr(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_clz(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_ctz(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_popcnt(&mut self) -> OpResult { Err(Error::Unimplemented) }    
    fn op_i32_eqz(&mut self) -> OpResult { Err(Error::Unimplemented) }

    fn op_i32_eq(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_ne(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_lt_s(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_lt_u(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_le_s(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_le_u(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_gt_s(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_gt_u(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_ge_s(&mut self) -> OpResult { Err(Error::Unimplemented) }
    fn op_i32_ge_u(&mut self) -> OpResult { Err(Error::Unimplemented) }
}