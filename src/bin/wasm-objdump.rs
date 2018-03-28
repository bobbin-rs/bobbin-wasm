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
use wasm::parser::{self, Id, Module, FallibleIterator, ExportDesc, ImportDesc, Immediate, FuncItem, Instr, Local};
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

    if matches.is_present("disassemble") {        
        dump_code(&mut out, &m)?;
    //     let mut d = wasm::dumper::Disassembler::new(&mut out );
    //     visitor::visit(&m, &mut d)?;
    }
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
    let mut import_funcs = 0;
    let mut import_tables = 0;
    let mut import_memory = 0;
    let mut import_globals = 0;    
    while let Some(s) = sections.next()? {
        let s_id = s.id();
        if s_id != Id::Code && s_id != Id::Custom {
            writeln!(out, "{}:", s_id.as_str())?;
        }
        match s_id {
            Id::Custom => {},
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
                let mut imports = s.imports();
                let mut n = 0;
                while let Some(i) = imports.next()? {
                    let module = i.module;
                    let name = i.name;
                    let desc = i.import_desc;
                    match desc {
                        ImportDesc::Func(f) => {
                            writeln!(out, " - func[{}] sig={} <{}> <- {}.{}", n, f, name, module, name)?;
                            import_funcs += 1;
                        },
                        ImportDesc::Table(t) => {
                            writeln!(out, " - table[{}] elem_type=anyfunc init={} max={:?} <- {}.{}", n, t.limits.min, t.limits.max, module, name)?;
                            import_tables += 1;
                        },
                        ImportDesc::Memory(m) => {
                            writeln!(out, " - memory[{}] pages: initial={} max={:?} <- {}.{}", n, m.limits.min, m.limits.max, module, name)?;
                            import_memory += 1;
                        },
                        ImportDesc::Global(g) => {
                            writeln!(out, " - global[{}] {} mutable={} <- {}.{}", n, g.valtype, if g.mutable { 1 } else { 0 }, module, name)?;                        
                            import_globals += 1;
                        },                    
                    }
                    n += 1;
                }
            },
            Id::Function => {
                let mut funcs = s.functions();
                let mut n = import_funcs;
                while let Some(f) = funcs.next()? {
                    writeln!(out," - func[{}] sig={}", n, f)?;
                    n += 1;
                }
            },
            Id::Table => {
                let mut tables = s.tables();
                let mut n = import_tables;
                while let Some(t) = tables.next()? {
                    let element_type = t.elemtype;
                    let limits = t.limits;
                    let limits_max = if let Some(limits_max) = limits.max {
                        limits_max
                    } else {
                        limits.min
                    };
                    writeln!(out, " - table[{}] type={} initial={} max={}", n, element_type, limits.min, limits_max)?;                    
                    n += 1;
                }
            },
            Id::Memory => {
                let mut memory = s.memory();
                let mut n = import_memory;
                while let Some(m) = memory.next()? {
                    let limits = m.limits;
                    write!(out, " - memory[{}] pages: initial={}", n, limits.min)?;
                    if let Some(maximum) = limits.max {
                        write!(out, " maximum={}", maximum)?;
                    }
                    writeln!(out, "")?;
                    
                    n += 1;
                }
            },
            Id::Global => {
                let mut globals = s.globals();
                let mut n = import_globals;
                while let Some(g) = globals.next()? {
                    let t = g.global_type;
                    let init = g.init;           
                    let instr = init.instr;         
                    let opcode = match instr.opcode {
                        0x41 => "i32",
                        0x42 => "i64",
                        0x43 => "f32",
                        0x44 => "f64",
                        0x23 => "global",
                        _ => unimplemented!()
                    };
                    writeln!(out, " - global[{}] {} mutable={} - init {}={:?}", n, t.valtype, if t.mutable { 1 } else { 0 }, opcode, instr.immediate)?;                    
                    n += 1;
                }
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
                let mut start = s.start()?;
                writeln!(out, " - start function: {}", start.func_index)?;
            },
            Id::Element => {
                let mut elements = s.elements();
                let mut n = 0;
                while let Some(e) = elements.next()? {
                    let imm = if let Immediate::I32Const { value } = e.offset.instr.immediate {
                        value
                    } else {
                        // FIXME
                        panic!("invalid immediate type");
                    };
                    
                    writeln!(out,  " - segment[{}] table={}", n, e.table_index)?;
                    writeln!(out,  " - init {}={}", "i32", imm)?;
                    for i in 0..e.init.len() {
                        writeln!(out,  "  - elem[{}] = func[{}]", i, e.init[i])?;
                    }
                    n += 1;
                }
            },
            Id::Code => {},
            Id::Data => {
                let mut data = s.data();
                let mut n = 0;
                while let Some(d) = data.next()? {                    
                    let offset = d.offset;
                    let imm = if let Immediate::I32Const { value } = offset.instr.immediate {
                        value
                    } else {
                        // FIXME
                        panic!("invalid immediate type");
                    };
                    let init = d.init;
                    writeln!(out, " - segment[{}] size={} - init {}={} ", n, init.len(), "i32", imm)?;
                    write!(out, "  - {:07x}:", imm)?;
                    for (i, d) in init.iter().enumerate() {
                        if i % 2 == 0 {
                            write!(out, " ")?;
                        }
                        write!(out, "{:02x}", d)?;
                    }
                    writeln!(out, "")?;                    
                    n += 1;
                }
            },
        }
        
    }
    Ok(())
}

