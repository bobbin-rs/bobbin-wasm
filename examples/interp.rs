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

use wasm::{ExportDesc};
use wasm::interp;
use wasm::memory_inst::MemoryInst;
use wasm::module::Module;


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

// pub struct Environment<'a,> {
//     memory: MemoryInst<'a>,
// }

// impl<'a> Environment<'a> {
//     pub fn new(memory: MemoryInst<'a>) -> Self {   
//         Environment { memory }
//     }
// }

// pub struct Executor<'a> {    
//     env: Environment<'a>
// }

// impl<'a> Executor<'a> {
//     pub fn new(env: Environment<'a>) -> Self {
//         Executor {}
//     }
// }


pub fn run(matches: ArgMatches) -> Result<(), Error> {
    let path = Path::new(matches.value_of("path").unwrap());
    let mut file = File::open(&path)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let path = path.file_name().unwrap().to_str().unwrap();

    info!("loading {}", path);

    let _out = String::new();


    let mut memory_buf = [0u8; 256];
    let memory = MemoryInst::new(&mut memory_buf, 1, None);

    // let env = Environment::new(memory);

    let m = Module::from(data.as_ref());

    let out_buf = &mut [0u8; 4096];

    let (mi, _out_buf) = m.instantiate(out_buf, &memory)?;

    // Interpreter

    let interp_buf = &mut [0u8; 1024];

    let mut interp = interp::Interp::new(interp_buf);

    if let Some(export_section) = mi.module().export_section() {
        for e in export_section.iter() {
            if let ExportDesc::Function(index) = e.export_desc {
                let id = e.identifier;
                match interp.call(&mi, index as usize) {
                    Ok(Some(value)) => {
                        println!("{}() => {:?}", id, value);
                    },
                    Ok(None) => {
                        println!("{}() => nil", id);
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        println!("---- Stack Dump ----");

                        let mut i = 0;
                        while let Ok(value) = interp.pop() {
                            println!("{}: {:?}", i, value);
                            i += 1;
                        }
                        println!("---- END ----");
                    }
                }
            }
        }
    }
    
    Ok(())
}

