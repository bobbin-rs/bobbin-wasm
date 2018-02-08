#![allow(unused_variables)]
use {Error, SectionType};

pub type DelegateResult = Result<(), Error>;

pub trait Delegate {
    fn start(&mut self) -> DelegateResult { Ok(()) }
    fn end(&mut self, pos: u32) -> DelegateResult { Ok(()) }

    fn section_start(&mut self, s_type: SectionType, s_start: u32, s_end: u32, s_len: u32) -> DelegateResult { Ok(()) }
    fn section_end(&mut self) -> DelegateResult { Ok(()) }
}