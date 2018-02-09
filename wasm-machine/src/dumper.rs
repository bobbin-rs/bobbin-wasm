use {SectionType};
use event::Event;
use delegate::*;
use core::str;

pub struct HeaderDumper {}

impl Delegate for HeaderDumper {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start => println!("Sections:\n"),
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

impl DetailsDumper {}

impl Delegate for DetailsDumper {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use ::event::Event::*;
        match evt {
            Start => println!("Section Details:\n"),
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
                print!("{:?}", t);                
            },
            TypeParametersEnd => {
                print!(") => (");
            }
            TypeReturn { n: _, t } => {
                print!(") -> {:?}", t);
            },
            TypeReturnsEnd => {
                println!(")");
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
                println!(" - global[{}] {:?} mutable={} init 0x{:02x}={} ", n, t, mutability, init.opcode, init.immediate);
            },
            Export { n, id, index: _ } => {
                println!(" - {:?}[{}] -> {:?}", "kind", n, str::from_utf8(id.0)?)            
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