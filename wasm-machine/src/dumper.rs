use {SectionType, TypeValue};
use delegate::*;
use core::str;

pub struct HeaderDumper {}

impl Delegate for HeaderDumper {
    fn start(&mut self) -> DelegateResult {
        Ok({
            println!("Sections:\n");
        })
    }    

    fn section_start(&mut self, s_type: SectionType, s_start: u32, s_end: u32, s_len: u32) -> DelegateResult {
        Ok({
            println!("{:>9} start={:#010x} end={:#010x} (size={:#010x}) count: 1", s_type.as_str(), s_start, s_end, s_len)
        })
    }    

    fn end(&mut self, _pos: u32) -> DelegateResult {
        Ok({ 
            println!("");
        })
    }
    
}

pub struct DetailsDumper {}

impl DetailsDumper {}

impl Delegate for DetailsDumper {
    fn start(&mut self) -> DelegateResult {
        Ok({
            println!("Section Details:\n");
        })
    }

    fn section_start(&mut self, s_type: SectionType, _s_start: u32, _s_end: u32, _s_len: u32) -> DelegateResult {
        Ok({
            println!("{}:", s_type.as_str());            
        })
    }    
    
    fn type_start(&mut self, index: u32, _form: i8) -> DelegateResult {
        Ok({
            print!(" - type[{}] ", index);
        })
    }

    fn type_parameters_start(&mut self, _count: u32) -> DelegateResult { 
        Ok({
            print!("(")
        })
    }
    fn type_parameter(&mut self, index: u32, tv: TypeValue) -> DelegateResult { 
        Ok({
            if index > 1 { print!(", ") }
            print!("{:?}", tv);
        })
    }     
    fn type_parameters_end(&mut self) -> DelegateResult { 
        Ok({
            print!(")")
        })
    }      
    
    fn type_return(&mut self, _index: u32, tv: TypeValue) -> DelegateResult { 
        Ok({
            print!(" -> {:?}", tv);
        })
    }   

    fn type_end(&mut self) -> DelegateResult {
        Ok({
            println!("");
        })
    }

    fn function(&mut self, index: u32, sig: u32) -> DelegateResult { 
        Ok({
            println!(" - func[{}] sig={}", index, sig);
        })
    }

    fn memory(&mut self, index: u32, _flags: u32, minimum: u32, maximum: Option<u32>) -> DelegateResult {
        Ok({
            print!(" - memory[{}] pages: initial={}", index, minimum);
            if let Some(maximum) = maximum {
                print!(" maximum={}", maximum);
            }
            println!("");
        })
    }

    fn export(&mut self, index: u32, id: &[u8], _kind: i8, _external_index: u32) -> DelegateResult { 
        Ok({
            println!(" - func[{}] -> {:?}", index, str::from_utf8(id)?)
        })
    }   

    fn end(&mut self, _pos: u32) -> DelegateResult {
        Ok({ 
            println!("");
        })
    }
}