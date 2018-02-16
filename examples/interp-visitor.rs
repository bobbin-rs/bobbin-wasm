#![feature(try_from)]

extern crate wasm;
extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;

use std::process;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

// use log::Level;
use clap::{App, Arg, ArgMatches};

use wasm::{TypeValue, SectionType, ExportDesc};
use wasm::memory_inst::MemoryInst;
use wasm::inplace::{self, visitor, Module};


#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    WasmError(wasm::Error),
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::IoError(other)
    }
}

impl From<wasm::Error> for Error {
    fn from(other: wasm::Error) -> Self {
        Error::WasmError(other)
    }
}

pub fn main() {
    env_logger::init();
    let matches = App::new("interp")
        .arg(Arg::with_name("path")
            .required(true))
        .arg(Arg::with_name("dump").long("dump"))
        .arg(Arg::with_name("no-compile").long("no-compile"))
        .get_matches();
    
    if let Err(e) = run(matches) {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

pub fn run(matches: ArgMatches) -> Result<(), Error> {
    let path = Path::new(matches.value_of("path").unwrap());
    let mut file = File::open(&path)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let path = path.file_name().unwrap().to_str().unwrap();

    info!("loading {}", path);

    let _out = String::new();


    let mut module_buf = [0u8; 4096];
    let m = Module::from(data.as_ref());

    let cfg = wasm::loader::Config::default();
    let mut loader = wasm::loader::Loader::new_with_config(cfg, &mut module_buf[..]);
    visitor::visit(&m, &mut loader)?;

    let (lm, buf) = loader.module();


    let mut code_buf = [0u8; 4096];
    let cfg = wasm::compiler::Config::default();
    let mut compiler = wasm::compiler::Compiler::new_with_config(cfg, &mut code_buf[..]);
    let code = compiler.compile(&m)?;
    println!("CODE");
    for (i, b) in code.iter().enumerate() {
        println!("  {}: {:08x} to {:08x}", i, b.start, b.end);
    }

    if matches.is_present("dump") {
        print!("{:?}", lm);
    }

    let mut memory_buf = [0u8; 256];
    let memory = MemoryInst::new(&mut memory_buf, 1, None);

    // let (mi, _buf) = lm.instantiate(buf, &memory)?;


    let (mi, _buf) = m.instantiate(buf, &memory, &code)?;


    // Interpreter

    use wasm::interp::Interp;
    use std::convert::TryFrom;

    let mut buf = [0u8; 1024];

    let mut interp = inplace::interp::Interp::new(&mut buf);

    for s in lm.iter() {
        if s.section_type == SectionType::Export {
            for e in s.exports() {

                if let ExportDesc::Function(index) = e.export_desc {
                    let t = lm.function_signature_type(index).unwrap();
                    let id = std::str::from_utf8(e.identifier.0).unwrap();
                    match interp.run(&mi, index as usize) {
                        Ok(_) => {
                            if t.returns.len() == 1 {
                                if interp.stack_len() == 1 {
                                    println!("{}() => {}:{}", id, TypeValue::try_from(t.returns[0]).unwrap(), interp.pop().unwrap() as u32);
                                } else {
                                    println!("---- Stack Dump ----");

                                    let mut i = 0;
                                    while let Ok(value) = interp.pop() {
                                        println!("{}: {:?}", i, value);
                                        i += 1;
                                    }
                                    println!("---- END ----");
                                }
                            }
                        }, 
                        Err(err) => {
                            println!("{}() => {:?}", id, err);
                        }
                    }


                }
            }
            
        }
    }


    
    Ok(())
}