use Error;
use opcode::*;
use core::fmt;
use core::str;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeValue {
    Any = 0x00,
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,
    AnyFunc = 0x70,
    Func = 0x60,
    Void = 0x40,
}

impl Default for TypeValue {
    fn default() -> Self {
        TypeValue::Any
    }
}

impl From<u8> for TypeValue {
    fn from(other: u8) -> Self {
        match other {
            0x00 => TypeValue::Any,
            0x7f => TypeValue::I32,
            0x7e  => TypeValue::I64,
            0x7d => TypeValue::F32,
            0x7c => TypeValue::F64,
            0x70 => TypeValue::AnyFunc,
            0x60 => TypeValue::Func,
            0x40 => TypeValue::Void,
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


#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value(pub i32);

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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalType {
    pub type_value: TypeValue,
    pub mutability: u8,
}

#[derive(Debug)]
pub enum ExportDesc {
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

pub enum ImportDesc {
    Type(u32),
    Table(Table),
    Memory(Memory),
    Global(GlobalType),
}

impl fmt::Debug for ImportDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImportDesc::Type(n) => n.fmt(f),
            ImportDesc::Table(ref t) => t.fmt(f),
            ImportDesc::Memory(ref m) => m.fmt(f),
            ImportDesc::Global(ref g) => g.fmt(f),
        }
    }
}


pub struct Type<'a> {
    pub parameters: &'a [u8],
    pub returns: &'a [u8],
}


impl<'a> fmt::Debug for Type<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            write!(f, "Type {{ {:?} -> {:?} }}", self.parameters, self.returns)?;
            // let indent = "    ";
            // writeln!(f, "{}<Type>", indent)?;
            // for p in self.parameters {
            //     let indent = "      ";
            //     writeln!(f, "{}<parameter>{:?}</parameter>", indent, TypeValue::from(*p as i8))?;
            // }
            // for r in self.returns {
            //     let indent = "      ";
            //     writeln!(f, "{}<return>{:?}</return>", indent, TypeValue::from(*r as i8))?;
            // }
            // writeln!(f, "{}</Type>", indent)?;
        })
    }
}

impl<'a> Type<'a> {
    pub fn new(parameters: &'a [u8], returns: &'a [u8]) -> Self {
        Type { parameters, returns }
    }
    
    // pub fn parameters(&self) -> TypeValuesIter<'a> {
    //     TypeValuesIter { index: 0, buf: self.parameters }
    // }

    // pub fn returns(&self) -> TypeValuesIter<'a> {
    //     TypeValuesIter { index: 0, buf: self.returns }
    // }

    // pub fn return_type(&self) -> Option<TypeValue> {
    //     self.returns.first().map(|t| TypeValue::from(*t))
    // }
}


pub struct Import<'a> {
    pub module: Identifier<'a>,
    pub export: Identifier<'a>,
    pub desc: ImportDesc,    
}

impl<'a> fmt::Debug for Import<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Import module={:?} export={:?}>", indent, 
                str::from_utf8(self.module.0).unwrap(),
                str::from_utf8(self.export.0).unwrap(),
            )?;
            write!(f, "  {:?}", self.desc)?;
            writeln!(f, "{}</Import>", indent)?;
        })
    }
}

pub struct Function {
    pub signature_type_index: u32,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Function signature_type={}>", indent, self.signature_type_index)?;
        })
    }
}

pub struct Table {
    pub element_type: TypeValue,
    pub limits: Limits,
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Table type={:?} min={} max={:?}>", indent,
                self.element_type, self.limits.min, self.limits.max
            )?;
        })
    }
}

pub struct Memory {
    pub limits: Limits,
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Memory min={} max={:?}>", indent, 
                self.limits.min, self.limits.max)?;
        })
    }
}

pub struct Global {
    pub global_type: GlobalType,
    pub init: Initializer,
}

impl fmt::Debug for Global {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Global type={:?} opcode=0x{:02x} immediate=0x{:08x}>", 
                indent, self.global_type, self.init.opcode, self.init.immediate)?;
        })
    }
}

#[derive(Debug)]
pub struct Export<'a> {
    pub identifier: Identifier<'a>,
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
    pub offset: Initializer,
    pub data: &'a [u8],
}

impl<'a> fmt::Debug for Element<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Element index={} opcode={:02x} immediate={:02x}>", indent,
                self.table_index, self.offset.opcode, self.offset.immediate,
            )?;
            write!(f, "{}  ", indent)?;
            for d in self.data {
                write!(f,"{:02x} ", *d)?;
            }
            writeln!(f, "")?;
            writeln!(f, "{}</Element>", indent)?;
        })
    }
}


pub struct Data<'a> {
    pub memory_index: u32,
    pub offset: Initializer,
    pub data: &'a [u8],
}


impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            let indent = "    ";
            writeln!(f, "{}<Data index={} opcode={:02x} immediate={:02x}>", indent,
                self.memory_index, self.offset.opcode, self.offset.immediate,
            )?;
            write!(f, "{}  ", indent)?;
            for d in self.data {
                write!(f,"{:02x} ", *d)?;
            }
            writeln!(f, "")?;
            writeln!(f, "{}</Data>", indent)?;
        })
    }
}



pub struct Identifier<'a>(pub &'a [u8]);

impl<'a> fmt::Debug for Identifier<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", unsafe { str::from_utf8_unchecked(self.0) } )
    }
}


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

#[derive(Debug)]
pub struct Limits {
    pub flags: u32,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug)]
pub struct Initializer {
    pub opcode: u8,
    pub immediate: i32,
    pub end: u8,
}

