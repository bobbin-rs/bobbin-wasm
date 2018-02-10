use {Error, WasmResult, Reader, TypeValue, SectionType, ExternalKind, Delegate};
use MAGIC_COOKIE;
use types::*;
use opcode::*;
use event::Event;

use core::mem;
use core::convert::TryFrom;
use core::ops::Range;



pub struct BinaryReader<'d, 'r, D: 'd + Delegate> {
    d: &'d mut D,
    r: Reader<'r>,
}

impl<'d, 'r, D: 'd + Delegate> BinaryReader<'d, 'r, D> {
    pub fn new(d: &'d mut D, r: Reader<'r>) -> Self {
        BinaryReader { d, r }
    }

    fn dispatch(&mut self, evt: Event) -> WasmResult<()> {
        self.d.dispatch(evt)
    }

    fn done(&self) -> bool {
        self.r.done()
    }

    fn slice(&self, range: Range<usize>) -> &[u8] {
        self.r.slice(range)
    }

    fn read_u8(&mut self) -> WasmResult<u8> {
        self.r.read_u8()
    }

    fn read_u32(&mut self) -> WasmResult<u32> {
        self.r.read_u32()
    }

    fn read_var_u1(&mut self) -> WasmResult<u8> {
        self.r.read_var_u1()
    }        

    fn read_var_i7(&mut self) -> WasmResult<i8> {
        self.r.read_var_i7()
    }    

    fn read_var_u7(&mut self) -> WasmResult<u8> {
        self.r.read_var_u7()
    }    

    fn read_var_u32(&mut self) -> WasmResult<u32> {
        self.r.read_var_u32()
    }

    fn read_var_i32(&mut self) -> WasmResult<i32> {
        self.r.read_var_i32()
    }

    fn read_var_i64(&mut self) -> WasmResult<i64> {
        self.r.read_var_i64()
    }

    fn read_f32(&mut self) -> WasmResult<f32> {
        self.r.read_f32()
    }

    fn read_f64(&mut self) -> WasmResult<f64> {
        self.r.read_f64()
    }

    fn read_u32_expecting(&mut self, want: u32, err: Error) -> WasmResult<u32> {
        if let Ok(got) = self.read_u32() {
            if got == want {
                return Ok(got)
            }
        }
        Err(err)
    }

    fn read_count(&mut self) -> WasmResult<u32> {
        self.read_var_u32()
    }

    fn read_type(&mut self) -> WasmResult<TypeValue> {
        Ok(TypeValue::from(self.r.read_var_i7()?))
    }

    fn read_depth(&mut self) -> WasmResult<Depth> {
        self.read_var_u32()
    }

    fn read_index(&mut self) -> WasmResult<u32> {
        self.read_var_u32()
    }

    fn read_type_index(&mut self) -> WasmResult<TypeIndex> {
        self.read_index().map(TypeIndex)
    }

    fn read_func_index(&mut self) -> WasmResult<FuncIndex> {
        self.read_index().map(FuncIndex)
    }

    fn read_table_index(&mut self) -> WasmResult<TableIndex> {
        self.read_index().map(TableIndex)
    }    

    fn read_mem_index(&mut self) -> WasmResult<MemIndex> {
        self.read_index().map(MemIndex)
    }

    fn read_global_index(&mut self) -> WasmResult<GlobalIndex> {
        self.read_index().map(GlobalIndex)
    }

    fn read_local_index(&mut self) -> WasmResult<LocalIndex> {
        self.read_index().map(LocalIndex)
    }

    fn read_external_kind(&mut self) -> WasmResult<ExternalKind> {
        Ok(match self.read_var_u7()? {
            0x00 => ExternalKind::Function,
            0x01 => ExternalKind::Table,
            0x02 => ExternalKind::Memory,
            0x03 => ExternalKind::Global,
            id @ _ => return Err(Error::InvalidGlobalKind{ id }),
        })
    }

    fn read_external_index(&mut self) -> WasmResult<ExternalIndex> {
        use ExternalKind::*;
        match self.read_external_kind()? {
            Function => self.read_func_index().map(ExternalIndex::Func),
            Table => self.read_table_index().map(ExternalIndex::Table),
            Memory => self.read_mem_index().map(ExternalIndex::Mem),
            Global => self.read_global_index().map(ExternalIndex::Global),
        }
    }
    
    fn read_bytes_range(&mut self) -> WasmResult<Range<usize>> {
        let len = self.r.read_var_u32()? as usize;
        self.r.read_range(len)
    }

    fn read_identifier_range(&mut self) -> WasmResult<Range<usize>> {
        self.read_bytes_range()
    }

    fn read_resizable_limits(&mut self) -> WasmResult<ResizableLimits> {
        let flags = self.read_var_u32()?;
        let min = self.read_var_u32()?;
        let max = if flags & 0x1 != 0 {
            Some(self.read_var_u32()?)
        } else {
            None
        };
        Ok(ResizableLimits { flags, min, max })
    }

    fn read_initializer(&mut self) -> WasmResult<Initializer> {
        let opcode = self.read_u8()?;
        let immediate = self.read_var_u32()?;
        let end = self.read_u8()?;
        Ok(Initializer { opcode, immediate, end })
    }

    fn slice_identifier(&self, range: Range<usize>) -> Identifier {
        Identifier(self.slice(range))
    }

    pub fn read(mut self, name: &str) -> WasmResult<()> {
        let version = self.read_header()?;        
        self.dispatch(Event::Start { name, version })?;
        while !self.done() {
            let _s = self.read_section()?;
        }
        self.dispatch(Event::End)?;
        Ok(())
    }

    pub fn read_header(&mut self) -> WasmResult<u32> {        
        self.read_u32_expecting(MAGIC_COOKIE, Error::InvalidHeader)?;
        self.read_u32()
    }
    
    pub fn read_section(&mut self) -> WasmResult<SectionType> {
        // ID(u8) LEN(u32) [LEN]
        Ok({
            let s_type = SectionType::try_from(self.read_var_u7()?)?;

            let s_len = self.read_var_u32()?;
            let s_beg = self.r.pos() as u32;
            let s_end = s_beg + s_len;

            self.dispatch(Event::SectionStart { s_type, s_beg, s_end, s_len })?;
            // self.d.section_start(s, s_beg, s_end, s_len)?;

            // self.write_u8(s as u8)?;
            // let fixup_len = self.write_u32_fixup()?;
            // let w_beg = self.w.pos();

            match s_type {
                SectionType::Type => self.read_types()?,
                SectionType::Import => self.read_imports()?,
                SectionType::Function => self.read_functions()?,
                SectionType::Table => self.read_tables()?,
                SectionType::Memory => self.read_linear_memory()?,
                SectionType::Global => self.read_globals()?,
                SectionType::Export => self.read_exports()?,
                SectionType::Start => self.read_start()?,
                SectionType::Element => self.read_elements()?,
                SectionType::Code => self.read_code()?,
                SectionType::Data => self.read_data()?,
                _ => self.r.advance(s_len as usize)
            }
            let r_pos = self.r.pos() as u32;
            if r_pos != s_end {            
                return Err(Error::UnexpectedData { wanted: s_len, got: (r_pos - s_beg) as u32 })
            }
            self.dispatch(Event::SectionEnd)?;
            // self.d.section_end()?;

            // let w_end = self.w.pos();
            // let w_len = w_end - w_beg;            
            // self.apply_u32_fixup(w_len as u32, fixup_len)?;
            s_type
        })
    }

    pub fn read_types(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#type-section
        Ok({
            let c = self.read_count()?;
            // self.d.types_start(count)?;
            self.dispatch(Event::TypesStart { c })?;

            for n in 0..c {
                let form = self.read_type()?;
                self.dispatch(Event::TypeStart { n, form })?;

                {
                    let c = self.read_count()?;
                    self.dispatch(Event::TypeParametersStart { c })?;
                    for n in 0..c {
                        let t = self.read_type()?;
                        self.dispatch(Event::TypeParameter { n, t })?;
                    }
                    self.dispatch(Event::TypeParametersEnd)?;
                }

                {
                    let c = self.read_count()?;
                    self.dispatch(Event::TypeReturnsStart { c })?;
                    for n in 0..c {
                        let t = self.read_type()?;
                        self.dispatch(Event::TypeReturn { n, t })?;
                    }
                    self.dispatch(Event::TypeReturnsEnd)?;
                }
    
                self.dispatch(Event::TypeEnd)?;
            }

            self.dispatch(Event::TypesEnd)?;            
        })
    }

    pub fn read_imports(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#import-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::ImportsStart { c })?;
            for n in 0..c {
                let module_range = self.read_identifier_range()?;
                let export_range = self.read_identifier_range()?;
                let index = self.read_external_index()?;

                let module = Identifier(self.r.slice(module_range));
                let export = Identifier(self.r.slice(export_range));
                self.d.dispatch(Event::Import { n, module, export, index })?;
            }
            self.dispatch(Event::ImportsEnd)?;
        })
    }

    pub fn read_functions(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#function-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::FunctionsStart { c })?;
            for n in 0..c {
                let index = self.read_type_index()?;
                self.dispatch(Event::Function { n, index })?;
            }
            self.dispatch(Event::FunctionsEnd)?;
        })
    }

    pub fn read_tables(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#table-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::TablesStart { c })?;
            for n in 0..c {
                let element_type = self.read_type()?;
                let limits = self.read_resizable_limits()?;
                self.dispatch(Event::Table { n, element_type, limits })?;
            }
            self.dispatch(Event::TablesEnd)?;
        })
    }

    pub fn read_linear_memory(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#linear-memory-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::MemsStart { c })?;
            for n in 0..c {
                let limits = self.read_resizable_limits()?;
                self.dispatch(Event::Mem { n, limits })?;
            }
            self.dispatch(Event::MemsEnd)?;
        })
    }

    pub fn read_globals(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#global-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::GlobalsStart { c })?;
            for n in 0..c {
                let t = self.read_type()?;
                let mutability = self.r.read_var_u1()?;
                let init = self.read_initializer()?;
                self.dispatch(Event::Global { n, t, mutability, init })?;
            }
            self.dispatch(Event::GlobalsEnd)?;
        })
    }

    pub fn read_exports(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#export-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::ExportsStart { c })?;
            for n in 0..c { 
                let id_range = self.read_identifier_range()?;
                let index = self.read_external_index()?;
                let id = Identifier(self.r.slice(id_range));

                self.d.dispatch(Event::Export { n, id, index})?;
            }
            self.dispatch(Event::ExportsEnd)?;
        })
    }

    pub fn read_start(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#start-section
        Ok({
            // start index
            let index = self.read_func_index()?;
            self.dispatch(Event::StartFunction { index })?;
        })
    }

    pub fn read_elements(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#element-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::ElementsStart { c })?;
            for n in 0..c { 
                let index = self.read_table_index()?;
                let offset = self.read_initializer()?;

                // TODO: check table index type
                let data_len = self.read_var_u32()? as usize;
                let data_beg = self.r.pos() as usize;
                let data_end = data_beg + data_len * mem::size_of::<FuncIndex>();
                let data = Some(self.r.slice(data_beg..data_end));
                self.d.dispatch(Event::Element { n, index, offset, data })?;
            }
            self.dispatch(Event::ElementsEnd)?;
        })
    }

    pub fn read_data(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#data-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::DataSegmentsStart { c })?;
            for n in 0..c { 
                let index = self.read_mem_index()?;
                let offset = self.read_initializer()?;
                let data_range = self.read_bytes_range()?;
                let data = self.r.slice(data_range);
                self.d.dispatch(Event::DataSegment { n, index, offset, data } )?;
            }
            self.dispatch(Event::DataSegmentsEnd)?;
        })
    }

    pub fn read_code(&mut self) -> WasmResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#code-section
        Ok({
            let c = self.read_count()?;
            self.dispatch(Event::CodeStart { c })?;
            for n in 0..c {
                let offset = self.r.pos() as u32;
                let size = self.read_var_u32()?;
                let body_beg = self.r.pos();
                let body_end = body_beg + size as usize;     

                let locals = self.read_count()?;
                self.dispatch(Event::Body { n, offset, size, locals })?;
                for i in 0..locals {
                    let n = self.read_count()?;
                    let t = self.read_type()?;
                    self.dispatch(Event::Local { i, n, t })?;
                }
                self.dispatch(Event::InstructionsStart { n, locals })?;                
                while self.r.pos() < body_end {
                    self.read_instruction(n, locals)?;
                }
                self.dispatch(Event::BodyEnd)?;
            }
            self.dispatch(Event::CodeEnd)?;
        })
    }

    pub fn read_instruction(&mut self, n: u32, locals: u32) -> WasmResult<()> {
        use self::ImmediateType::*;
        let offset = self.r.pos() as u32;
        let op = Opcode::try_from(self.read_u8()?)?;
        let imm = match op.immediate_type() {
            None => Immediate::None,
            BlockSignature => {
                let signature = self.read_type()?;
                Immediate::Block { signature }
            },
            BranchDepth => {
                let depth = self.read_depth()?;
                Immediate::Branch { depth }
            },
            BranchTable => {
                let count = self.read_count()?;
                let imm = Immediate::BranchTable { count };
                {
                    let end = self.r.pos();
                    let data = self.r.slice(offset as usize..end);
                    self.d.dispatch(Event::Instruction(n, locals, Instruction { offset, data, op: &op, imm }))?;
                }
                for i in 0..count {
                    let depth = self.read_depth()?;
                    let imm = Immediate::BranchTableDepth { n: i, depth };
                    {
                        let end = self.r.pos();
                        let data = self.r.slice(offset as usize..end);
                        self.d.dispatch(Event::Instruction(n, locals, Instruction { offset, data, op: &op, imm }))?;
                    }                }
                let depth = self.read_depth()?;
                let imm = Immediate::BranchTableDefault { depth };
                {
                    let end = self.r.pos();
                    let data = self.r.slice(offset as usize..end);
                    self.d.dispatch(Event::Instruction(n, locals, Instruction { offset, data, op: &op, imm }))?;
                }
                return Ok(())                
            },
            Local => {                
                let index = self.read_local_index()?;
                Immediate::Local { index }
            },
            Global => {
                let index = self.read_global_index()?;
                Immediate::Global { index }
            },
            Call => {
                let index = self.read_func_index()?;
                Immediate::Call { index }
            },
            CallIndirect => {
                let index = self.read_type_index()?;
                Immediate::CallIndirect { index }
            },
            I32 => {
                let value = self.read_var_i32()?;
                Immediate::I32Const { value }
            },
            F32 => {
                let value = self.read_f32()?;
                Immediate::F32Const { value }
            },
            I64 => {
                let value = self.read_var_i64()?;
                Immediate::I64Const { value }
            },
            F64 => {
                let value = self.read_f64()?;
                Immediate::F64Const { value }
            },
            LoadStore=> {
                let align = self.read_var_u32()?;
                let offset = self.read_var_u32()?;
                Immediate::LoadStore { align, offset }
            },
            Memory => {
                let reserved = self.read_var_u1()?;
                Immediate::Memory { reserved }
            },                
        };
        let end = self.r.pos();
        let data = self.r.slice(offset as usize..end);
        self.d.dispatch(Event::Instruction(n, locals, Instruction { offset, data, op: &op, imm }))?;
        Ok(())
    }
}
