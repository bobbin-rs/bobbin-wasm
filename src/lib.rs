#![no_std]
#![feature(try_from, offset_to, align_offset, core_float)]

extern crate fallible_iterator;
extern crate byteorder;
#[macro_use] extern crate log;

pub mod error;
pub mod types;
pub mod cursor;
pub mod reader;
pub mod writer;
pub mod stack;
pub mod small_vec;
pub mod compiler;
pub mod typeck;
pub mod interp;
pub mod memory_inst;
pub mod module_inst;
pub mod environ;
pub mod floathex;
pub mod parser;

use parser::opcode as opcode;

pub use error::*;
pub use types::*;
pub use cursor::*;
pub use writer::*;


pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;
pub const FIXUP: u32 = 0xffff_ffff;
pub const PAGE_SIZE: usize = 64;
