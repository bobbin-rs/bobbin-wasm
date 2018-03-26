use parser::error::Error;
use parser::reader::{Reader, Read, ReadIterator, SectionReadIterator, FallibleIterator};
use parser::types::*;
use parser::opcode::*;

use core::str;
use core::fmt;

pub type Index = u32;
pub type Depth = u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Id {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
}

impl<'a> Read<Id> for Reader<'a> {
    fn read(&mut self) -> Result<Id, Error> {
        Ok(match self.read_u8()? {
            0 => Id::Custom,
            1 => Id::Type,
            2 => Id::Import,
            3 => Id::Function,
            4 => Id::Table,
            5 => Id::Memory,
            6 => Id::Global,
            7 => Id::Export,
            8 => Id::Start,
            9 => Id::Element,
            10 => Id::Code,
            11 => Id::Data,
            _ => return Err(Error::InvalidSectionId)
        })
    }
}

impl Id {
    pub fn as_str(&self) -> &'static str {
        use self::Id::*;
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

#[derive(Debug)]
pub struct Module<'a> {
    pub magic: u32,
    pub version: u32,    
    pub buf: &'a [u8]
}

impl<'a> Read<Module<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Module<'a>, Error> {
        Ok({
            let magic = self.read_u32()?;
            let version = self.read_u32()?;
            let buf = self.rest();
            Module { magic, version, buf }
        })
    }
}

impl<'a> Module<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, Error> {
        Reader::new(buf).read()
    }

    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn sections(&self) -> ReadIterator<Section<'a>> {
        ReadIterator::new(Reader::new(self.buf))
    }

    pub fn offset_to(&self, buf: &[u8]) -> usize {
        Reader::new(buf).offset_from(&Reader::new(self.buf)) + 8
    }
}

#[derive(Debug)]
pub struct Section<'a> {
    pub id: Id,
    pub buf: &'a [u8],
}

impl<'a> Section<'a> {
    pub fn id(&self) -> Id {
        self.id
    }
    pub fn count(&self) -> Result<usize, Error> {
        Reader::new(self.buf).read_var_u7().map(|v| v.into())
    }

    fn empty_iter<T>(&self) -> SectionReadIterator<'a, T> {
        SectionReadIterator::new(Reader::new(&[]))
    }

    fn iter_for_section_id<T>(&self, id: Id) -> SectionReadIterator<'a, T> {
        if self.id == id {
            SectionReadIterator::new(Reader::new(self.buf))
        } else {
            self.empty_iter()
        }        
    }
    
    pub fn function_types(&self) -> SectionReadIterator<'a, FunctionType<'a>> {
        self.iter_for_section_id(Id::Type)
    }

    pub fn types(&self) -> SectionReadIterator<'a, FunctionType<'a>> {
        self.iter_for_section_id(Id::Type)
    }

    pub fn functions(&self) -> SectionReadIterator<'a, Index> {
        self.iter_for_section_id(Id::Function)
    }

    pub fn tables(&self) -> SectionReadIterator<'a, TableType> {
        self.iter_for_section_id(Id::Table)
    }

    pub fn globals(&self) -> SectionReadIterator<'a, Global> {
        self.iter_for_section_id(Id::Global)
    }

    pub fn memory(&self) -> SectionReadIterator<'a, MemoryType> {
        self.iter_for_section_id(Id::Memory)
    }

    pub fn elements(&self) -> SectionReadIterator<'a, Element> {
        self.iter_for_section_id(Id::Element)
    }

    pub fn exports(&self) -> SectionReadIterator<'a, Export<'a>> {
        self.iter_for_section_id(Id::Export)
    }

    pub fn code(&self) -> SectionReadIterator<'a, Code<'a>> {
        self.iter_for_section_id(Id::Code)
    }

    pub fn data(&self) -> SectionReadIterator<'a, Data<'a>> {
        self.iter_for_section_id(Id::Data)
    }
    pub fn custom(&self) -> Result<Custom<'a>, Error> {
        Reader::new(self.buf).read()
    }
}

