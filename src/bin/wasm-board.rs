extern crate bobbin_wasm as wasm;
extern crate clap;
extern crate log;
extern crate env_logger;

use std::process;
use std::io::{self, stdout, Read, Write};
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

// use log::Level;
use clap::{App, Arg, ArgMatches};

use wasm::{ExportDesc, ImportDesc};
use wasm::interp::Interp;
use wasm::environ::{Environment, HostHandler};
use wasm::module_inst::*;
use wasm::memory_inst::MemoryInst;

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
        .get_matches();
    
    if let Err(e) = run(matches) {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

pub struct BoardHandler {}

pub const WRITE_FN: usize = 0x0;
pub const LED_FN: usize = 0x1;
pub const DELAY_FN: usize = 0x2;

impl HostHandler for BoardHandler {
    fn import(&self, _module: &str, export: &str, _import_desc: &ImportDesc) -> Result<usize, wasm::Error> {
        Ok({
            match export {
                "write" => WRITE_FN,
                "led" => LED_FN,
                "delay" => DELAY_FN,
                _ => return Err(wasm::Error::InvalidImport)
            }
        })
    }

    fn dispatch(&self, interp: &mut Interp, mem: &MemoryInst, _type_index: usize, index: usize) -> Result<(), wasm::Error> {
        Ok({ 
            match index {
                WRITE_FN => {
                    let len = interp.pop()? as usize;
                    let ptr = interp.pop()? as usize;
                    let buf = &mem.as_ref()[ptr..ptr+len];
                    stdout().write_all(buf).unwrap();

                },
                LED_FN => {
                    let arg = interp.pop()?;
                    if arg == 0 {
                        println!("[LED OFF]");
                    } else {
                        println!("[LED ON]");
                    }
                },
                DELAY_FN => {
                    let arg = interp.pop()?;
                    if arg > 0 {
                        thread::sleep(Duration::from_millis(arg as u64));
                    }
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

    let h = BoardHandler {};

    let buf = &mut [0u8; 8192];
    let (buf, mut env) = Environment::new(buf, h);    

    let (buf, mi) = env.load_module(path, buf, data.as_ref())?;

    // Interpreter

    let mut interp = Interp::new(buf);

    for e in mi.exports() {
        // println!("export: {:?}", e);
        if let ExportDesc::Func(index) = e.export_desc {
            let id = e.name;
            if id != "main" {
                continue
            }
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
            break;
        }
    }
    
    Ok(())
}

