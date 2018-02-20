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
use wasm::interp::Interp;
use wasm::environ::Environment;
use wasm::module_inst::*;

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

fn host_hello(_interp: &mut Interp, index: usize) -> Result<(), wasm::Error> {
    Ok({ 
        println!("host_hello: {}", index);
    })
}



pub fn run(matches: ArgMatches) -> Result<(), Error> {
    let path = Path::new(matches.value_of("path").unwrap());
    let mut file = File::open(&path)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let path = path.file_name().unwrap().to_str().unwrap();

    info!("loading {}", path);

    let _out = String::new();

    let buf = &mut [0u8; 4096];
    let (buf, mut env) = Environment::new(buf);    

    env.register_host_function(host_hello)?;

    let (buf, mi) = env.load_module(buf, data.as_ref())?;

    // Interpreter

    let mut interp = Interp::new(buf);

    for e in mi.exports() {
        println!("export: {:?}", e);
        if let ExportDesc::Function(index) = e.export_desc {
            let id = &e.identifier;            
            match &mi.functions()[index as usize] {
                &FuncInst::Import { type_index: _, module: _, name: _, import_index: _ } => {
                    println!("Calling Import");
                },
                &FuncInst::Local { type_index: _, function_index } => {
                    println!("Calling Local Function {}", function_index);
                    match interp.call(&env, &mi, function_index as usize) {
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
                },
            }

        }
    }
    
    Ok(())
}

