extern crate wasm;
extern crate clap;
// #[macro_use] extern crate log;
extern crate env_logger;

use std::process;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

use clap::{App, Arg, ArgMatches};

// use wasm::{Reader, BinaryReader};
use wasm::parser;
use wasm::visitor;
// use wasm::Module;

use std::fmt::{self, Write};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    FmtError(fmt::Error),
    ParserError(parser::Error),
    WasmError(wasm::Error),
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::IoError(other)
    }
}

impl From<fmt::Error> for Error {
    fn from(other: fmt::Error) -> Self {
        Error::FmtError(other)
    }
}

impl From<parser::Error> for Error {
    fn from(other: parser::Error) -> Self {
        Error::ParserError(other)
    }
}

impl From<wasm::Error> for Error {
    fn from(other: wasm::Error) -> Self {
        Error::WasmError(other)
    }
}

pub fn main() {
    env_logger::init();
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
    let name = path.file_name().unwrap().to_str().unwrap();
    let mut out = String::new();

    
    let m = parser::Module::new(data.as_ref())?;
    
    writeln!(out, "\n{}:\tfile format wasm 0x{:x}\n", name, m.version())?;
    
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