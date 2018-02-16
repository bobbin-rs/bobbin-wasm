#![allow(dead_code)]
#![no_std]
#![feature(try_from, offset_to, align_offset)]

// extern crate core;
extern crate byteorder;
#[macro_use] extern crate log;

pub mod inplace;

pub mod error;
pub mod opcode;
pub mod types;
pub mod cursor;
pub mod reader;
pub mod writer;
pub mod stack;
pub mod small_vec;
pub mod compiler;
// pub mod loader;
pub mod typeck;
// pub mod interp;
pub mod memory_inst;
pub mod module;
pub mod module_inst;
// pub mod binary_reader;
pub mod event;
pub mod delegate;
pub mod dumper;
pub mod visitor;
pub mod wasm_read;

pub use error::*;
pub use types::*;
pub use event::*;
pub use cursor::*;
pub use reader::*;
pub use writer::*;
pub use module::*;
// pub use binary_reader::*;
pub use delegate::*;
pub use dumper::*;

use core::str;

pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;
pub const FIXUP: u32 = 0xffff_ffff;

pub type WasmResult<T> = Result<T, Error>;


#[derive(Debug, Clone, Copy)]
pub struct Function {
    offset: u32,
    signature: u32,
}

impl Function {
    pub fn new(offset: u32, signature: u32) -> Self {
        Function { offset, signature }
    }
}

pub trait Handler {
    fn call(&mut self, id: u32, args: &[Value]) -> Option<Value>;
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SectionType {
    Custom = 0x0,
    Type = 0x1,
    Import = 0x2,
    Function = 0x3,
    Table = 0x4,
    Memory = 0x5,
    Global = 0x6,
    Export = 0x7,
    Start = 0x8,
    Element = 0x9,
    Code = 0x0a,
    Data = 0x0b,
}

impl SectionType {
    fn try_from_u32(other: u32) -> WasmResult<Self> {
        use SectionType::*;
        Ok(
            match other {
                0x00 => Custom,
                0x01 => Type,
                0x02 => Import,
                0x03 => Function,
                0x04 => Table,
                0x05 => Memory,
                0x06 => Global,
                0x07 => Export,
                0x08 => Start,
                0x09 => Element,
                0x0a => Code,
                0x0b => Data,
                _ => return Err(Error::InvalidSection { id: other })                
            }
        )
    }
    fn try_from(other: u8) -> WasmResult<Self> {
        SectionType::try_from_u32(other as u32)
    }

    fn as_str(&self) -> &'static str {
        use SectionType::*;
        match *self {
            Custom => "Custom",
            Type => "Type",
            Import => "Import",
            Function => "Function",
            Table => "Table",
            Memory => "Memory",
            Global => "Global",
            Export => "Export",
            Start => "Start",
            Element => "Element",
            Code => "Code",
            Data => "Data",            
        }
    }
}


impl From<u8> for SectionType {
    fn from(other: u8) -> Self {
        SectionType::try_from(other).expect("Invalid Section Type")
    }
}

pub type DelegateResult = Result<(), Error>;

pub trait Delegate {
    fn dispatch(&mut self, evt: event::Event) -> DelegateResult;
}

pub trait WriteTo {
    fn write_to(&self, w: &mut Writer) -> WasmResult<()>;
}

// pub trait NewFrom {
//     fn new_from(m: &mut Module, c: &mut Cursor) -> WasmResult<Self> {

//     }
// }