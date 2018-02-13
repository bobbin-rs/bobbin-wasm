use {Error, TypeValue};
use stack::Stack;
use opcode::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelType {
    Func,
    Block,
    Loop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label {
    label_type: LabelType,
    signature: TypeValue,
    stack_limit: usize, 
    unreachable: bool,
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
        info!("get_label({})", depth);
        Ok(self.label_stack.peek(depth)?)
    }

    fn top_label(&self) -> Result<Label, Error> {
        self.get_label(0)
    }

    pub fn type_stack_size(&self) -> usize { self.type_stack.len() }
    pub fn label_stack_size(&self) -> usize { self.label_stack.len() }
    
    pub fn push_label(&mut self, label_type: LabelType, signature: TypeValue) -> Result<(), Error> {
        Ok({
            let stack_limit = self.type_stack.len();
            let unreachable = false;
            self.label_stack.push(Label {
                label_type,
                signature,
                stack_limit,
                unreachable,
            })?;
        })
    }

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

    pub fn reset_type_stack_to_label(&mut self, label: Label) -> Result<(), Error> {    
        self.type_stack.set_pos(label.stack_limit)?;
        Ok(())
    }

    pub fn drop_types(&mut self, drop_count: usize) -> Result<(), Error> {
        info!("drop_types({})", drop_count);
        let label = self.top_label()?;
        if label.stack_limit + drop_count > self.type_stack.len() {
            if label.unreachable {
                self.reset_type_stack_to_label(label)?;
                return Ok(())
            }
            return Err(Error::TypeCheck)
        }
        let len = self.type_stack.len();
        self.type_stack.erase(len - drop_count, len)?;
        Ok(())
    }

    pub fn peek_and_check_type(&mut self, depth: usize, sig: TypeValue) -> Result<(), Error> {
        let t = self.type_stack.peek(depth)?;
        if t == sig {
            Ok(())
        } else {
            return Err(Error::TypeCheck)?;
        }        
    }

    pub fn check_signature(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("check_signature({:?})", sig);
        self.peek_and_check_type(0, sig)
    }

    pub fn pop_and_check_signature(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("pop_and_check_signature({:?})", sig);
        self.check_signature(sig)?;
        self.drop_types(1)?;        
        Ok(())
    }

    pub fn begin_function(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("begin_function({:?})", sig);
        self.type_stack.reset()?;
        self.label_stack.reset()?;
        self.push_label(LabelType::Func, sig)?;
        Ok(())
    }


    pub fn type_check(&mut self, i: &Instruction) -> Result<(), Error> {        
        info!("L: {} T: {} | {}", self.label_stack.len(), self.type_stack.len(), i.op.text);
        match i.op.code {
            RETURN => {
                let label = self.get_label(0)?;
                self.pop_and_check_signature(label.signature)?;
                self.set_unreachable(true)?;
            },
            I32_CONST => {
                self.push_type(I32)?;
            },
            _ => {},
        }
        Ok(())
    }    
}



pub trait TypeStack {
    fn push_type<T: Into<TypeValue>>(&mut self, type_value: T) -> Result<(), Error>;
    fn pop_type(&mut self) -> Result<TypeValue, Error>;
    fn pop_type_expecting(&mut self, tv: TypeValue) -> Result<(), Error>;
    fn expect_type(&self, wanted: TypeValue) -> Result<(), Error>;
    fn expect_type_stack_depth(&self, wanted: u32) -> Result<(), Error>;
    fn type_drop_keep(&mut self, drop: u32, keep: u32) -> Result<(), Error>;
    fn erase(&mut self, bottom: usize, top: usize) -> Result<(), Error>;
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
    fn erase(&mut self, bottom: usize, top: usize) -> Result<(), Error> {
        for i in bottom..top {
            self.set(i, VOID)?;
        }
        Ok(())
    }
}