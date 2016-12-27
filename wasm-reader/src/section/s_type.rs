use buf::Buf;
use {Error, Section};

pub struct TypeSection<'a>(pub Section<'a>);

impl<'a> TypeSection<'a> {
    pub fn name(&self) -> &str {
        "TYPE"
    }
    pub fn start(&self) -> usize {
        self.0.buf.pos()
    }
    pub fn end(&self) -> usize {
        self.0.buf.pos() + self.0.buf.remaining() 
    }
    pub fn len(&self) -> usize {
        self.0.buf.remaining()
    }
    pub fn count(&self) -> u32 {
        self.0.buf.clone().read_var_u32().unwrap()
    }

    pub fn iter(&self) -> TypeSectionIter<'a> {
        TypeSectionIter { buf: self.0.buf.clone(), count: None, index: 0, state: TypeSectionState::Form }
    }
}

#[derive(Debug)]
pub enum TypeSectionState {
    Form,
    Param(u32),
    Return,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeSectionItem {
    Form(i8),
    ParamType(i8),
    ReturnType(i8),
}

pub struct TypeSectionIter<'a> {
    buf: Buf<'a>,
    count: Option<u32>,
    index: u32,
    state: TypeSectionState,
}

impl<'a> TypeSectionIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<(u32, TypeSectionItem)>, Error> {
        if self.count.is_none() {
            self.count = Some(try!(self.buf.read_var_u32()));
        }
        if self.index == self.count.unwrap() {
            return Ok(None)
        }
        //println!("state: {:?}", self.state);
        match self.state {
            TypeSectionState::Form => {
                let form = try!(self.buf.read_var_i7());                
                let item = Some((self.index, TypeSectionItem::Form(form)));
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
                self.index += 1;
                Ok(item)
            },
            TypeSectionState::Param(param_count) => {
                let param_type = try!(self.buf.read_var_i7());
                let item = Some((self.index, TypeSectionItem::ParamType(param_type)));
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
                self.index += 1;
                Ok(item)                
            },
            TypeSectionState::Return => {
                let return_type = try!(self.buf.read_var_i7());
                let item = Some((self.index, TypeSectionItem::ReturnType(return_type)));
                self.state = TypeSectionState::Form;
                self.index += 1;
                Ok(item)
            }
        }        
    }
}

impl<'a> Iterator for TypeSectionIter<'a> {
    type Item = (u32, TypeSectionItem);
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASIC: &'static [u8] = include_bytes!("../../testdata/basic.wasm");    

    #[test]
    fn test_type() {
        let s = TypeSection(Section {id: 1, buf: Buf::new_slice(&BASIC, 0x0a, 0x7)});
        assert_eq!(s.len(), 0x7);
        assert_eq!(s.count(), 1);
        let mut iter = s.iter();

        assert_eq!(iter.next(), Some((0,TypeSectionItem::Form(-0x20))));
        assert_eq!(iter.next(), Some((0,TypeSectionItem::ParamType(-0x01))));
        assert_eq!(iter.next(), Some((0,TypeSectionItem::ParamType(-0x01))));
        assert_eq!(iter.next(), Some((0,TypeSectionItem::ReturnType(-0x01))));
        assert_eq!(iter.next(), None);
    }
}