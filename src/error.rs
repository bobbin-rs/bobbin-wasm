use {ValueType};
use stack;
use parser;
use parser::module::Id;

use core::{fmt, str};

#[derive(Debug, PartialEq)]
pub enum Error {
    Unreachable,
    Return,
    End,
    Unimplemented(&'static str),
    InvalidOpcode(u8),
    UnimplementedOpcode(u8),
    InvalidBlockType,
    ScopesFull,
    FixupsFull,
    OutOfBounds,
    Leb128Overflow,
    UndefinedTableIndex { id: i32 },
    SignatureMismatch,
    TypeCheck(&'static str),

    MissingSection { id: Id },

    NoHostFunction,
    NoHostImportFunction,
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
    UnexpectedType { wanted: ValueType, got: ValueType },
    UnexpectedReturnValue { wanted: ValueType, got: ValueType},
    UnexpectedReturnLength { got: u32 },
    FmtError(fmt::Error),
    Utf8Error(str::Utf8Error),
    // OpcodeError(opcode::Error),
    StackError(stack::Error),
    ParserError(parser::Error)
}

impl From<fmt::Error> for Error {
    fn from(other: fmt::Error) -> Error {
        Error::FmtError(other)
    }
}

// impl From<opcode::Error> for Error {
//     fn from(other: opcode::Error) -> Error {
//         Error::OpcodeError(other)
//     }
// }

impl From<stack::Error> for Error {
    fn from(other: stack::Error) -> Error {
        Error::StackError(other)
    }
}

impl From<parser::Error> for Error {
    fn from(other: parser::Error) -> Error {
        Error::ParserError(other)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(other: str::Utf8Error) -> Error {
        Error::Utf8Error(other)
    }
}