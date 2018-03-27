use error::Error;


use types::*;
use parser::{FallibleIterator};
use parser::module::*;
use compiler::*;
use core::cell::Cell;
use environ::{Environment, HostHandler};
use memory_inst::MemoryInst;
use small_vec::SmallVec;
use writer::Writer;

use parser::module::ExportDesc;
use parser::types::{FunctionType, GlobalType};

use core::fmt;

pub struct ModuleInst<'buf> {
    function_types: SmallVec<'buf, FunctionType<'buf>>,
    functions: SmallVec<'buf, FuncInst<'buf>>,
    globals: SmallVec<'buf, GlobalInst>,
    exports: SmallVec<'buf, ExportInst<'buf>>,
    tables: SmallVec<'buf, SmallVec<'buf, u32>>,
    code: CompiledCode<'buf>,
}

impl<'buf, 'env> ModuleInst<'buf> {
    pub fn new<H: HostHandler>(buf: &'buf mut [u8], env: &Environment<H>, mem: &MemoryInst, m: Module) -> Result<(&'buf mut [u8], ModuleInst<'buf>), Error> {
        let mut w = Writer::new(buf);

        let mut function_types = w.alloc_smallvec(16);
        let mut functions = w.alloc_smallvec(16);
        let mut globals = w.alloc_smallvec(16);
        let mut tables = w.alloc_smallvec(16);
        let mut exports = w.alloc_smallvec(16);

        let mut sections = m.sections();

        while let Some(section) = sections.next()? {
            match section.id() {
                Id::Type => {
                    let mut types = section.types();
                    while let Some(t) = types.next()? {
                        let functype = 0x60;
                        let parameters: &[ValueType] = w.copy_slice(t.parameters)?;
                        let results: &[ValueType] = w.copy_slice(t.results)?;
                        function_types.push(FunctionType { functype, parameters, results });
                    }
                },
                Id::Import => {
                    let mut imports = section.imports();
                    let mut import_index = 0;
                    while let Some(i) = imports.next()? {                    
                        info!("Import: {:?}", i);
                        match i.import_desc {
                            ImportDesc::Func(type_index) => {
                                let type_index = type_index as usize;
                                let module_bytes = w.copy_slice(i.module.as_bytes())?;
                                let module = ::core::str::from_utf8(module_bytes)?;
                                let name_bytes = w.copy_slice(i.name.as_bytes())?;
                                let name = ::core::str::from_utf8(name_bytes)?;
                                let module_index = 0;
                                if module_bytes == b"host" {
                                    let host_index = env.import_host_function(module, name, &i.import_desc)?;
                                    functions.push(FuncInst::Host { type_index, module, name, host_index });
                                } else {
                                    functions.push(FuncInst::Import { type_index, module, name, module_index, import_index });
                                }
                            },
                            ImportDesc::Table(_) => {
                                // info!("Import Table");
                            },
                            ImportDesc::Memory(_) => {
                                // info!("Import Memory");
                            },
                            ImportDesc::Global(global_type) => {
                                globals.push(GlobalInst::Import { global_type, import_index});
                            }
                        }
                        import_index += 1;
                    }
                },
                Id::Function => {
                    let mut funcs = section.functions();
                    let mut function_index = 0;
                    while let Some(function) = funcs.next()? {                    
                        let type_index = function as usize;
                        functions.push(FuncInst::Local { type_index, function_index });
                        function_index += 1;
                    }
                },
                Id::Global => {
                    let mut globs = section.globals();
                    let mut global_index = 0;
                    while let Some(global) = globs.next()? {  
                        let global_type = global.global_type;
                        let init = global.init;
                        let value = if let Some(value) = init.i32_value() {
                            Cell::new(Value::from(value))
                        } else {
                            panic!("Invalid global initializer value");
                        };
                        globals.push(GlobalInst::Local { global_type, global_index, value });
                        global_index += 1;
                    }
                },
                Id::Table => {
                    let mut tabs = section.tables();
                    while let Some(table) = tabs.next()? {                          
                        let TableType { elemtype, limits } = table;
                        info!("Adding table: {} {:?}", elemtype, limits);
                        let size = if let Some(max) = limits.max {
                            max
                        } else {
                            limits.min
                        };
                        let mut t: SmallVec<u32> = w.alloc_smallvec(size as usize);
                        for _ in 0..size {
                            t.push(0);
                        }
                        tables.push(t);
                    }
                },   
                Id::Export => {
                    let mut expts = section.exports();
                    while let Some(export) = expts.next()? {                         
                        let Export { name, export_desc } = export;
                        let bytes = w.copy_slice(name.as_bytes())?;
                        let name = ::core::str::from_utf8(bytes)?;
                        exports.push(ExportInst { name, export_desc });
                    }
                }
                Id::Element => {
                    let mut elements = section.elements();
                    while let Some(element) = elements.next()? {                         
                        let Element { table_index, offset, init } = element;
                        // use byteorder::{ByteOrder, LittleEndian};

                        info!("Initializing table {}", table_index);
                        let table = &mut tables[table_index as usize];
                        let offset = offset.i32_value().unwrap();
                        let mut i = 0;
                        let mut o = offset as usize;
                        while i < init.len() {
                            // let d = LittleEndian::read_u32(&data[i..]);
                            let d = init[i] as u32;
                            info!("{:08x}: {:08x}", o, d);
                            table[o] = d;
                            o += 1;
                            i += 1;
                        }              
                    }
                },
                Id::Data => {
                    let mut data = section.data();
                    while let Some(data) = data.next()? {                                             
                        let Data { mem_index: _, offset, init } = data;
                        let offset = offset.i32_value().unwrap();
                        for i in 0..init.len() {
                            let d = init[i];
                            let o = offset as usize + i;
                            // info!("{:08x}: {:02x}", o, d);
                            mem.set(o, d);
                        }
                    }
                },
                _ => {},
            }
        }

        let buf = w.into_slice();

        // Change compiler to use ModuleInst

        let (buf, code) = Compiler::new(&mut [0u8; 4096]).compile(buf, 
            function_types.as_ref(),
            functions.as_ref(), 
            globals.as_ref(),
        &m)?;

        Ok((buf, ModuleInst { function_types, functions, globals, exports, tables, code }))
    }

    pub fn function_types(&self) -> &[FunctionType] {
        self.function_types.as_ref()
    }

    pub fn functions(&self) -> &[FuncInst] {
        self.functions.as_ref()
    }

    pub fn globals(&self) -> &[GlobalInst] {
        self.globals.as_ref()
    }

    pub fn table(&self, index: usize) -> &SmallVec<u32> {
        &self.tables[index]
    }

    pub fn exports(&self) -> &[ExportInst] {
        self.exports.as_ref()
    }

    pub fn indirect_functions_len(&self) -> usize {
        self.tables[0].len()
    }

    pub fn indirect_function_id(&self, index: usize) -> u32 {
        self.tables[0][index]
    }

    pub fn type_signature(&self, index: usize) -> &FunctionType {
        &self.function_types[index]
    }

    pub fn global_type(&self, index: u32) -> Result<GlobalType, Error> {
        Ok({
            info!("global_type({})", index);
            if index as usize > self.globals.len() {
                return Err(Error::OutOfBounds);
            }
            match self.globals[index as usize] {
                GlobalInst::Local { global_type, global_index: _, value: _} => {
                    global_type
                },
                GlobalInst::Import { global_type, import_index: _ } => {
                    global_type
                }
            }
        })         
    }

    pub fn get_global(&self, index: u32) -> Result<i32, Error> {
        Ok({
            info!("get_global({})", index);
            if index as usize > self.globals.len() {
                return Err(Error::OutOfBounds);
            }
            match self.globals[index as usize] {
                GlobalInst::Local { global_type: _, global_index: _, ref value } => {
                    let v = value.get().0;
                    info!("  => {}", v);
                    v
                },
                GlobalInst::Import { global_type: _, import_index: _ } => {
                    unimplemented!()
                }
            }
        })        
    }
    pub fn set_global(&self, index: u32, new_value: i32) -> Result<(), Error> {
        Ok({
            info!("set_global({}, {})", index, new_value);
            if index as usize > self.globals.len() {
                return Err(Error::OutOfBounds);
            }
            match self.globals[index as usize] {
                GlobalInst::Local { global_type: _, global_index: _, ref value } => {
                    value.set(Value(new_value))
                },
                GlobalInst::Import { global_type: _, import_index: _ } => {
                    unimplemented!()
                }
            }
        })        
    }    
    pub fn code(&self) -> &CompiledCode {
        &self.code
    }

    // pub fn body(&self, index: usize) -> Option<Body> {
    //     self.m.body(index as u32)
    // }
}