pub fn dump_code<W: Write>(out: &mut W, m: &Module) -> Result<(), Error> {
    use parser::opcode::*;

    writeln!(out, "Code Disassembly:")?;
    let mut sections = m.sections();
    let mut import_funcs = 0;
    
    while let Some(s) = sections.next()? {
        match s.id() {
            Id::Import => {
                let mut imports = s.imports();
                while let Some(i) = imports.next()? {
                    match i.import_desc {
                        ImportDesc::Func(_) => {
                            import_funcs += 1;
                        },
                        _ => {},
                    }
                }                
            },
            _ => {},
        }
    }

    let mut sections = m.sections();
    while let Some(s) = sections.next()? {
        let s_id = s.id();
        if s_id != Id::Code { continue }

        let mut code_iter = s.code();
        let mut n = import_funcs;
        while let Some(code) = code_iter.next()? {
            let offset = m.offset_to(code.buf);
            writeln!(out, "{:06x} func[{}]:", offset, n)?;

            let mut funcs = code.func.iter();
            let mut depth = 0;
            let mut local_count = 0usize;
            let mut local_index = 0usize;            
            while let Some(func_item) = funcs.next()? {
                match func_item {
                    FuncItem::Local(Local { n, t }) => {
                        let n = n as usize;
                        let offset = offset + 2 + local_count * 2;
                        write!(out, " {:06x}:", offset)?;
                        let mut w = 0;
                        write!(out, " {:02x} {:02x}", n, t as u8)?;
                        w += 6;
                        while w < 28 {
                            write!(out, " ")?;
                            w += 1;
                        }
                        write!(out, "| ")?;                        
                        if n == 1 {
                            writeln!(out, "local[{}] type={}", local_index, t)?;
                        } else {
                            writeln!(out, "local[{}..{}] type={}",
                                local_index,
                                local_index + n - 1,
                                t
                            )?;
                        }
                        local_count += 1;
                        local_index += n;
                    },
                    FuncItem::Instr(Instr { opcode, immediate: imm, data}) => {
                        let offset = m.offset_to(data);
                        
                        let op = if let Some(op) = Op::from_opcode(opcode) {
                            op
                        } else {
                            panic!("Unrecognized opcode: {}", opcode);
                        };
                        match op.code {
                            ELSE | END => {
                                if depth > 0 {
                                    depth -= 1;
                                }
                            },
                            _ => {},
                        }
                        write!(out, " {:06x}:", offset)?;
                        let mut w = 0;
                        if op.code == I64_CONST {
                            for b in data.iter().take(10) {
                            write!(out, " {:02x}", b)?;
                                w += 3;
                            }
                            if w > 28 {
                                write!(out,  " ")?;
                            }
                        } else {
                            for b in data.iter() {
                            write!(out, " {:02x}", b)?;
                                w += 3;
                            }
                        }
                        while w < 28 {
                            write!(out, " ")?;
                            w += 1;
                        }
                        write!(out, "| ")?;
                        for _ in 0..depth { write!(out, "  ")?; }
                        match imm {
                            Immediate::None | Immediate::BranchTable { table: _ } => writeln!(out, "{}", op.text)?,
                            Immediate::Block { signature } => if signature != ::parser::ValueType::Void {
                                writeln!(out, "{} {}", op.text, signature)?
                            } else {
                                writeln!(out, "{}", op.text)?
                            },
                            _ => writeln!(out, "{} {:?}", op.text, imm)?,
                        }

                        match op.code {
                            BLOCK | LOOP | IF | ELSE => {
                                depth += 1;
                            },
                            _ => {},
                        }                
                    }
                    }
                }



            n += 1;
        }
    }
    Ok(())
}