use {Error, SectionType};
// use types::Identifier;
use module::*;
use small_vec::SmallVec;
use writer::Writer;

pub struct ModuleInst<'a> {
    name: &'a str,
    types: SmallVec<'a, Type<'a>>,
    functions: SmallVec<'a, Function>,
    globals: SmallVec<'a, Global>,
}

impl<'a> ModuleInst<'a> {
    pub fn new(m: &Module, buf: &'a mut [u8]) -> Result<(Self, &'a mut [u8]), Error> {
        let mut w = Writer::new(buf);
        let name = w.copy_str(m.name());

        let mut types = w.alloc_smallvec(16);
        let mut functions = w.alloc_smallvec(16);
        let mut globals = w.alloc_smallvec(16);
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
                    for i in section.imports() {
                        if let ImportDesc::Type(signature_type_index) = i.desc {
                            functions.push(Function { signature_type_index })
                        }
                    }
                },
                SectionType::Function => {
                    for t in section.functions() {
                        functions.push(t);
                    }
                },                
                SectionType::Global => {
                    for t in section.globals() {
                        globals.push(t);
                    }
                },                                

                _ => {},
            }
        }


        Ok((ModuleInst { name, types, functions, globals }, w.into_slice()))
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn types(&self) -> &[Type] {
        self.types.as_ref()
    }

    pub fn functions(&self) -> &[Function] {
        self.functions.as_ref()
    }

    pub fn globals(&self) -> &[Global] {
        self.globals.as_ref()
    }

    pub fn type_signature(&self, index: usize) -> &Type {
        &self.types[index]
    }
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