impl Initializer {
    pub fn value(&self) -> Result<Value, Error> {
        match self.opcode {
            I32_CONST => Ok(Value(self.immediate)),
            _ => unimplemented!(),
        }
    }
}

// pub const MAX_PARAMETERS: usize = 16;
// pub const MAX_RETURNS: usize = 1;

// #[derive(Debug, PartialEq, Eq)]
// pub struct Sig<'a> {
//     buf: &'a [u8],    
// }

// impl<'a> Sig<'a> {
//     fn is_valid_type(v: u8) -> bool {
//         match v as i8 {
//             -0x01 | -0x02 |-0x03 | -0x04 => true,
//             _ => false,
//         }
//     }

//     pub fn new(buf: &mut Cursor<'a>) -> Sig<'a> {
//         let mut orig = buf.clone();
//         let p_num = buf.read_var_i32();
        
//         for _ in 0..p_num {
//             buf.read_var_i7();
//         }
//         let r_num = buf.read_var_i32();
        
//         for _ in 0..r_num {
//             buf.read_var_i7();
//         }
//         let len = orig.len() - buf.len();
        
//         let buf = orig.slice(len);        
//         Sig { buf }
//     }

//     pub fn read_slice(buf: &[u8]) -> Option<(Sig, &[u8])> {
//         debug_assert!(mem::size_of::<TypeValue>() == mem::size_of::<u8>());
        
//         // Check for appropriate length

//         if buf.len() < 1 { return None }

//         let p_num = buf[0] as usize;

//         // Check parameter count
//         if p_num > MAX_PARAMETERS { return None }

//         let p_len = 1 + p_num;

//         if buf.len() < p_len + 1 { return None }

//         let r_num = buf[p_len] as usize;

//         // Check return count
//         if r_num > MAX_RETURNS { return None }
        
//         let s_len = p_len + 1 + r_num;

//         if buf.len() < s_len { return None }

//         // Check that type is one of I32, I64, F32, F64

//         for i in 1..p_len {
//             if !Self::is_valid_type(buf[i]) { return None }
//         }

//         for i in p_len+1..s_len {
//             if !Self::is_valid_type(buf[i]) { return None }
//         }

//         Some((Sig { buf: &buf[..s_len] }, &buf[s_len..]))
//     }    

//     pub fn buf(&self) -> &'a [u8] {
//         self.buf
//     }

//     pub fn parameters(&self) -> &'a [TypeValue] {
//         let len = self.buf[0] as usize;        
//         let buf = &self.buf[1..1+len];        
//         unsafe { slice::from_raw_parts(buf.as_ptr() as *const TypeValue, buf.len())}
//     }

//     pub fn returns(&self) -> &'a [TypeValue] {
//         let p_num = self.buf[0] as usize;        
//         let p_len = 1 + p_num;        
//         let r_num = self.buf[p_len] as usize;        
//         let buf = &self.buf[p_len+1..p_len+1+r_num];
//         unsafe { slice::from_raw_parts(buf.as_ptr() as *const TypeValue, buf.len())}
//     }
// }

// pub struct SigIter<'a> {
//     buf: &'a [u8],
//     len: u8,
//     pos: u8,
// }

// impl<'a> Iterator for SigIter<'a> {
//     type Item = TypeValue;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.pos < self.len {
//             let t = TypeValue::from(self.buf[self.pos as usize]);
//             self.pos += 1;
//             Some(t)
//         } else {
//             None
//         }
//     }
// }


// #[cfg(test)]
// mod tests {
//     use super::*;

//     // #[test]
//     // fn test_read_sig() {
//     //     let (sig, buf) = Sig::read_slice(&[0, 0]).unwrap();
//     //     assert_eq!(sig.parameters(), &[]);
//     //     assert_eq!(sig.returns(), &[]);
//     //     assert_eq!(buf, &[]);

//     //     let (sig, buf) = Sig::read_slice(&[1, I32 as u8, 0]).unwrap();
//     //     assert_eq!(sig.parameters(), &[TypeValue::I32]);
//     //     assert_eq!(sig.returns(), &[]);
//     //     assert_eq!(buf, &[]);        

//     //     let (sig, buf) = Sig::read_slice(&[0, 1, I32 as u8]).unwrap();
//     //     assert_eq!(sig.parameters(), &[]);
//     //     assert_eq!(sig.returns(), &[TypeValue::I32]);
//     //     assert_eq!(buf, &[]);        


//     //     let (sig, buf) = Sig::read_slice(&[2, I32 as u8, I32 as u8, 1, I64 as u8]).unwrap();
//     //     assert_eq!(sig.parameters(), &[TypeValue::I32, TypeValue::I32]);
//     //     assert_eq!(sig.returns(), &[TypeValue::I64]);
//     //     assert_eq!(buf, &[]);
//     // }

//     // #[test]
//     // fn test_read_invalid_sig() {
//     //     // empty
//     //     assert!(Sig::read_slice(&[]).is_none());
//     //     // too short
//     //     assert!(Sig::read_slice(&[0]).is_none());
//     //     // parameters too short
//     //     assert!(Sig::read_slice(&[2, I32 as u8, 1, I64 as u8]).is_none());
//     //     // parameters too long
//     //     assert!(Sig::read_slice(&[2, I32 as u8, I32 as u8, I32 as u8, 1, I64 as u8]).is_none());
//     //     // returns too short
//     //     assert!(Sig::read_slice(&[2, I32 as u8, I32 as u8, 1]).is_none());
//     // }    
// }