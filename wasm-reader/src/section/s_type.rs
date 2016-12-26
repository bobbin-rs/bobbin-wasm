use buf::Buf;
use Error;

pub struct TypeSection<'a>(pub &'a [u8]);

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

#[derive(Debug)]
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
    pub fn next(&mut self) -> Result<Option<TypeSectionItem>, Error> {
        if self.count == 0 {
            return Ok(None)
        }
        //println!("state: {:?}", self.state);
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
                if param_count > 1 {
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

#[derive(Debug, PartialEq, Eq)]
pub enum TypeSectionItem {
    Form(i8),
    ParamType(i8),
    ReturnType(i8),
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = TypeSection(&BASIC[0x0a..0x11]);
        assert_eq!(s.len(), 0x7);
        assert_eq!(s.count().unwrap(), 1);
        let mut iter = s.iter().unwrap();

        assert_eq!(iter.next().unwrap(), Some(TypeSectionItem::Form(-0x20)));
        assert_eq!(iter.next().unwrap(), Some(TypeSectionItem::ParamType(-0x01)));
        assert_eq!(iter.next().unwrap(), Some(TypeSectionItem::ParamType(-0x01)));
        assert_eq!(iter.next().unwrap(), Some(TypeSectionItem::ReturnType(-0x01)));
        assert_eq!(iter.next().unwrap(), None);
    }
}