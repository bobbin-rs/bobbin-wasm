#![allow(dead_code)]
#![no_std]
#![feature(try_from, offset_to, align_offset)]

// extern crate core;
extern crate byteorder;
#[macro_use] extern crate log;

pub mod inplace;

pub mod opcode;
pub mod types;
pub mod cursor;
pub mod reader;
pub mod writer;
pub mod stack;
pub mod small_vec;
pub mod loader;
pub mod typeck;
pub mod interp;
pub mod memory_inst;
pub mod module;
pub mod module_inst;
pub mod binary_reader;
pub mod event;
pub mod delegate;
pub mod dumper;

pub use types::*;
pub use event::*;
pub use cursor::*;
pub use reader::*;
pub use writer::*;
pub use module::*;
pub use binary_reader::*;
pub use delegate::*;
pub use dumper::*;

use core::fmt;
use core::str;

pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;
pub const FIXUP: u32 = 0xffff_ffff;

pub type WasmResult<T> = Result<T, Error>;

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
    Leb128Overflow,
    UndefinedTableIndex { id: i32 },
    SignatureMismatch,
    TypeCheck(&'static str),

    InvalidHeader,
    InvalidSection { id: u32 },
    InvalidGlobalKind { id: u8 },
    UnknownSignatureType,
    UnknownExternalKind,
    InvalidReturnType,
    InvalidIfSignature,
    InvalidReservedValue,
    InvalidBranchTableDefault { id: u32, len: u32},
    InvalidImport,
    InvalidLocal { id: u32 },
    InvalidGlobal { id: u32 },
    InvalidFunction { id: u32 },
    InvalidSignature { id: u32 },
    UnexpectedData { wanted: u32, got: u32 },
    UnexpectedStackDepth { wanted: u32, got: u32},
    UnexpectedTypeStackDepth { wanted: u32, got: u32},
    UnexpectedType { wanted: TypeValue, got: TypeValue },
    UnexpectedReturnValue { wanted: TypeValue, got: TypeValue},
    UnexpectedReturnLength { got: u32 },
    FmtError(fmt::Error),
    Utf8Error(str::Utf8Error),
    OpcodeError(opcode::Error),
    StackError(stack::Error),
}

impl From<fmt::Error> for Error {
    fn from(other: fmt::Error) -> Error {
        Error::FmtError(other)
    }
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


impl From<str::Utf8Error> for Error {
    fn from(other: str::Utf8Error) -> Error {
        Error::Utf8Error(other)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExternalKind {
    Function = 0x00,
    Table = 0x01,
    Memory = 0x02,
    Global = 0x03,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum TypeValue {
    Any = 0x00,
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
        TypeValue::Any
    }
}
impl From<u8> for TypeValue {
    fn from(other: u8) -> Self {
        TypeValue::from(other as i8)
    }
}
impl From<i8> for TypeValue {
    fn from(other: i8) -> Self {
        match other {
             0x00 => TypeValue::Any,
            -0x01 => TypeValue::I32,
            -0x02 => TypeValue::I64,
            -0x03 => TypeValue::F32,
            -0x04 => TypeValue::F64,
            -0x10 => TypeValue::AnyFunc,
            -0x20 => TypeValue::Func,
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

impl fmt::Display for TypeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypeValue::*;
        write!(f, "{}", match *self {
            Any => "any",
            I32 => "i32",
            I64 => "i64",
            F32 => "f32",
            F64 => "f64",
            AnyFunc => "anyfunc",
            Func => "func",
            Void => "void",
        })
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