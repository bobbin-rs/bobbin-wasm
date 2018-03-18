extern crate wasm;
extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;

use std::process;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

use clap::{App, Arg, ArgMatches};

// use wasm::{Reader, BinaryReader};
use wasm::visitor;
use wasm::Module;

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
    info!("running!");
    let matches = App::new("dump")
        .arg(Arg::with_name("path")
            .required(true))
        .arg(Arg::with_name("headers")
            .long("headers")
            .short("h"))
        .arg(Arg::with_name("details")
            .long("details")
            .short("x"))
        .arg(Arg::with_name("disassemble")
            .long("disassemble")
            .short("d"))
        .arg(Arg::with_name("relocations")
            .short("r"))
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

    // let path = path.file_name().unwrap().to_str().unwrap();
    let mut out = String::new();

    
    let m = Module::from(data.as_ref());
    
    if matches.is_present("headers") {        
    
        let mut d = wasm::dumper::HeaderDumper{ w: &mut out };
        
        visitor::visit(&m, &mut d)?;
        
    } 
    
    if matches.is_present("details") {
        let mut d = wasm::dumper::DetailsDumper{ w: &mut out };
        visitor::visit(&m, &mut d)?;
    }

    if matches.is_present("disassemble") {        
        let mut d = wasm::dumper::Disassembler::new(&mut out );
        visitor::visit(&m, &mut d)?;
    }
    print!("{}", out);

    Ok(())
}