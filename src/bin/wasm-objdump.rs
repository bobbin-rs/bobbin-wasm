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
use wasm::parser::{self, Id, Module, FallibleIterator, ExportDesc};
// use wasm::visitor;


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
        dump_headers(&mut out, &m)?;
        // let mut d = wasm::dumper::HeaderDumper{ w: &mut out };    
        // visitor::visit(&m, &mut d)?;
        
    } 
    
    if matches.is_present("details") {
        dump_details(&mut out, &m)?;
    //     let mut d = wasm::dumper::DetailsDumper{ w: &mut out };
    //     visitor::visit(&m, &mut d)?;
    }

    // if matches.is_present("disassemble") {        
    //     let mut d = wasm::dumper::Disassembler::new(&mut out );
    //     visitor::visit(&m, &mut d)?;
    // }
    print!("{}", out);

    Ok(())
}

pub fn dump_headers<W: Write>(out: &mut W, m: &Module) -> Result<(), Error> {
    writeln!(out, "Sections:")?;
    let mut sections = m.sections();
    while let Some(s) = sections.next()? {
        let s_id = s.id();
        let s_count = s.count()?;
        let s_beg = m.offset_to(s.buf);
        let s_len = s.buf.len();
        let s_end = s_beg + s_len;
        match s_id {
            Id::Custom => {     
                // let mut c = Cursor::new(data);
                // let s_name = c.read_identifier();
                let s_name = "";
                writeln!(out, "{:>9} start={:#010x} end={:#010x} (size={:#010x}) {:?}", s_id.as_str(), s_beg, s_end, s_len, s_name)?;
            },
            Id::Start => {     
                writeln!(out, "{:>9} start={:#010x} end={:#010x} (size={:#010x}) start: {}", s_id.as_str(), s_beg, s_end, s_len, s_count)?;
            },
            _ => {
                writeln!(out, "{:>9} start={:#010x} end={:#010x} (size={:#010x}) count: {}", s_id.as_str(), s_beg, s_end, s_len, s_count)?;
            }
        }
    }
    Ok(())
}

pub fn dump_details<W: Write>(out: &mut W, m: &Module) -> Result<(), Error> {
    writeln!(out, "Section Details:")?;    
    let mut sections = m.sections();
    while let Some(s) = sections.next()? {
        let s_id = s.id();
        if s_id != Id::Code && s_id != Id::Element {
            writeln!(out, "{}:", s_id.as_str())?;
        }
        match s_id {
            Id::Custom => {

            },
            Id::Type => {
                let mut types = s.types();
                let mut n = 0;
                while let Some(t) = types.next()? {
                    write!(out, " - type[{}] (", n)?;
                    for (n, p) in t.parameters.iter().enumerate() {
                        if n > 0 { write!(out, ", ")? }
                        write!(out, "{}", p)?;
                    }
                    write!(out, ")")?;
                    write!(out, " ->")?;
                    if t.results.len() == 0 {
                        write!(out, " nil")?
                    } else {
                        write!(out, " {}", t.results[0])?;
                    }
                    writeln!(out, "")?;
                    n += 1;
                }
            },
            Id::Import => {
            },
            Id::Function => {
                let mut funcs = s.functions();
                let mut n = 0;
                while let Some(f) = funcs.next()? {
                    writeln!(out," - func[{}] sig={}", n, f)?;
                    n += 1;
                }
            },
            Id::Table => {

            },
            Id::Memory => {

            },
            Id::Global => {

            },
            Id::Export => {
                let mut exports = s.exports();
                let mut n = 0;
                while let Some(e) = exports.next()? {
                    let kind = match e.export_desc {
                        ExportDesc::Func(_) => "func",
                        ExportDesc::Table(_) => "table",
                        ExportDesc::Memory(_) => "memory",
                        ExportDesc::Global(_) => "global",
                    };
                    writeln!(out, " - {}[{}] -> {:?}", kind, n, e.name)?;      
                }
            },
            Id::Start => {

            },
            Id::Element => {

            },
            Id::Code => {

            },
            Id::Data => {

            },
        }
        
    }
    Ok(())
}