#[derive(Debug)]
pub enum FuncInst<'a> {
    Host { type_index: usize, module: &'a str, name: &'a str, host_index: usize },
    Import { type_index: usize, module: &'a str, name: &'a str, module_index: usize, import_index: usize },
    Local { type_index: usize, function_index: usize },
}

impl<'a> FuncInst<'a> {
    pub fn type_index(&self) -> usize {
        match self {
            &FuncInst::Host { type_index, module: _, name: _, host_index: _ } => type_index,
            &FuncInst::Import { type_index, module: _, name: _, module_index: _, import_index: _ } => type_index,
            &FuncInst::Local { type_index, function_index: _ } => type_index,
        }
    }
}

#[derive(Debug)]
pub enum GlobalInst {
    Import { global_type: GlobalType, import_index: usize },
    Local { global_type: GlobalType, global_index: usize, value: Cell<Value> },
}

impl GlobalInst {
    pub fn global_type(&self) -> GlobalType {
        match self {
            &GlobalInst::Import { global_type, import_index: _ } => global_type,
            &GlobalInst::Local { global_type, global_index: _, value: _ } => global_type,
        }
    }
}

#[derive(Debug)]
pub struct ExportInst<'buf> {
    pub name: &'buf str,
    pub export_desc: ExportDesc,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Value(pub i32);

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "i32:{}", self.0 as u32)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}



