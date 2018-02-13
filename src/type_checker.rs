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