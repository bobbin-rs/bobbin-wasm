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

    let mut cfg = wasm::loader::Config::default();
    if matches.is_present("no-compile") {
        cfg.compile = false;
    }

    let mut module_buf = [0u8; 4096];
    let r = Reader::new(&mut data[..]);
    
    let mut loader = wasm::loader::Loader::new_with_config(cfg, &mut module_buf[..]);
    BinaryReader::new(&mut loader, r).read(path)?;        
    let (m, buf) = loader.module();
    if matches.is_present("dump") {
        print!("{:?}", m);
    }
    println!("remaining: {}", buf.len());
    let (mi, _buf) = m.instantiate(buf)?;
    println!("mi: {}", mi.name());
    println!("types");

    for (i, t) in mi.types().iter().enumerate() {
        println!("  {}: {:?} -> {:?}", i, t.parameters, t.return_type());
    }
    println!("functions");
    for (i, f) in mi.functions().iter().enumerate() {
        println!("  {}: {:?}", i, f);
    }
    println!("globals");
    for (i, g) in mi.globals().iter().enumerate() {
        println!("  {}: {:?}", i, g);
    }

    // Interpreter

    use wasm::interp::Interp;
    let mut buf = [0u8; 1024];

    let mut interp = Interp::new(&mut buf);

    interp.run(&mi, 0).unwrap();

    println!("Stack Dump (top first):");

    let mut i = 0;
    while let Ok(value) = interp.pop() {
        println!("{}: {:?}", i, value);
        i += 1;
    }
    println!("---");
    
    Ok(())
}