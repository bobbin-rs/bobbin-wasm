extern crate wasm_machine as wasm;
extern crate clap;

use std::process;
use std::io::{self, Read};
use std::fs::File;

use clap::{App, Arg};

use wasm::{Reader, Writer, Module};

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
            .required(true)
        ).get_matches();
    
    let path = matches.value_of("path").unwrap();
    if let Err(e) = run(path) {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}

pub fn run(path: &str) -> Result<(), Error> {
    let mut file = File::open(&path)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let mut buf = [0u8; 64 * 1024];
    let r = Reader::new(&mut data[..]);
    let w = Writer::new(&mut buf);
    let mut m = Module::new(r, w);

    m.load()?;

    Ok(())
}