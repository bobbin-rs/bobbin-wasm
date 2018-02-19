extern crate wasm;

use wasm::parser::*;


fn main() {
    match run() {
        Ok(_) => {},
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn run() -> Result<(), Error> {
    let data = include_bytes!("../local_test/basic.wasm");
    let m = Module::new(data)?;
    validator::validate(&m)?;
    let mut sections = m.sections();
    while let Some(section) = sections.next()? {
        println!("{:?}", section.id);
        match section.id {
            Id::Type => {
                let mut function_types = section.function_types();
                let n = 0;
                while let Some(t) = function_types.next()? {
                    println!(" {}:", n);
                    for t in t.parameters {
                        println!("    p: {:?}", t);
                    }
                    for t in t.results {
                        println!("    r: {:?}", t);
                    }
                }
            },
            Id::Function => {
                let mut functions = section.functions();
                let mut n = 0;
                while let Some(e) = functions.next()? {
                    println!("  func[{}]: {:?}", n, e);
                    n += 1;
                }
            },     
            Id::Table => {
                let mut tables = section.tables();
                while let Some(t) = tables.next()? {
                    println!("  Table: {:?}", t);
                }
            },     
            Id::Memory => {
                let mut memory = section.memory();
                while let Some(t) = memory.next()? {
                    println!("  Memory: {:?}", t);
                }
            },                                 
            Id::Export => {
                let mut exports = section.exports();
                while let Some(e) = exports.next()? {
                    println!("  {:?}", e);
                }
            },
            Id::Element => {
                let mut segment = 0;
                let mut elements = section.elements();
                while let Some(e) = elements.next()? {
                    println!(" - segment[{}] table={}", segment, e.table_index);
                    println!(" - init {:?}", e.offset);
                    let mut i = 0;
                    let mut items = e.iter();
                    while let Some(item) = items.next()? {
                        println!("  - elem[{}] = func[{}]", i, item);
                        i += 1;
                    }
                    segment += 1;
                }
            },      
            Id::Code => {
                let mut func_index = 0;
                let mut code_section = section.code();
                while let Some(code) = code_section.next()? {
                    let func_offset = m.offset_to(code.buf);
                    println!("{:06x} func[{}]:", func_offset, func_index);
                    let mut indent = 0;
                    let mut item_iter = code.func.iter();
                    while let Some(item) = item_iter.next()? {
                        match item {
                            FuncItem::Local(_local) => {
                                // println!("{:?}", local);
                            },
                            FuncItem::Instr(instr) => {
                                use opcode::*;

                                match instr.opcode {
                                    ELSE | CATCH | CATCH_ALL | END => {
                                        indent -= 1;
                                    },
                                    _ => {}
                                }


                                let offset = m.offset_to(instr.data);
                                print!(" {:06x}:", offset);
                                let data = if instr.data.len() > 10 {
                                    &instr.data[..10]
                                } else {
                                    instr.data
                                };
                                let mut n = 0;
                                for b in data {
                                    print!(" {:02x}", b);
                                    n += 1;                                    
                                 }
                                 while n < 9 {
                                    print!("   ");
                                    n += 1;
                                 }
                                 print!(" | ");
                                 for _ in 0..indent {
                                     print!("  ");
                                 }
                                 println!("{:?}", instr);

                                 match instr.opcode {
                                     BLOCK | LOOP | IF => {
                                         indent += 1;
                                     },
                                     _ => {},
                                 }
                            }
                        }
                    }
                    func_index += 1;
                }
            },                    
            Id::Data => {
                let mut segment = 0;
                let mut data = section.data();
                while let Some(e) = data.next()? {
                    println!(" - segment[{}] size={} - init {:?}", segment, e.init.len(), e.offset);
                    // println!("  {:?}", e);
                    segment += 1;
                }
            },  
            Id::Custom => {
                // println!("{:?}", section);
                let mut custom = section.custom()?;
                println!("  {:?}", custom);
            },                          
            _ => {},
        }
    }
    Ok(())
}