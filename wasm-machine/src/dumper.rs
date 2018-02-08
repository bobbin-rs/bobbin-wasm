use {SectionType};
use delegate::*;

pub struct Dumper {

}

impl Delegate for Dumper {
    fn section_start(&mut self, s_type: SectionType, s_start: u32, s_end: u32, s_len: u32) -> DelegateResult {
        Ok({
            println!("{:>9} start={:#010x} end={:#010x} (size={:#010x}) count: 1", s_type.as_str(), s_start, s_end, s_len)
        })
    }    
    
}