impl From<i32> for Value {
    fn from(other: i32) -> Value {
        Value(other)
    }
}

impl From<u32> for Value {
    fn from(other: u32) -> Value {
        Value(other as i32)
    }
}

impl From<Value> for i32 {
    fn from(other: Value) -> i32 {
        other.0
    }
}

impl From<Value> for u32 {
    fn from(other: Value) -> u32 {
        other.0 as u32
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_module_inst() {
    //     let mut buf = [0u8; 2048];

    //     let mut m = Module::new();
    //     m.set_name("hello.wasm");

    //     let (mi, _buf) = ModuleInst::new(&m, &mut buf).unwrap();
    //     assert_eq!(mi.name(), "hello.wasm");
    // }

    #[test]
    fn test_copy_types() {
        use opcode::I32;

        let mut buf = [0u8; 1024];
        let mut w = Writer::new(&mut buf);

        let t_new = {
            let parameters = &[I32 as u8, I32 as u8][..];
            let returns = &[I32 as u8][..];
            let t = Type { parameters, returns };
            Type {
                parameters: w.copy_slice(t.parameters).unwrap(),
                returns: w.copy_slice(t.returns).unwrap(),
            }
        };
        assert_eq!(t_new.parameters.len(), 2);
        assert_eq!(t_new.returns.len(), 1);
    }


    #[test]
    fn test_build_type_list() {
        use opcode::{I32, I64};
        use {Error, ValueType};

        trait WriteTo<W, E> {
            fn write_to(&self, w: &mut W) -> Result<(), E>; 
        }

        impl<'buf> WriteTo<Writer<'buf>, Error> for ValueType {
            fn write_to(&self, w: &mut Writer<'buf>) -> Result<(), Error> {
                w.write_i8(*self as i8)
            }
        }

        impl<'buf, W, T, E> WriteTo<W, E> for &'buf [T] where T: WriteTo<W, E> {
            fn write_to(&self, w: &mut W) -> Result<(), E> {
                for item in self.iter() {
                    item.write_to(w)?;
                }
                Ok(())
            }
        }

        let src = &[I32, I64][..];

        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);

        src.write_to(&mut w).unwrap();

        // for t in src {
        //     (*t).write_to(&mut w).unwrap();
        // }
        let _dst: &[i8] = w.split();

    }
}