impl<'a> Read<Section<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Section<'a>, Error> {
        let id = self.read()?;
        let buf = self.read()?;
        Ok(Section { id, buf })
    }
}

#[derive(Debug)]
pub struct Custom<'a> {
    pub name: &'a str,
    pub buf: &'a [u8],
}

impl<'a> Read<Custom<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Custom<'a>, Error> {
        let name = self.read()?;
        let buf = self.rest();
        Ok(Custom { name, buf })
    }
}

#[derive(Debug)]
pub enum ImportDesc {
    Func(Index),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}
impl<'a> Read<ImportDesc> for Reader<'a> {
    fn read(&mut self) -> Result<ImportDesc, Error> {
        match self.read_u8()? {
            0x00 => self.read().map(ImportDesc::Func),
            0x01 => self.read().map(ImportDesc::Table),
            0x02 => self.read().map(ImportDesc::Memory),
            0x03 => self.read().map(ImportDesc::Global),
            _ => Err(Error::InvalidImportDesc),
        }
    }
}

#[derive(Debug)]
pub struct Import<'a> {
    pub module: &'a str,
    pub name: &'a str,
    pub import_desc: ImportDesc,
}

impl<'a> Read<Import<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Import<'a>, Error> {
        Ok({
            let module = self.read()?;
            let name = self.read()?;
            let import_desc = self.read()?;
            Import { module, name, import_desc }
        })
    }
}

#[derive(Debug)]
pub struct Global<'a> {
    pub global_type: GlobalType,
    pub init: Initializer<'a>,
}

impl<'a> Read<Global<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Global<'a>, Error> {
        Ok({
            let global_type = self.read()?;
            let init = self.read()?;
            Global { global_type, init }
        })
    }
}

#[derive(Debug)]
pub enum ExportDesc {
    Func(Index),
    Table(Index),
    Memory(Index),
    Global(Index),
}
impl<'a> Read<ExportDesc> for Reader<'a> {
    fn read(&mut self) -> Result<ExportDesc, Error> {
        match self.read_u8()? {
            0x00 => self.read().map(ExportDesc::Func),
            0x01 => self.read().map(ExportDesc::Table),
            0x02 => self.read().map(ExportDesc::Memory),
            0x03 => self.read().map(ExportDesc::Global),
            _ => Err(Error::InvalidExportDesc),
        }
    }
}

#[derive(Debug)]
pub struct Export<'a> {
    pub name: &'a str,
    pub export_desc: ExportDesc,
}

impl<'a> Read<Export<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Export<'a>, Error> {
        Ok({
            let name = self.read()?;
            let export_desc = self.read()?;
            Export {name, export_desc }
        })
    }
}

#[derive(Debug)]
pub struct Start {
    pub func_index: Index,
}

impl<'a> Read<Start> for Reader<'a> {
    fn read(&mut self) -> Result<Start, Error> {
        Ok({
            let func_index = self.read()?;
            Start { func_index }
        })
    }
}

#[derive(Debug)]
pub struct Element<'a> {
    pub table_index: Index,
    pub offset: Initializer<'a>,
    pub init: &'a [u8],
}

impl<'a> Read<Element<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Element<'a>, Error> {
        Ok({            
            let table_index = self.read()?;
            let offset = self.read()?;            
            let base = self.clone();
            let count: u32 = self.read()?;
            for _ in 0..count {
                let _: Index = self.read()?;
            }
            let len = self.offset_from(&base);
            let init = &base.into_slice()[..len];            
            Element { table_index, offset, init }
        })
    }
}

impl<'a> Element<'a> {
    pub fn iter(&self) -> SectionReadIterator<'a, Index> {
        SectionReadIterator::new(Reader::new(self.init))
    }
}

