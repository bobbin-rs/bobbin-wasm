extern crate wasm;
extern crate clap;
extern crate log;
extern crate env_logger;

use std::process;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

// use log::Level;
use clap::{App, Arg, ArgMatches};

use wasm::{ExportDesc, ImportDesc};
use wasm::interp::Interp;
use wasm::environ::{Environment, HostHandler};
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
        .arg(Arg::with_name("run-all-exports").long("run-all-exports"))
        .arg(Arg::with_name("host-print").long("host-print"))
        .get_matches();
    
    if let Err(e) = run(matches) {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

pub struct Handler {}

pub const HELLO_FN: usize = 0x0;
pub const PRINT_FN: usize = 0x1;
pub const ADD_FN: usize = 0x2;

impl HostHandler for Handler {
    fn import(&self, _module: &str, export: &str, _import_desc: &ImportDesc) -> Result<usize, wasm::Error> {
        Ok({
            match export {
                "hello" => HELLO_FN,
                "print" => PRINT_FN,
                "add" => ADD_FN,
                _ => return Err(wasm::Error::InvalidImport)
            }
        })
    }

    fn dispatch(&self, interp: &mut Interp, _type_index: usize, index: usize) -> Result<(), wasm::Error> {
        Ok({ 
            match index {
                HELLO_FN => println!("Hello, World"),
                PRINT_FN => {
                    let arg = interp.pop()?;
                    println!("called host host.print() => i32.{}", arg);
                },
                ADD_FN => {
                    let arg1 = interp.pop()?;
                    let arg2 = interp.pop()?;
                    let ret = arg1 + arg2;
                    println!("{:?} + {:?} -> {:?}", arg1, arg2, ret);
                    interp.push(ret)?;
                }
                _ => return Err(wasm::Error::InvalidFunction { id: index as u32 })
            }
        })
    }
    
}

#[allow(dead_code)]
fn load_file(file_name: &str) -> Result<Vec<u8>, Error> {
    let path = Path::new(file_name);
    let mut file = File::open(&path)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

pub fn run(matches: ArgMatches) -> Result<(), Error> {
    let path = Path::new(matches.value_of("path").unwrap());
    let mut file = File::open(&path)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let path = path.file_stem().unwrap().to_str().unwrap();

    let _out = String::new();

    let h = Handler {};

    let buf = &mut [0u8; 8192];
    let (buf, mut env) = Environment::new(buf, h);    


    // let math = load_file("local_test/math.wasm")?;

    // println!("loading {:?}", "math");

    // let (buf, _) = env.load_module("math", buf, math.as_ref())?;

    // println!("loading {:?}", path);



    let (buf, mi) = env.load_module(path, buf, data.as_ref())?;

    // Interpreter

    let mut interp = Interp::new(buf);

    if matches.is_present("run-all-exports") {

        for e in mi.exports() {
            // println!("export: {:?}", e);
            if let ExportDesc::Func(index) = e.export_desc {
                let id = &e.name;            
                match &mi.functions()[index as usize] {
                    &FuncInst::Local { type_index: _, function_index } => {
                        // println!("Calling Local Function {}", function_index);
                        match interp.call(&env, &mi, function_index as usize) {
                            Ok(Some(value)) => {
                                println!("{}() => {:?}", id, value);
                            },
                            Ok(None) => {
                                println!("{}() =>", id);
                            },
                            Err(wasm::Error::Unreachable) => {
                                println!("{}() => error: unreachable executed", id);
                            },
                            Err(wasm::Error::UndefinedTableIndex { id: _ }) => {
                                println!("{}() => error: undefined table index", id);
                            },
                            Err(wasm::Error::SignatureMismatch) => {
                                println!("{}() => error: indirect call signature mismatch", id);
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
                    f @ _ => {
                        println!("Unable to call {:?}", f);
                    }
                }

            }
        }
    }
    
    Ok(())
}

