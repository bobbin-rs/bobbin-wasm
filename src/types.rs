use Error;
use core::fmt;
use core::str;

pub use parser::module::{Global, Import, Initializer, ImportDesc, ExportDesc};
pub use parser::types::{Index, Limits, MemoryType, TableType, ValueType, GlobalType};

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
    pub fn try_from_u32(other: u32) -> Result<Self, Error> {
        use types::SectionType::*;
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
    pub fn try_from(other: u8) -> Result<Self, Error> {
        SectionType::try_from_u32(other as u32)
    }

    pub fn as_str(&self) -> &'static str {
        use types::SectionType::*;
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


#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Value(pub i32);

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "i32:{}", self.0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}



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


// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct GlobalType {
//     pub type_value: ValueType,
//     pub mutability: u8,
// }

// #[derive(Debug)]
// pub enum ExportDesc {
//     Function(u32),
//     Table(u32),
//     Memory(u32),
//     Global(u32),
// }

// pub enum ImportDesc {
//     Type(u32),
//     Table(TableType),
//     Memory(MemoryType),
//     Global(GlobalType),
// }

// impl fmt::Debug for ImportDesc {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             ImportDesc::Type(n) => n.fmt(f),
//             ImportDesc::Table(ref t) => t.fmt(f),
//             ImportDesc::Memory(ref m) => m.fmt(f),
//             ImportDesc::Global(ref g) => g.fmt(f),
//         }
//     }
// }


// pub struct Type<'a> {
//     pub parameters: &'a [ValueType],
//     pub returns: &'a [ValueType],
// }


// impl<'a> fmt::Debug for Type<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             write!(f, "Type {{ {:?} -> {:?} }}", self.parameters, self.returns)?;
//             // let indent = "    ";
//             // writeln!(f, "{}<Type>", indent)?;
//             // for p in self.parameters {
//             //     let indent = "      ";
//             //     writeln!(f, "{}<parameter>{:?}</parameter>", indent, ValueType::from(*p as i8))?;
//             // }
//             // for r in self.returns {
//             //     let indent = "      ";
//             //     writeln!(f, "{}<return>{:?}</return>", indent, ValueType::from(*r as i8))?;
//             // }
//             // writeln!(f, "{}</Type>", indent)?;
//         })
//     }
// }

// impl<'a> Type<'a> {
//     pub fn new(parameters: &'a [ValueType], returns: &'a [ValueType]) -> Self {
//         Type { parameters, returns }
//     }
    
//     // pub fn parameters(&self) -> ValueTypesIter<'a> {
//     //     ValueTypesIter { index: 0, buf: self.parameters }
//     // }

//     // pub fn returns(&self) -> ValueTypesIter<'a> {
//     //     ValueTypesIter { index: 0, buf: self.returns }
//     // }

//     // pub fn return_type(&self) -> Option<ValueType> {
//     //     self.returns.first().map(|t| ValueType::from(*t))
//     // }
// }

// #[derive(Debug)]
// pub struct Import<'a> {
//     pub module: &'a str,
//     pub export: &'a str,
//     pub desc: ImportDesc,    
// }

// pub struct Function {
//     pub signature_type_index: u32,
// }

// impl fmt::Debug for Function {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             let indent = "    ";
//             writeln!(f, "{}<Function signature_type={}>", indent, self.signature_type_index)?;
//         })
//     }
// }

// pub struct Table {
//     pub element_type: ValueType,
//     pub limits: Limits,
// }

// impl fmt::Debug for Table {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             let indent = "    ";
//             writeln!(f, "{}<Table type={:?} min={} max={:?}>", indent,
//                 self.element_type, self.limits.min, self.limits.max
//             )?;
//         })
//     }
// }

// // pub struct Memory {
// //     pub limits: Limits,
// // }

// impl fmt::Debug for Memory {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             let indent = "    ";
//             writeln!(f, "{}<Memory min={} max={:?}>", indent, 
//                 self.limits.min, self.limits.max)?;
//         })
//     }
// }

// pub struct Global {
//     pub global_type: GlobalType,
//     pub init: Initializer,
// }

