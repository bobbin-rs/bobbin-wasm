use {Error, TypeValue};
use stack::Stack;
use opcode::*;

use core::convert::TryFrom;
use core::fmt;


#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Label {
    opcode: u8,
    signature: TypeValue,
    offset: u32,
    fixup_offset: u32,
    stack_limit: u32,
    unreachable: bool,
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opc = Opcode::try_from(self.opcode).unwrap();
        write!(f, "Label {{ opcode: {} signature: {:?}, offset: 0x{:08x}, fixup_offset: 0x{:08x}, stack_limit: {} }}", opc.text, self.signature, self.offset, self.fixup_offset, self.stack_limit)
    }
}

pub struct TypeChecker<'m> {
    label_stack: Stack<'m, Label>,
    type_stack: Stack<'m, TypeValue>,
}

impl<'m> TypeChecker<'m> {
    pub fn new(label_stack: Stack<'m, Label>, type_stack: Stack<'m, TypeValue>) -> Self {
        TypeChecker { label_stack, type_stack }
    }

    fn get_label(&self, depth: usize) -> Result<Label, Error> {
        Ok(self.label_stack.peek(depth)?)
    }

    pub fn type_stack_size(&self) -> usize { self.type_stack.len() }
    pub fn label_stack_size(&self) -> usize { self.label_stack.len() }
    
    pub fn set_unreachable(&mut self, value: bool) -> Result<(), Error> {        
        // info!("UNREACHABLE: {}", value);
        Ok(self.label_stack.pick(0)?.unreachable = value)
    }

    pub fn is_unreachable(&self) -> Result<bool, Error> {
        Ok(self.label_stack.peek(0)?.unreachable)
    }

    pub fn push_type(&mut self, t: TypeValue) -> Result<(), Error> {
        Ok(self.type_stack.push(t)?)
    }

    pub fn pop_type(&mut self) -> Result<TypeValue, Error> {
        Ok(self.type_stack.pop()?)
    }
}



pub trait TypeStack {
    fn push_type<T: Into<TypeValue>>(&mut self, type_value: T) -> Result<(), Error>;
    fn pop_type(&mut self) -> Result<TypeValue, Error>;
    fn pop_type_expecting(&mut self, tv: TypeValue) -> Result<(), Error>;
    fn expect_type(&self, wanted: TypeValue) -> Result<(), Error>;
    fn expect_type_stack_depth(&self, wanted: u32) -> Result<(), Error>;
    fn type_drop_keep(&mut self, drop: u32, keep: u32) -> Result<(), Error>;
}

impl<'a> TypeStack for Stack<'a, TypeValue> {
    fn push_type<T: Into<TypeValue>>(&mut self, type_value: T) -> Result<(), Error> {
        let tv = type_value.into();
        // info!("-- type: {} <= {:?}", self.len(), tv);
        Ok(self.push(tv)?)
    }

    fn pop_type(&mut self) -> Result<TypeValue, Error> {
        // let depth = self.len();
        let tv = self.pop()?;
        // info!("-- type: {} => {:?}", depth, tv);
        Ok(tv)
    }

    fn pop_type_expecting(&mut self, tv: TypeValue) -> Result<(), Error> {
        if tv == TypeValue::Void || tv == TypeValue::None {
           Ok(()) 
        } else {
            let t = self.pop_type()?;
            if t == tv {
                Ok(())
            } else {
                Err(Error::UnexpectedType { wanted: tv, got: t })
            }
        }
    }

    fn expect_type(&self, wanted: TypeValue) -> Result<(), Error> {
        if wanted == TypeValue::Void || wanted == TypeValue::None {
            Ok(())
        } else {
            let got = self.top()?;
            if wanted != got {
                Err(Error::UnexpectedType { wanted, got })
            } else {
                Ok(())
            }
        }
    }

    fn expect_type_stack_depth(&self, wanted: u32) -> Result<(), Error> {
        let got = self.len() as u32;
        if wanted != got {
            Err(Error::UnexpectedTypeStackDepth { wanted, got })
        } else {
            Ok(())
        }
    }

    fn type_drop_keep(&mut self, drop: u32, keep: u32) -> Result<(), Error> {
        // info!("drop_keep {}, {}", drop,keep);
        self.drop_keep(drop as usize, keep as usize)?;
        Ok(())
    }    
}