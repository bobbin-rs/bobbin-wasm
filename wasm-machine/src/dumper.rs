use {SectionType, Delegate, DelegateResult, ExternalIndex, Event};
use opcode::*;

use core::str;

fn print_start(name: &str, version: u32) {
    println!("\n{}:\tfile format wasm 0x{:x}\n", name, version);
}

pub struct HeaderDumper {}

impl Delegate for HeaderDumper {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start { name, version } => print_start(name, version),
            SectionStart { s_type, s_beg, s_end, s_len } => {
                println!("{:>9} start={:#010x} end={:#010x} (size={:#010x}) count: 1", s_type.as_str(), s_beg, s_end, s_len);
            },
            End => println!(""),
            _ => {},
        }
        Ok(())
    }
}

pub struct DetailsDumper {}

impl Delegate for DetailsDumper {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start { name, version } => print_start(name, version),
            SectionStart { s_type, s_beg: _, s_end: _, s_len: _ } => {
                if s_type != SectionType::Code {
                    println!("{}:", s_type.as_str());            
                }
            },
            TypeStart { n, form: _ } => {
                print!(" - type[{}] (", n);
            },
            TypeParameter { n, t } => {
                if n > 1 { print!(", ") }
                print!("{}", t);                
            },
            TypeParametersEnd => {
                print!(") ->");
            }
            TypeReturn { n: _, t } => {
                print!(" {}", t);
            },
            TypeReturnsEnd => {
                println!("");
            },
            Function { n, index } => {
                println!(" - func[{}] sig={}", n, index.0);
            },
            Table { n, element_type, limits } => {
                println!(" - table[{}] type={:?} initial={}", n, element_type, limits.min);
            },
            Mem { n, limits }=> {
                print!(" - memory[{}] pages: initial={}", n, limits.min);
                if let Some(maximum) = limits.max {
                    print!(" maximum={}", maximum);
                }
                println!("");
            },
            Global { n, t, mutability, init } => {
                println!(" - global[{}] {} mutable={} init 0x{:02x}={} ", n, t, mutability, init.opcode, init.immediate);
            },
            Export { n, id, index } => {
                let kind = match index {
                    ExternalIndex::Func(_) => "func",
                    ExternalIndex::Table(_) => "table",
                    ExternalIndex::Mem(_) => "memory",
                    ExternalIndex::Global(_) => "global",
                };
                println!(" - {}[{}] -> {:?}", kind, n, str::from_utf8(id.0)?)            
            },
            StartFunction { index } => {
                println!(" - start function: {}", index.0);
            },
            DataSegment {n, index: _, offset, data } => {
                println!(" - segment[{}] size={} - init {}={} ", n, data.len(), "i32", offset.immediate);
                print!(" - {:07x}:", offset.immediate);
                for (i, d) in data.iter().enumerate() {
                    if i % 2 == 0 {
                        print!(" ");
                    }
                    print!("{:02x}", d);
                }
                println!("");                
            },
            End => println!(""),
            _ => {},
        }
        Ok(())
    }
}

pub struct Disassembler {
    depth: usize,
}

impl Disassembler {
    pub fn new() -> Self {
        Disassembler { depth: 0 }
    }
}

impl Delegate for Disassembler {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start { name, version } => print_start(name, version),
            CodeStart { c: _ } => {
                println!("Code Disassembly:\n")
            },
            Body { n, offset, size: _, locals: _ } => {
                println!("{:06x} func[{}]:", offset, n);
            },
            Instruction { n: _, offset, data, op, imm } => {
                match op.code {
                    ELSE | END => {
                        if self.depth > 0 {
                            self.depth -= 1;
                        }
                    },
                    _ => {},
                }
                print!(" {:06x}:", offset);
                let mut w = 0;
                for b in data.iter() {
                    print!(" {:02x}", b);
                    w += 3;
                }
                while w < 28 {
                    print!(" ");
                    w += 1;
                }
                print!("| ");
                for _ in 0..self.depth { print!("  ") }
                println!("{}{:?}", op.text, imm);

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