extern crate wasm_machine as wasm;
extern crate clap;

use std::process;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

use clap::{App, Arg, ArgMatches};

use wasm::{Reader, BinaryReader};

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
    let matches = App::new("dump")
        .arg(Arg::with_name("path")
            .required(true))
        .arg(Arg::with_name("headers")
            .short("h"))
        .arg(Arg::with_name("details")
            .short("x"))
        .arg(Arg::with_name("disassemble")
            .short("d"))
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

    if matches.is_present("headers") {
        let r = Reader::new(&mut data[..]);
        let mut d = wasm::dumper::HeaderDumper{};
        BinaryReader::new(&mut d, r).load(path)?;        
    } 
    
    if matches.is_present("details") {
        let r = Reader::new(&mut data[..]);
        let mut d = wasm::dumper::DetailsDumper{};
        BinaryReader::new(&mut d, r).load(path)?;                
    }

    if matches.is_present("disassemble") {
        let r = Reader::new(&mut data[..]);
        let mut d = wasm::dumper::Disassembler::new();
        BinaryReader::new(&mut d, r).load(path)?;                
    }

    Ok(())
}