#[derive(Debug)]
pub struct Code<'a> {
    pub buf: &'a [u8],
    pub func: Func<'a>,
}

impl<'a> Read<Code<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Code<'a>, Error> {
        Ok({
            let buf = self.rest();
            let func = self.read()?;
            Code { buf, func }
        })
    }
}

#[derive(Debug)]
pub struct Func<'a> {
    pub buf: &'a [u8],
}

impl<'a> Read<Func<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Func<'a>, Error> {
        Ok({
            let buf = self.read()?;
            Func { buf }
        })
    }
}

impl<'a> Func<'a> {
    pub fn iter(&self) -> FuncItemIterator<'a> {
        FuncItemIterator::new(Reader::new(self.buf))
    }
}

#[derive(Debug)]
pub struct Local {
    pub n: u32,
    pub t: ValueType,
}

impl<'a> Read<Local> for Reader<'a> {
    fn read(&mut self) -> Result<Local, Error> {
        Ok({
            let n = self.read()?;
            let t = self.read()?;
            Local { n: n, t: t }
        })
    }
}

#[derive(Debug)]
pub enum FuncItem<'a> {
    Local(Local),
    Instr(Instr<'a>)
}

enum FuncItemIteratorState {
    Start,
    Local(u32),
    Instr,
}

pub struct FuncItemIterator<'a> {
    r: Reader<'a>,
    state: FuncItemIteratorState,
}

impl<'a> FuncItemIterator<'a> {
    pub fn new(r: Reader<'a>) -> Self {
        use self::FuncItemIteratorState as State;
        FuncItemIterator { r, state: State::Start }
    }    
}

impl<'a> FallibleIterator for FuncItemIterator<'a> {
    type Item = FuncItem<'a>;
    type Error = Error;
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        use self::FuncItemIteratorState as State;
        loop {
            match self.state {
                State::Start => {
                    let count = self.r.read()?;
                    if count > 0 {
                        self.state = State::Local(count);
                    } else {
                        self.state = State::Instr;
                    }
                },
                State::Local(count) => {
                    let local = self.r.read()?;
                    if count > 1 {
                        self.state = State::Local(count - 1);
                    } else {
                        self.state = State::Instr;
                    }
                    return Ok(Some(FuncItem::Local(local)));
                },
                State::Instr => {
                    if self.r.len() > 0 {
                        let instr = self.r.read()?;
                        return Ok(Some(FuncItem::Instr(instr)));
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Data<'a> {
    pub mem_index: Index,
    pub offset: Initializer<'a>,
    pub init: &'a [u8],
}

impl<'a> Read<Data<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Data<'a>, Error> {
        Ok({
            let mem_index = self.read()?;
            let offset = self.read()?;
            let init = self.read()?;
            Data { mem_index, offset, init }
        })
    }
}

#[derive(Debug)]
pub struct Initializer<'a> {
    pub instr: Instr<'a>,
    pub end: u8,
}
impl<'a> Read<Initializer<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Initializer<'a>, Error> {
        Ok({
            let instr = self.read()?;
            let end = self.read_u8()?;
            if end != END {
                return Err(Error::InvalidEnd)
            }
            Initializer { instr, end }
        })
    }
}

pub struct Instr<'a> {
    pub opcode: u8,
    pub immediate: Immediate<'a>,
    pub data: &'a [u8],
}

