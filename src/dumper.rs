use {SectionType, Delegate, DelegateResult, ExternalIndex, Event};
use opcode::*;
use opcode;

use core::str;
use core::fmt::Write;


pub struct HeaderDumper<W: Write> { pub w: W }

impl<W: Write> Delegate for HeaderDumper<W> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start { name, version } => {
                writeln!(self.w, "\n{}:\tfile format wasm 0x{:x}\n", name, version).unwrap();
            },
            SectionStart { s_type, s_beg, s_end, s_len } => {
                writeln!(self.w,"{:>9} start={:#010x} end={:#010x} (size={:#010x}) count: 1", s_type.as_str(), s_beg, s_end, s_len)?;
            },
            End => {
                writeln!(self.w,"")?;
            },
            _ => {},
        }
        Ok(())
    }
}

pub struct DetailsDumper<W: Write> { pub w: W }

impl<W: Write> Delegate for DetailsDumper<W> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start { name, version } => {
                writeln!(self.w, "\n{}:\tfile format wasm 0x{:x}\n", name, version).unwrap();
            },
            SectionStart { s_type, s_beg: _, s_end: _, s_len: _ } => {
                if s_type != SectionType::Code {
                    writeln!(self.w,"{}:", s_type.as_str())?;
                }
            },
            TypeStart { n, form: _ } => {
                write!(self.w," - type[{}] (", n)?;
            },
            TypeParameter { n, t } => {
                if n > 0 { write!(self.w,", ")? }
                write!(self.w,"{}", t)?;
            },
            TypeParametersEnd => {
                write!(self.w,") ->")?;
            }
            TypeReturnsStart { c } => {
                if c == 0 { write!(self.w, " nil")? }
            },
            TypeReturn { n: _, t } => {
                write!(self.w," {}", t)?;
            },
            TypeReturnsEnd => {
                writeln!(self.w,"")?;
            },
            Function { n, index } => {
                writeln!(self.w," - func[{}] sig={}", n, index.0)?;
            },
            Table { n, element_type, limits } => {
                writeln!(self.w," - table[{}] type={:?} initial={}", n, element_type, limits.min)?;
            },
            Mem { n, limits }=> {
                write!(self.w," - memory[{}] pages: initial={}", n, limits.min)?;
                if let Some(maximum) = limits.max {
                    write!(self.w," maximum={}", maximum)?;
                }
                writeln!(self.w,"")?;
            },
            Global { n, t, mutability, init } => {
                writeln!(self.w," - global[{}] {} mutable={} init 0x{:02x}={} ", n, t, mutability, init.opcode, init.immediate)?;
            },
            Export { n, id, index } => {
                let kind = match index {
                    ExternalIndex::Func(_) => "func",
                    ExternalIndex::Table(_) => "table",
                    ExternalIndex::Mem(_) => "memory",
                    ExternalIndex::Global(_) => "global",
                };
                writeln!(self.w," - {}[{}] -> {:?}", kind, n, str::from_utf8(id.0)?)?;            
            },
            Import { n, module, export, index } => {
                let (kind, i_type, index) = match index {
                    ExternalIndex::Func(i) => ("func", "sig", i.0),
                    ExternalIndex::Table(i) => ("table", "table", i.0),
                    ExternalIndex::Mem(i) => ("memory", "memory", i.0),
                    ExternalIndex::Global(i) => ("global", "global", i.0),
                };
                let module = str::from_utf8(module.0)?;
                let export = str::from_utf8(export.0)?;
                writeln!(self.w, " - {}[{}] {}[{}] <- {}.{}", kind, n, i_type, index, module, export)?;
            }
            
            StartFunction { index } => {
                writeln!(self.w," - start function: {}", index.0)?;
            },
            DataSegment {n, index: _, offset, data } => {
                writeln!(self.w," - segment[{}] size={} - init {}={} ", n, data.len(), "i32", offset.immediate)?;;
                write!(self.w," - {:07x}:", offset.immediate)?;
                for (i, d) in data.iter().enumerate() {
                    if i % 2 == 0 {
                        write!(self.w," ")?;
                    }
                    write!(self.w,"{:02x}", d)?;
                }
                writeln!(self.w,"")?;
            },
            End => writeln!(self.w,"")?,
            _ => {},
        }
        Ok(())
    }
}

pub struct Disassembler<W: Write> { 
    w: W,
    depth: usize,
}

impl<W: Write> Disassembler<W> {
    pub fn new(w: W) -> Self {
        Disassembler { w, depth: 0 }
    }
}

impl<W: Write> Delegate for Disassembler<W> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start { name, version } => {
                writeln!(self.w, "\n{}:\tfile format wasm 0x{:x}\n", name, version)?;
            },
            CodeStart { c: _ } => {
                writeln!(self.w,"Code Disassembly:\n")?;
            },
            Body { n, offset, size: _, locals: _ } => {
                writeln!(self.w,"{:06x} func[{}]:", offset, n)?;
            },
            Instruction(opcode::Instruction { offset, data, op, imm }) => {
                match op.code {
                    ELSE | END => {
                        if self.depth > 0 {
                            self.depth -= 1;
                        }
                    },
                    _ => {},
                }
                write!(self.w," {:06x}:", offset)?;
                let mut w = 0;
                for b in data.iter() {
                    write!(self.w," {:02x}", b)?;
                    w += 3;
                }
                while w < 28 {
                    write!(self.w," ")?;
                    w += 1;
                }
                write!(self.w,"| ")?;
                for _ in 0..self.depth { write!(self.w,"  ")?; }
                writeln!(self.w,"{}{:?}", op.text, imm)?;

                match op.code {
                    BLOCK | LOOP | IF | ELSE => {
                        self.depth += 1;
                    },
                    _ => {},
                }
            }
            _ => {},
        }
        Ok(())
    }
}