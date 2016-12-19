use buf::Buf;
use Error;

pub enum Section<'a> {
    Name(NameSection<'a>),
    Type(TypeSection<'a>),
    Import(ImportSection<'a>),
    Function(FunctionSection<'a>),
    Table(TableSection<'a>),
    Memory(MemorySection<'a>),
    Global(GlobalSection<'a>),
    Export(ExportSection<'a>),
    Start(StartSection<'a>),
    Element(ElementSection<'a>),
    Code(CodeSection<'a>),
    Data(DataSection<'a>),
}

pub struct NameSection<'a>(pub &'a [u8]);
pub struct TypeSection<'a>(pub &'a [u8]);
pub struct ImportSection<'a>(pub &'a [u8]);
pub struct FunctionSection<'a>(pub &'a [u8]);
pub struct TableSection<'a>(pub &'a [u8]);
pub struct MemorySection<'a>(pub &'a [u8]);
pub struct GlobalSection<'a>(pub &'a [u8]);
pub struct ExportSection<'a>(pub &'a [u8]);
pub struct StartSection<'a>(pub &'a [u8]);
pub struct ElementSection<'a>(pub &'a [u8]);
pub struct CodeSection<'a>(pub &'a [u8]);
pub struct DataSection<'a>(pub &'a [u8]);

impl<'a> Section<'a> {
    pub fn id(&self) -> u8 {
        use self::Section::*;
        match self {
            &Name{..} => 0,
            &Type{..} => 1,
            &Import{..} => 2,
            &Function{..} => 3,
            &Table{..} => 4,
            &Memory{..} => 5,
            &Global{..} => 6,
            &Export{..} => 7,
            &Start{..} => 8,
            &Element{..} => 9,
            &Code{..} => 10,
            &Data{..} => 11,
        }
    }
}

impl<'a> TypeSection<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn buf(&self) -> Buf<'a> {
        Buf::new(self.0)
    }

    pub fn count(&self) -> Result<u32, Error> {        
        self.buf().read_var_u32()
    }

    pub fn iter(&self) -> Result<TypeSectionIter<'a>, Error> {
        let mut buf = self.buf();
        let count = try!(buf.read_var_u32());
        Ok(TypeSectionIter { buf: buf, count: count, state: TypeSectionState::Form })
    }
}

pub enum TypeSectionState {
    Form,
    Param(u32),
    Return,
}

pub struct TypeSectionIter<'a> {
    buf: Buf<'a>,
    count: u32,
    state: TypeSectionState,
}

impl<'a> TypeSectionIter<'a> {
    fn next(&mut self) -> Result<Option<TypeSectionItem>, Error> {
        if self.count == 0 {
            return Ok(None)
        }
        match self.state {
            TypeSectionState::Form => {
                let form = try!(self.buf.read_var_i7());                
                let item = Some(TypeSectionItem::Form(form));                
                let param_count = try!(self.buf.read_var_u32());
                if param_count > 0 {
                    self.state = TypeSectionState::Param(param_count);
                    return Ok(item)
                }
                let return_count = try!(self.buf.read_var_u1());
                if return_count {
                    self.state = TypeSectionState::Return;
                    return Ok(item)
                }
                self.state = TypeSectionState::Form;
                self.count -= 1;
                Ok(item)
            },
            TypeSectionState::Param(param_count) => {
                let param_type = try!(self.buf.read_var_i7());
                let item = Some(TypeSectionItem::ParamType(param_type));
                if param_count > 0 {
                    self.state = TypeSectionState::Param(param_count - 1);
                    return Ok(item)
                }
                let return_count = try!(self.buf.read_var_u1());
                if return_count {
                    self.state = TypeSectionState::Return;
                    return Ok(item)
                }                
                self.state = TypeSectionState::Form;
                self.count -= 1;
                Ok(item)                
            },
            TypeSectionState::Return => {
                let return_type = try!(self.buf.read_var_i7());
                let item = Some(TypeSectionItem::ReturnType(return_type));
                self.state = TypeSectionState::Form;
                self.count -= 1;
                Ok(item)
            }
        }        
    }
}

pub enum TypeSectionItem {
    Form(i8),
    ParamType(i8),
    ReturnType(i8),
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = TypeSection(&BASIC[0x0a..0x11]);
        assert_eq!(s.len(), 0x7);
        assert_eq!(s.count().unwrap(), 1);
    }
}