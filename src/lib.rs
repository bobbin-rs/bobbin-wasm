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
pub mod interp;
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


pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;
pub const FIXUP: u32 = 0xffff_ffff;