impl<'a> Read<Instr<'a>> for Reader<'a> {
    fn read(&mut self) -> Result<Instr<'a>, Error> {
        let base = self.clone();
        let opcode = self.read()?;
        let immediate = match opcode {
            BLOCK | LOOP | IF => {
                Immediate::Block { signature: self.read()? }
            },
            BR | BR_IF => {
                Immediate::Branch { depth: self.read()? }
            },
            BR_TABLE => {
                let mut base = self.clone();
                let count: u32 = self.read()?;
                for _ in 0..count {                    
                    let _: u32 = self.read()?;
                }
                let _: u32 = self.read()?;
                let len = self.offset_from(&base);
                let table = base.read_slice(len)?;
                Immediate::BranchTable { table }
            },
            GET_LOCAL | SET_LOCAL | TEE_LOCAL => {
                Immediate::Local { index: self.read()? }         
            }
            GET_GLOBAL | SET_GLOBAL => {
                Immediate::Global { index: self.read()? }
            },
            CALL => {
                Immediate::Call { index: self.read()? }
            },
            CALL_INDIRECT => {
                Immediate::CallIndirect { index: self.read()?, reserved: self.read()? }
            },
            I32_CONST => {
                Immediate::I32Const { value: self.read()? }
            },
            F32_CONST => {
                Immediate::F32Const { value: self.read()? }
            },
            I64_CONST => {
                Immediate::I64Const { value: self.read()? }
            },
            F64_CONST => {
                Immediate::F64Const { value: self.read()? }
            },
            I32_LOAD | I32_STORE |
            I32_LOAD8_S ... I32_LOAD16_U |
            I32_STORE8 ... I32_STORE16 => {
                Immediate::LoadStore { align: self.read()?, offset: self.read()? }
            },

            F32_LOAD | F32_STORE => {
                Immediate::LoadStore { align: self.read()?, offset: self.read()? }
            },

            I64_LOAD | I64_STORE |
            I64_LOAD8_S ... I64_LOAD32_U |
            I64_STORE8 ... I64_STORE32 => {
                Immediate::LoadStore { align: self.read()?, offset: self.read()? }
            },

            F64_LOAD | F64_STORE => {
                Immediate::LoadStore { align: self.read()?, offset: self.read()? }
            },
            
            MEM_SIZE | MEM_GROW => {
                Immediate::Memory { reserved: self.read()? }
            },
            _ => Immediate::None,            
        };
        let len = self.offset_from(&base);
        let data = &base.into_slice()[..len];
        Ok(Instr { opcode, immediate, data })
    }
}

impl<'a> fmt::Debug for Instr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            if let Some(op) = Op::from_opcode(self.opcode) {
                write!(f, "{}{:?}", op.text, self.immediate)?;
            } else {
                write!(f, "unknown")?;
            }
        })
    }
}
pub enum Immediate<'a> {
    None,
    Block { signature: ValueType },
    Branch { depth: Depth },
    BranchTable { table: &'a [u8] },
    Local { index: Index },
    Global { index: Index },
    Call { index: Index },
    CallIndirect { index: Index, reserved: Reserved },
    I32Const { value: i32 },
    F32Const { value: f32 },
    I64Const { value: i64 },
    F64Const { value: f64 },
    LoadStore { align: u32, offset: u32 },
    Memory { reserved: Reserved },
}


impl<'a> fmt::Debug for Immediate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Immediate::*;
        match *self {
            None => Ok(()),
            Block { signature } => if signature != ValueType::Void {
                write!(f, " {:?}", signature)
            } else {
                Ok(())
            },
            Branch { depth } => write!(f, " {}", depth),
            BranchTable { table: _ } => Ok(()),
            Local { ref index } => write!(f, " {}", index),
            Global { ref index } => write!(f, " {}", index),
            Call { ref index } => write!(f, " {}", index),
            CallIndirect { ref index, reserved } => write!(f, " {} {}", index, reserved),
            I32Const { value } => write!(f, " {}", value as u32),
            F32Const { value } => if !value.is_nan() {            
                write!(f, " {:?}", value)
            } else {
                write!(f, " nan")
            }
            I64Const { value } => write!(f, " {}", value),
            F64Const { value } => if !value.is_nan() {
                write!(f, " {:?}", value)
            } else {
                write!(f, " nan")
            }
            LoadStore { align, offset } => write!(f, " {} {}", align, offset),
            Memory { reserved: _ } => Ok(()),
        }

    }
}
