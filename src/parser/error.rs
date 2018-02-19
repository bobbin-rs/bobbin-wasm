#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    UnexpectedEof,
    InvalidU0,
    InvalidU1,
    InvalidU7,
    InvalidU32,
    InvalidI32,
    InvalidU64,
    InvalidI64,
    InvalidUtf8,
    InvalidOpcode,
    InvalidEnd,
    InvalidValueType,
    InvalidBlockType,
    InvalidFunctionType,
    InvalidTableType,
    InvalidSectionId,
    InvalidImportDesc,
    InvalidExportDesc,

    InvalidMagic,
    InvalidVersion,
}