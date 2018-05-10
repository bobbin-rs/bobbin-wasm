#![no_std]
#![feature(try_from, ptr_offset_from, align_offset, core_float, core_intrinsics, float_internals)]

extern crate fallible_iterator;
extern crate byteorder;
#[cfg(not(feature="enable-log-off"))]
#[macro_use] extern crate log;
#[cfg(feature="enable-log-off")]
#[macro_use] pub mod no_log;

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
pub const PAGE_SIZE: usize = 65535;