// impl fmt::Debug for Global {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             let indent = "    ";
//             writeln!(f, "{}<Global type={:?} opcode=0x{:02x} immediate=0x{:08x}>", 
//                 indent, self.global_type, self.init.opcode, self.init.immediate)?;
//         })
//     }
// }

#[derive(Debug)]
pub struct Export<'a> {
    pub identifier: &'a str,
    pub export_desc: ExportDesc,
}

pub struct Start {
    pub function_index: u32,
}

impl fmt::Debug for Start {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Start index={:?}>", indent, self.function_index)?;
        })
    }
}

pub struct Element<'a> {
    pub table_index: u32,
    pub offset: Initializer<'a>,
    pub data: &'a [u8],
}

impl<'a> fmt::Debug for Element<'a> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            // let indent = "    ";
            // writeln!(f, "{}<Element index={} opcode={:02x} immediate={:02x}>", indent,
            //     self.table_index, self.offset.instr.opcode, self.offset.instr.immediate,
            // )?;
            // write!(f, "{}  ", indent)?;
            // for d in self.data {
            //     write!(f,"{:02x} ", *d)?;
            // }
            // writeln!(f, "")?;
            // writeln!(f, "{}</Element>", indent)?;
        })
    }
}


pub struct Data<'a> {
    pub memory_index: u32,
    pub offset: Initializer<'a>,
    pub data: &'a [u8],
}


impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            // let indent = "    ";
            // writeln!(f, "{}<Data index={} opcode={:02x} immediate={:02x}>", indent,
            //     self.memory_index, self.offset.instr.opcode, self.offset.instr.immediate,
            // )?;
            // write!(f, "{}  ", indent)?;
            // for d in self.data {
            //     write!(f,"{:02x} ", *d)?;
            // }
            // writeln!(f, "")?;
            // writeln!(f, "{}</Data>", indent)?;
        })
    }
}



// pub struct &'a str(pub &'a [u8]);

// impl<'a> &'a str {
//     pub fn as_str(&self) -> &str {
//         unsafe { &str::from_utf8_unchecked(self.0) }
//     }
// }

// impl<'a> fmt::Debug for &'a str {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self.as_str() )
//     }
// }

// impl<'a> fmt::Display for &'a str {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.as_str() )
//     }
// }



#[derive(Debug)]
pub struct TypeIndex(pub u32);
#[derive(Debug)]
pub struct FuncIndex(pub u32);
#[derive(Debug)]
pub struct TableIndex(pub u32);
#[derive(Debug)]
pub struct MemIndex(pub u32);
#[derive(Debug)]
pub struct GlobalIndex(pub u32);
#[derive(Debug)]
pub struct LocalIndex(pub u32);
#[derive(Debug)]
pub struct LabelIndex(pub u32);

#[derive(Debug)]
pub enum ExternalIndex {
    Func(FuncIndex),
    Table(TableIndex),
    Mem(MemIndex),
    Global(GlobalIndex),
}

impl ExternalIndex {
    pub fn kind(&self) -> u8 {
        use ExternalIndex::*;
        match *self {
            Func(_) => 0x00,
            Table(_) => 0x01,
            Mem(_) => 0x02,
            Global(_) => 0x03,
        }
    }
    pub fn index(&self) -> u32 {
        use ExternalIndex::*;
        match *self {
            Func(FuncIndex(n)) => n,
            Table(TableIndex(n)) => n,
            Mem(MemIndex(n)) => n,
            Global(GlobalIndex(n)) => n,
        }        
    }
}

// #[derive(Debug)]
// pub struct Limits {
//     pub flags: u32,
//     pub min: u32,
//     pub max: Option<u32>,
// }

// #[derive(Debug)]
// pub struct Initializer {
//     pub opcode: u8,
//     pub immediate: i32,
//     pub end: u8,
// }

// impl Initializer {
//     pub fn value(&self) -> Result<Value, Error> {
//         match self.opcode {
//             I32_CONST => Ok(Value(self.immediate)),
//             _ => unimplemented!(),
//         }
//     }
// }