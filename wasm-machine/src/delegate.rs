#![allow(unused_variables)]
use {Error, SectionType, TypeValue, ExternalKind};

pub type DelegateResult = Result<(), Error>;

pub trait Delegate {
    fn start(&mut self) -> DelegateResult { Ok(()) }
    fn end(&mut self, pos: u32) -> DelegateResult { Ok(()) }

    fn section_start(&mut self, s_type: SectionType, s_start: u32, s_end: u32, s_len: u32) -> DelegateResult { Ok(()) }
    fn section_end(&mut self) -> DelegateResult { Ok(()) }

    fn types_start(&mut self, count: u32) -> DelegateResult { Ok(()) }

    fn type_start(&mut self, index: u32, form: i8) -> DelegateResult { Ok(()) }
    fn type_parameters_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn type_parameter(&mut self, index: u32, tv: TypeValue) -> DelegateResult { Ok(()) }
    fn type_parameters_end(&mut self) -> DelegateResult { Ok(()) }
    fn type_returns_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn type_return(&mut self, index: u32, tv: TypeValue) -> DelegateResult { Ok(()) }
    fn type_returns_end(&mut self) -> DelegateResult { Ok(()) }
    fn type_end(&mut self) -> DelegateResult { Ok(()) }
    
    fn types_end(&mut self) -> DelegateResult { Ok(()) }

    fn functions_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn function(&mut self, index: u32, sig: u32) -> DelegateResult { Ok(()) }
    fn functions_end(&mut self) -> DelegateResult { Ok(()) }

    fn tables_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn table(&mut self, index: u32, element_type: TypeValue, flags: u32, minimum: u32, maximum: Option<u32>) -> DelegateResult { Ok(()) }
    fn tables_end(&mut self) -> DelegateResult { Ok(()) }

    fn memories_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn memory(&mut self, index: u32, flags: u32, minimum: u32, maximum: Option<u32>) -> DelegateResult { Ok(()) }
    fn memories_end(&mut self) -> DelegateResult { Ok(()) }    

    fn globals_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn global(&mut self, index: u32, tv: TypeValue, mutability: u8, init_opcode: u8, init_immediate: u32) -> DelegateResult { Ok(()) }
    fn globals_end(&mut self) -> DelegateResult { Ok(()) }

    fn exports_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn export(&mut self, index: u32, id: &[u8], kind: ExternalKind, external_index: u32) -> DelegateResult { Ok(()) }
    fn exports_end(&mut self) -> DelegateResult { Ok(()) }

    fn data_segments_start(&mut self, count: u32) -> DelegateResult { Ok(()) }
    fn data_segment(&mut self, index: u32, memory_index: u32, offset_opcode: u8, offset_immediate: u32, data: &[u8]) -> DelegateResult { Ok(()) }
    fn data_segments_end(&mut self) -> DelegateResult { Ok(()) }

}