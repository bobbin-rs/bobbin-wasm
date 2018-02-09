use {Error, Reader, Writer, TypeValue, SectionType, ExternalKind, Module, Delegate};
use types::*;
// use loader::{Label, Loader};
// use stack::Stack;
use opcode::*;
use event::Event;

use core::mem;
use core::convert::TryFrom;
use core::ops::Range;

// macro_rules! event {
//     ($evt:expr) => (
//         self.dispatch($evt)?;
//     )
// }

pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;
pub const FIXUP: u32 = 0xffff_ffff;

pub type ModuleResult<T> = Result<T, Error>;

pub struct ModuleLoader<'d, 'r, 'w, D: 'd + Delegate> {
    d: &'d mut D,
    r: Reader<'r>,
    w: Writer<'w>,
    m: Module<'w>,
}

impl<'d, 'r, 'w, D: 'd + Delegate> ModuleLoader<'d, 'r, 'w, D> {
    pub fn new(d: &'d mut D, r: Reader<'r>, mut w: Writer<'w>) -> Self {
        let m = Module::new(w.split());
        ModuleLoader { d, r, w, m }
    }

    fn dispatch(&mut self, evt: Event) -> ModuleResult<()> {
        self.d.dispatch(evt)
    }

    fn done(&self) -> bool {
        self.r.done()
    }

    fn slice(&self, range: Range<usize>) -> &[u8] {
        self.r.slice(range)
    }

    fn read_u8(&mut self) -> ModuleResult<u8> {
        self.r.read_u8()
    }

    fn read_u32(&mut self) -> ModuleResult<u32> {
        self.r.read_u32()
    }

    fn read_var_u1(&mut self) -> ModuleResult<u8> {
        self.r.read_var_u1()
    }        

    fn read_var_i7(&mut self) -> ModuleResult<i8> {
        self.r.read_var_i7()
    }    

    fn read_var_u7(&mut self) -> ModuleResult<u8> {
        self.r.read_var_u7()
    }    

    fn read_var_u32(&mut self) -> ModuleResult<u32> {
        self.r.read_var_u32()
    }

    fn read_var_i32(&mut self) -> ModuleResult<i32> {
        self.r.read_var_i32()
    }

    fn read_var_i64(&mut self) -> ModuleResult<i64> {
        self.r.read_var_i64()
    }

    fn read_f32(&mut self) -> ModuleResult<f32> {
        self.r.read_f32()
    }

    fn read_f64(&mut self) -> ModuleResult<f64> {
        self.r.read_f64()
    }


    fn read_var_i7_expecting(&mut self, want: i8, err: Error) -> ModuleResult<i8> {
        if let Ok(got) = self.read_var_i7() {
            if got == want {
                return Ok(got)
            }
        }
        Err(err)
    }

    fn read_u32_expecting(&mut self, want: u32, err: Error) -> ModuleResult<u32> {
        if let Ok(got) = self.read_u32() {
            if got == want {
                return Ok(got)
            }
        }
        Err(err)
    }

    fn read_count(&mut self) -> ModuleResult<u32> {
        self.read_var_u32()
    }

    fn read_type(&mut self) -> ModuleResult<TypeValue> {
        Ok(TypeValue::from(self.r.read_var_i7()?))
    }

    fn read_depth(&mut self) -> ModuleResult<Depth> {
        self.read_var_u32()
    }

    fn read_index(&mut self) -> ModuleResult<u32> {
        self.read_var_u32()
    }

    fn read_type_index(&mut self) -> ModuleResult<TypeIndex> {
        self.read_index().map(TypeIndex)
    }

    fn read_func_index(&mut self) -> ModuleResult<FuncIndex> {
        self.read_index().map(FuncIndex)
    }

    fn read_table_index(&mut self) -> ModuleResult<TableIndex> {
        self.read_index().map(TableIndex)
    }    

    fn read_mem_index(&mut self) -> ModuleResult<MemIndex> {
        self.read_index().map(MemIndex)
    }

    fn read_global_index(&mut self) -> ModuleResult<GlobalIndex> {
        self.read_index().map(GlobalIndex)
    }

    fn read_local_index(&mut self) -> ModuleResult<LocalIndex> {
        self.read_index().map(LocalIndex)
    }

    fn read_external_kind(&mut self) -> ModuleResult<ExternalKind> {
        Ok(match self.read_var_u7()? {
            0x00 => ExternalKind::Function,
            0x01 => ExternalKind::Table,
            0x02 => ExternalKind::Memory,
            0x03 => ExternalKind::Global,
            id @ _ => return Err(Error::InvalidGlobalKind{ id }),
        })
    }

    fn read_external_index(&mut self) -> ModuleResult<ExternalIndex> {
        use ExternalKind::*;
        match self.read_external_kind()? {
            Function => self.read_func_index().map(ExternalIndex::Func),
            Table => self.read_table_index().map(ExternalIndex::Table),
            Memory => self.read_mem_index().map(ExternalIndex::Mem),
            Global => self.read_global_index().map(ExternalIndex::Global),
        }
    }
    
    fn read_bytes_range(&mut self) -> ModuleResult<Range<usize>> {
        let len = self.r.read_var_u32()? as usize;
        self.r.read_range(len)
    }

    fn read_identifier_range(&mut self) -> ModuleResult<Range<usize>> {
        self.read_bytes_range()
    }

    fn read_resizable_limits(&mut self) -> ModuleResult<ResizableLimits> {
        let flags = self.read_var_u32()?;
        let min = self.read_var_u32()?;
        let max = if flags & 0x1 != 0 {
            Some(self.read_var_u32()?)
        } else {
            None
        };
        Ok(ResizableLimits { flags, min, max })
    }

    fn read_initializer(&mut self) -> ModuleResult<Initializer> {
        let opcode = self.read_u8()?;
        let immediate = self.read_var_u32()?;
        let end = self.read_u8()?;
        Ok(Initializer { opcode, immediate, end })
    }

    fn slice_identifier(&self, range: Range<usize>) -> Identifier {
        Identifier(self.slice(range))
    }

    fn write_u8(&mut self, value: u8) -> ModuleResult<()> {
        self.w.write_u8(value)
    }

    fn write_i8(&mut self, value: i8) -> ModuleResult<()> {
        self.w.write_i8(value)
    }    

    fn write_u32(&mut self, value: u32) -> ModuleResult<()> {
        self.w.write_u32(value)
    }

    fn write_u32_at(&mut self, value: u32, offset: u32) -> ModuleResult<()> {
        self.w.write_u32_at(value, offset as usize)
    }

    fn write_u32_fixup(&mut self) -> ModuleResult<u32> {
        let pos = self.w.pos();
        self.w.write_u32(FIXUP)?;
        Ok(pos as u32)
    }
    
    fn apply_u32_fixup(&mut self, value: u32, offset: u32) -> ModuleResult<()> {
        // println!("apply_fixup: {:08x} @ {:08x}", value, offset);
        self.w.write_u32_at(value as u32, offset as usize)
    }

    fn copy_u8(&mut self) -> ModuleResult<u8> {
        let val = self.r.read_u8()?;
        self.w.write_u8(val)?;
        Ok(val)
    }    

    fn copy_var_u1(&mut self) -> ModuleResult<u8> {
        let val = self.r.read_var_u1()?;
        self.w.write_u8(val)?;
        Ok(val)
    }    

    fn copy_var_i7(&mut self) -> ModuleResult<i8> {
        let val = self.r.read_var_i7()?;
        self.w.write_i8(val)?;
        Ok(val)
    }    

    fn copy_var_u7(&mut self) -> ModuleResult<u8> {
        let val = self.r.read_var_u7()?;
        self.w.write_u8(val)?;
        Ok(val)
    }        

    fn copy_var_u32(&mut self) -> ModuleResult<u32> {
        let val = self.r.read_var_u32()?;
        self.w.write_u32(val)?;
        Ok(val)
    }

    fn copy_var_i32(&mut self) -> ModuleResult<i32> {
        let val = self.r.read_var_i32()?;
        self.w.write_i32(val)?;
        Ok(val)
    }

    fn copy_kind(&mut self) -> ModuleResult<u8> {
        self.copy_var_u7()
    }

    fn copy_type(&mut self) -> ModuleResult<i8> {
        self.copy_var_i7()
    }

    fn copy_index(&mut self) -> ModuleResult<u32> {
        self.copy_var_u32()
    }

    fn copy_type_index(&mut self) -> ModuleResult<u32> {
        self.copy_index()
    }

    fn copy_function_index(&mut self) -> ModuleResult<u32> {
        self.copy_index()
    }

    fn copy_table_index(&mut self) -> ModuleResult<u32> {
        self.copy_index()
    }

    fn copy_memory_index(&mut self) -> ModuleResult<u32> {
        self.copy_index()
    }    

    fn copy_global_index(&mut self) -> ModuleResult<u32> {
        self.copy_index()
    }    

    fn copy_len(&mut self) -> ModuleResult<u32> {
        self.copy_var_u32()
    }

    fn copy_count(&mut self) -> ModuleResult<u32> {
        self.copy_var_u32()
    }

    fn copy_identifier(&mut self) -> ModuleResult<()> {
        let n = self.copy_len()?;
        for _ in 0..n {
            self.copy_u8()?;
        }
        Ok(())
    }

    pub fn load(mut self, name: &str) -> ModuleResult<Module<'w>> {
        let version = self.load_header()?;        
        self.dispatch(Event::Start { name, version })?;
        while !self.done() {
            let _s = self.load_section()?;
            self.m.extend(self.w.split())
        }
        self.dispatch(Event::End)?;
        Ok(self.m)
    }

    pub fn load_header(&mut self) -> ModuleResult<u32> {        
        self.read_u32_expecting(MAGIC_COOKIE, Error::InvalidHeader)?;
        self.read_u32()
    }
    
    pub fn load_section(&mut self) -> ModuleResult<SectionType> {
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
                SectionType::Type => self.load_types()?,
                SectionType::Import => self.load_imports()?,
                SectionType::Function => self.load_functions()?,
                SectionType::Table => self.load_tables()?,
                SectionType::Memory => self.load_linear_memory()?,
                SectionType::Global => self.load_globals()?,
                SectionType::Export => self.load_exports()?,
                SectionType::Start => self.load_start()?,
                SectionType::Element => self.load_elements()?,
                SectionType::Code => self.load_code()?,
                SectionType::Data => self.load_data()?,
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

    pub fn copy_resizable_limits(&mut self) -> ModuleResult<()> {    
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#resizable-limits
        Ok({
            // flags
            let f = self.copy_var_u32()?;
            // minimum
            self.copy_var_u32()?;
            if f & 1 != 0 {
                // maximum
                self.copy_var_u32()?;
            }
        })
    }

    pub fn copy_linear_memory_description(&mut self) -> ModuleResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#linear-memory-description
        self.copy_resizable_limits()
    }

    pub fn copy_table_description(&mut self) -> ModuleResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#table-description
        Ok({
            // table element type
            self.copy_type()?;
            // resizable
            self.copy_resizable_limits()?;
        })
    }


    pub fn copy_global_description(&mut self) -> ModuleResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#global-description
        Ok({
            // type
            self.copy_type()?;
            // mutability
            self.copy_var_u1()?;
        })
    }

    pub fn copy_external_kind(&mut self) -> ModuleResult<ExternalKind> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#external-kinds
        use ExternalKind::*;
        Ok({
            match self.copy_var_u7()? {
                0x00 => Function,
                0x01 => Table,
                0x02 => Memory,
                0x03 => Global,
                id @ _ => return Err(Error::InvalidGlobalKind{ id }),
            }
        })
    }


    pub fn copy_initializer(&mut self) -> ModuleResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#instantiation-time-initializers
        Ok({
            // opcode
            self.copy_u8()?;
            // immediate
            self.copy_var_i32()?;
            // end - verify
            self.read_u8()?;
        })
    }   

    pub fn copy_local(&mut self) -> ModuleResult<()> {
        Ok({
            // count
            self.copy_count()?;
            // value type
            self.copy_type()?;
        })
    }

    pub fn load_types(&mut self) -> ModuleResult<()> {
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

    pub fn load_imports(&mut self) -> ModuleResult<()> {
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

    pub fn load_functions(&mut self) -> ModuleResult<()> {
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

    pub fn load_tables(&mut self) -> ModuleResult<()> {
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

    pub fn load_linear_memory(&mut self) -> ModuleResult<()> {
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

    pub fn load_globals(&mut self) -> ModuleResult<()> {
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

    pub fn load_exports(&mut self) -> ModuleResult<()> {
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

    pub fn load_start(&mut self) -> ModuleResult<()> {
        // https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#start-section
        Ok({
            // start index
            let index = self.read_func_index()?;
            self.dispatch(Event::StartFunction { index })?;
        })
    }

    pub fn table_type(&self, _index: u32) -> ModuleResult<TypeValue> {
        Err(Error::Unimplemented)
    }

    pub fn load_elements(&mut self) -> ModuleResult<()> {
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

    pub fn load_data(&mut self) -> ModuleResult<()> {
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

    pub fn load_code(&mut self) -> ModuleResult<()> {
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
                let mut i = 0;                
                while self.r.pos() < body_end {
                    self.load_instruction(i)?;
                    i += 1;
                }
            }
            self.dispatch(Event::CodeEnd)?;
        })
    }

    pub fn load_instruction(&mut self, n: u32) -> ModuleResult<()> {
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
                    self.d.dispatch(Event::Instruction { n, offset, data, op: &op, imm})?;
                }
                for i in 0..count {
                    let depth = self.read_depth()?;
                    let imm = Immediate::BranchTableDepth { n: i, depth };
                    {
                        let end = self.r.pos();
                        let data = self.r.slice(offset as usize..end);
                        self.d.dispatch(Event::Instruction { n, offset, data, op: &op, imm})?;
                    }                }
                let depth = self.read_depth()?;
                let imm = Immediate::BranchTableDefault { depth };
                {
                    let end = self.r.pos();
                    let data = self.r.slice(offset as usize..end);
                    self.d.dispatch(Event::Instruction { n, offset, data, op: &op, imm})?;
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
        self.d.dispatch(Event::Instruction { n, offset, data, op: &op, imm })?;
        Ok(())
    }

    pub fn trace<F: FnOnce()->()>(&self, _f: F) -> ModuleResult<()> {
        // Ok(f())
        Ok(())
    }
}
