extern crate wasm_machine as wasm;
extern crate clap;

use std::process;
use std::io::{self, Read};
use std::fs::File;
use std::str;

use clap::{App, Arg};

use wasm::{Reader, Writer, ModuleLoader};

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
    let mut d = wasm::dumper::Dumper{};
    let r = Reader::new(&mut data[..]);
    let w = Writer::new(&mut buf);
    let _m = ModuleLoader::new(&mut d, r, w).load()?;



    // println!("----");
    // for s in m.iter() {
    //     println!("{:?}: {:04x}", s.section_type, s.buf.len());
    //     match s.section_type {
    //         SectionType::Type => {
    //             for t in s.types() {
    //                 println!("  p: {:?} r: {:?}", t.parameters, t.returns);
    //             }
    //         },
    //         SectionType::Function => {
    //             for f in s.functions() {
    //                 let t = m.function_signature_type(f.index).unwrap();
    //                 println!("  s: ({:?}) -> {:?}", t.parameters, t.returns);
    //             }
    //         }
    //         SectionType::Export => {
    //             for e in s.exports() {
    //                 println!("  identifier: {:?} index: {:?}", str::from_utf8(e.identifier).unwrap(), e.export_index);
    //             }
    //         },
    //         SectionType::Code => {
    //             for c in s.codes() {
    //                 println!("  len: {:04x}", c.body.len());
    //             }
    //         },
    //         SectionType::Data => {
    //             for d in s.data() {
    //                 println!("   {:04x} {:?}", d.offset_parameter, d.data);
    //             }
    //         }
    //         _ => {},
    //     }
    // }

    Ok(())
}