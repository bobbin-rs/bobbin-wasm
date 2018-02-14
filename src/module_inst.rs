use {Error, SectionType, Value};
// use types::Initializer;
use module::*;
use core::cell::Cell;
use memory_inst::MemoryInst;
use small_vec::SmallVec;
use writer::Writer;

pub struct ModuleInst<'m, 'a, 'mem> {
    name: &'a str,
    m: &'m Module<'m>,
    types: SmallVec<'a, Type<'a>>,
    functions: SmallVec<'a, FuncInst>,
    globals: SmallVec<'a, GlobalInst>,
    tables: SmallVec<'a, SmallVec<'a, u32>>,
    memory_inst: &'mem MemoryInst<'mem>,
}

impl<'m, 'a, 'mem> ModuleInst<'m, 'a, 'mem> {
    pub fn new(m: &'m Module<'m>, buf: &'a mut [u8], memory_inst: &'mem MemoryInst<'mem>) -> Result<(Self, &'a mut [u8]), Error> {
        let mut w = Writer::new(buf);
        let name = w.copy_str(m.name());

        let mut types = w.alloc_smallvec(16);
        let mut functions = w.alloc_smallvec(16);
        let mut globals = w.alloc_smallvec(16);
        let mut tables = w.alloc_smallvec(16);
        // let mut imports = w.alloc_smallvec(16);

        for section in m.iter() {
            match section.section_type {
                SectionType::Type => {
                    for t in section.types() {
                        let parameters= w.copy_slice(t.parameters)?;
                        let returns = w.copy_slice(t.returns)?;
                        types.push(Type { parameters, returns });
                    }
                },
                SectionType::Import => {
                    for (import_index, i) in section.imports().enumerate() {
                        match i.desc {
                            ImportDesc::Type(type_index) => {
                                let type_index = type_index as usize;
                                functions.push(FuncInst::Import { type_index, import_index });
                            },
                            ImportDesc::Table(_) => {
                                // info!("Import Table");
                            },
                            ImportDesc::Memory(_) => {
                                // info!("Import Memory");
                            },
                            ImportDesc::Global(global_type) => {
                                // info!("Import Global");
                                globals.push(GlobalInst::Import { global_type, import_index});
                            }
                        }
                    }
                },
                SectionType::Function => {
                    for (function_index, function) in section.functions().enumerate() {
                        let type_index = function.signature_type_index as usize;
                        functions.push(FuncInst::Local { type_index, function_index });
                    }
                },
                SectionType::Global => {
                    for (global_index, global) in section.globals().enumerate() {
                        let global_type = global.global_type;
                        let init = global.init;
                        let value = Cell::new(init.value()?);
                        globals.push(GlobalInst::Local { global_type, global_index, value });
                    }
                },
                SectionType::Table => {
                    for Table { element_type, limits } in section.tables() {
                        info!("Adding table: {} {:?}", element_type, limits);
                        let size = if let Some(max) = limits.max {
                            max
                        } else {
                            limits.min
                        };
                        let t: SmallVec<u32> = w.alloc_smallvec(size as usize);
                        tables.push(t);
                    }
                },                
                SectionType::Element => {
                    for Element { table_index, offset, data } in section.elements() {
                        use byteorder::{ByteOrder, LittleEndian};

                        info!("Initializing table {}", table_index);
                        let table = &mut tables[table_index as usize];
                        let Value(offset) = offset.value()?;
                        let mut i = 0;
                        let mut o = offset as usize;
                        while i < data.len() {
                            let d = LittleEndian::read_u32(&data[i..]);
                            info!("{:08x}: {:08x}", o, d);
                            table[o] = d;
                            o += 1;
                            i += 4;
                        }              
                    }
                },
                SectionType::Data => {
                    for Data{ memory_index: _, offset, data } in section.data() {
                        let Value(offset) = offset.value()?;
                        for i in 0..data.len() {
                            let d = data[i];
                            let o = offset as usize + i;
                            // info!("{:08x}: {:02x}", o, d);
                            memory_inst.set(o, d);
                        }
                    }
                },
                _ => {},
            }
        }


        Ok((ModuleInst { name, m, types, functions, globals, tables, memory_inst }, w.into_slice()))
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn module(&self) -> &'m Module<'m> {
        self.m
    }

    pub fn types(&self) -> &[Type] {
        self.types.as_ref()
    }

    pub fn functions(&self) -> &[FuncInst] {
        self.functions.as_ref()
    }

    pub fn globals(&self) -> &[GlobalInst] {
        self.globals.as_ref()
    }

    pub fn type_signature(&self, index: usize) -> &Type {
        &self.types[index]
    }

    pub fn memory_inst(&self) -> &MemoryInst {
        self.memory_inst
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

    // pub fn body(&self, index: usize) -> Option<Body> {
    //     self.m.body(index as u32)
    // }
}

#[derive(Debug)]
pub enum FuncInst {
    Import { type_index: usize, import_index: usize },
    Local { type_index: usize, function_index: usize },
}

#[derive(Debug)]
pub enum GlobalInst {
    Import { global_type: GlobalType, import_index: usize },
    Local { global_type: GlobalType, global_index: usize, value: Cell<Value> },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_inst() {
        let mut buf = [0u8; 2048];

        let mut m = Module::new();
        m.set_name("hello.wasm");

        let (mi, _buf) = ModuleInst::new(&m, &mut buf).unwrap();
        assert_eq!(mi.name(), "hello.wasm");
    }

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
        use {Error, TypeValue};

        trait WriteTo<W, E> {
            fn write_to(&self, w: &mut W) -> Result<(), E>; 
        }

        impl<'a> WriteTo<Writer<'a>, Error> for TypeValue {
            fn write_to(&self, w: &mut Writer<'a>) -> Result<(), Error> {
                w.write_i8(*self as i8)
            }
        }

        impl<'a, W, T, E> WriteTo<W, E> for &'a [T] where T: WriteTo<W, E> {
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