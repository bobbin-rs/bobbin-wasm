use parser::error::Error;
use parser::reader::{Reader, Read};
use core::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Reserved();

impl<'a> Read<Reserved> for Reader<'a> {
    fn read(&mut self) -> Result<Reserved, Error> {
        Ok({
            self.read_var_u0()?;
            Reserved()
        })
    }
}

impl fmt::Display for Reserved {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueType {
    Any = 0x00,
    Void = 0x40,    
    Func = 0x60,
    AnyFunc = 0x70,
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
}

impl<'a> Read<ValueType> for Reader<'a> {
    fn read(&mut self) -> Result<ValueType, Error> {
        Ok(match self.read_u8()? {
            0x00 => ValueType::Any,
            0x40 => ValueType::Void,
            0x60 => ValueType::Func,
            0x70 => ValueType::AnyFunc,
            0x7f => ValueType::I32,
            0x7e => ValueType::I64,
            0x7d => ValueType::F32,
            0x7c => ValueType::F64,
            _ => return Err(Error::InvalidValueType)
        })
    }
}

impl From<u8> for ValueType {
    fn from(other: u8) -> Self {
        match other {
            0x00 => ValueType::Any,
            0x40 => ValueType::Void,
            0x60 => ValueType::Func,
            0x70 => ValueType::AnyFunc,
            0x7f => ValueType::I32,
            0x7e  => ValueType::I64,
            0x7d => ValueType::F32,
            0x7c => ValueType::F64,
            _ => panic!("Unrecognized ValueType: 0x{:02x}", other)
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ValueType::*;
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
#[derive(Debug)]
pub struct FunctionType<'a> {
    pub functype: u8,
    pub parameters: &'a [ValueType],
    pub results: &'a [ValueType],
}

impl<'a> Read<FunctionType<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<FunctionType<'a>, Error> {
        Ok({
            let functype = self.read()?;
            if functype != 0x60 {
                return Err(Error::InvalidFunctionType);
            }
            let parameters = self.read()?;
            let results = self.read()?;
            FunctionType { functype, parameters, results }
        })
    }
}

#[derive(Debug)]
pub struct Limits {
    pub flag: bool,
    pub min: u32,
    pub max: Option<u32>,
}

impl<'a> Read<Limits> for Reader<'a> {
    fn read(&mut self) -> Result<Limits, Error> {
        Ok({
            let flag = self.read()?;
            let min = self.read()?;
            let max = if flag {
                self.read().map(Some)?
            } else {
                None
            };
            Limits { flag, min, max }
        })
    }
}

#[derive(Debug)]
pub struct MemoryType {
    pub limits: Limits,
}

impl<'a> Read<MemoryType> for Reader<'a> {
    fn read(&mut self) -> Result<MemoryType, Error> {
        Ok({
            let limits = self.read()?;
            MemoryType { limits }
        })
    }
}

#[derive(Debug)]
pub struct TableType {
    pub elemtype: ValueType,
    pub limits: Limits,
}

impl<'a> Read<TableType> for Reader<'a> {
    fn read(&mut self) -> Result<TableType, Error> {
        Ok({
            let elemtype = self.read()?;
            if elemtype != ValueType::AnyFunc {
                return Err(Error::InvalidTableType)
            }
            let limits = self.read()?;
            TableType { elemtype, limits }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalType {
    pub valtype: ValueType,
    pub mutable: bool,
}

impl<'a> Read<GlobalType> for Reader<'a> {
    fn read(&mut self) -> Result<GlobalType, Error> {
        Ok({
            let valtype = self.read()?;
            let mutable = self.read()?;
            GlobalType { valtype, mutable }
        })
    }
}
