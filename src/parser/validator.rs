use parser::error::Error;
use parser::reader::FallibleIterator;
use parser::module::{Module, Id, FuncItem};

pub fn validate(m: &Module) -> Result<(), Error> {
    if m.magic != &[0x00, 0x61, 0x73, 0x6d] {
        return Err(Error::InvalidMagic)
    }

    if m.version != &[0x01, 0x00, 0x00, 0x00] {
        return Err(Error::InvalidVersion)
    }

    let mut sections = m.sections();
    while let Some(section) = sections.next()? {
        match section.id {
            Id::Type => {
                let mut function_types = section.function_types();
                while let Some(t) = function_types.next()? {
                    for _ in t.parameters {
                    }
                    for _ in t.results {
                    }
                }
            },
            Id::Function => {
                let mut functions = section.functions();
                while let Some(_) = functions.next()? {
                }
            },     
            Id::Table => {
                let mut tables = section.tables();
                while let Some(_) = tables.next()? {
                }
            },     
            Id::Memory => {
                let mut memory = section.memory();
                while let Some(_) = memory.next()? {
                }
            },                                 
            Id::Export => {
                let mut exports = section.exports();
                while let Some(_) = exports.next()? {
                }
            },
            Id::Element => {
                let mut elements = section.elements();
                while let Some(element) = elements.next()? {
                    let mut items = element.iter();
                    while let Some(_) = items.next()? {
                    }
                }
            },      
            Id::Code => {
                let mut code_section = section.code();
                while let Some(code) = code_section.next()? {
                    let mut item_iter = code.func.iter();
                    while let Some(item) = item_iter.next()? {
                        match item {
                            FuncItem::Local(_local) => {
                            },
                            FuncItem::Instr(_instr) => {
                            }
                        }
                    }
                }
            },                    
            Id::Data => {
                let mut data = section.data();
                while let Some(_d) = data.next()? {
                }
            },  
            Id::Custom => {
                let _custom = section.custom()?;
            },                          
            _ => {},
        }
    }


    Ok(())
}