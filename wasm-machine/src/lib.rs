#![allow(dead_code)]
//#![no_std]
#![feature(try_from)]

extern crate core;

extern crate byteorder;
extern crate wasm_leb128;

pub mod opcode;
pub mod reader;
pub mod writer;
pub mod stack;
pub mod scanner;
pub mod loader;
pub mod machine;
pub mod module;

pub use reader::*;
pub use writer::*;
pub use module::*;

// use byteorder::{ByteOrder, LittleEndian};
// use wasm_leb128::{read_i32, read_u32};

#[derive(Debug, PartialEq)]
pub enum Error {
    Unreachable,
    Return,
    End,
    Unimplemented,
    InvalidBlockType,
    ScopesFull,
    FixupsFull,
    OutOfBounds,

    InvalidHeader,
    InvalidSection { id: u32 },
    UnknownSignatureType,
    UnknownExternalKind,
    UnexpectedData,

    InvalidIfSignature,
    InvalidReservedValue,
    InvalidBranchTableDefault { id: usize, len: usize},
    InvalidLocal { id: usize, len: usize },
    InvalidGlobal { id: usize, len: usize },
    InvalidFunction { id: usize, len: usize },
    InvalidSignature { id: usize, len: usize },
    UnexpectedStackDepth { wanted: usize, got: usize},
    UnexpectedType { wanted: TypeValue, got: TypeValue },
    UnexpectedReturnValue { wanted: TypeValue, got: TypeValue},
    UnexpectedReturnLength { got: usize },
    OpcodeError(opcode::Error),
    StackError(stack::Error),
    Leb128Error(wasm_leb128::Error),

}

impl From<opcode::Error> for Error {
    fn from(other: opcode::Error) -> Error {
        Error::OpcodeError(other)
    }
}

impl From<stack::Error> for Error {
    fn from(other: stack::Error) -> Error {
        Error::StackError(other)
    }
}

impl From<wasm_leb128::Error> for Error {
    fn from(other: wasm_leb128::Error) -> Error {
        Error::Leb128Error(other)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value(i32);

impl From<i32> for Value {
    fn from(other: i32) -> Value {
        Value(other)
    }
}

impl From<u32> for Value {
    fn from(other: u32) -> Value {
        Value(other as i32)
    }
}

impl From<Value> for i32 {
    fn from(other: Value) -> i32 {
        other.0
    }
}

impl From<Value> for u32 {
    fn from(other: Value) -> u32 {
        other.0 as u32
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum TypeValue {
    None = 0x00,
    I32 = -0x01,
    I64 = -0x02,
    F32 = -0x03,
    F64 = -0x04,
    AnyFunc = -0x10,
    Func = -0x20,
    Void = -0x40,
}

impl Default for TypeValue {
    fn default() -> Self {
        TypeValue::None
    }
}

impl From<i8> for TypeValue {
    fn from(other: i8) -> Self {
        match other {
             0x00 => TypeValue::None,
            -0x01 => TypeValue::I32,
            -0x02 => TypeValue::I64,
            -0x03 => TypeValue::F32,
            -0x04 => TypeValue::F64,
            -0x40 => TypeValue::Void,
            _ => panic!("Unrecognized TypeValue: 0x{:02x}", other)
        }
    }
}

impl From<TypeValue> for i8 {
    fn from(other: TypeValue) -> Self {
        other as i8
    }
}

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