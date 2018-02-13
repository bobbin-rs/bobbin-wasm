use {Error, TypeValue};
use stack::Stack;
use opcode::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelType {
    Func,
    Block,
    Loop,
    If,
    Else,
    Try,
    Catch,
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
            let label = Label {
                label_type,
                signature,
                stack_limit,
                unreachable,
            };
            let d = self.label_stack.len();
            info!("PUSH_LABEL: {} {:?}", d, label);
            self.label_stack.push(label)?;
        })
    }

    pub fn set_unreachable(&mut self, value: bool) -> Result<(), Error> {        
        info!("set_unreachable({})", value);
        {
            let label = self.label_stack.pick(0)?;
            label.unreachable = value;
        }
        let label = self.top_label()?;
        self.reset_type_stack_to_label(label)?;
        Ok(())
    }

    pub fn is_unreachable(&self) -> Result<bool, Error> {
        let value = self.label_stack.peek(0)?.unreachable;
        info!("is_unreachable() -> {}", value);
        Ok(value)
    }

    pub fn push_type(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("PUSH {}: {:?}", self.type_stack.len(), t);
        Ok(self.type_stack.push(t)?)
    }

    pub fn push_types(&mut self, types: &[TypeValue]) -> Result<(), Error> {
        Ok({
            for t in types {
                self.push_type(*t)?;
            }
        })
    }

    pub fn pop_type(&mut self) -> Result<TypeValue, Error> {        
        let d = self.type_stack.len();
        let v = self.type_stack.pop()?;
        info!("POP  {}: {:?}", d, v);
        Ok(v)
    }

    pub fn pop_label(&mut self) -> Result<Label, Error> {
        let d = self.label_stack.len();
        let label = self.label_stack.pop()?;
        info!("POP_LABEL:  {} {:?}", d, label);
        Ok(label)
    }    

    pub fn reset_type_stack_to_label(&mut self, label: Label) -> Result<(), Error> {    
        self.type_stack.set_pos(label.stack_limit)?;
        Ok(())
    }

    pub fn drop_types(&mut self, drop_count: usize) -> Result<(), Error> {
        info!("drop_types({})", drop_count);
        let label = self.top_label()?;
        info!("stack_limit: {} drop_count: {} type_stack: {}", label.stack_limit, drop_count, self.type_stack.len());
        if label.stack_limit + drop_count > self.type_stack.len() {
            if label.unreachable {
                self.reset_type_stack_to_label(label)?;
                return Ok(())
            }
            return Err(Error::TypeCheck("stack_limit + drop_count > len"))
        }
        let len = self.type_stack.len();
        self.type_stack.erase(len - drop_count, len)?;
        Ok(())
    }

    pub fn dump_type_stack(&self) -> Result<(), Error> {
        info!("--- TYPE STACK ---");
        for i in 0..self.type_stack.len() {
            info!("{}: {:?}", i, self.type_stack.get(i));
        }
        info!("--- END ---");
        Ok(())
    }

    pub fn peek_type(&mut self, depth: usize) -> Result<TypeValue, Error> {
        let label = self.top_label()?;
        if label.stack_limit + depth >= self.type_stack.len() {
            if label.unreachable {
                return Ok(TypeValue::Any);
            } else {
                return Err(Error::TypeCheck("invalid depth in peek_type"));
            }
        }
        Ok(self.type_stack.peek(depth)?)
    }

    pub fn check_type(&self, actual: TypeValue, expected: TypeValue) -> Result<(), Error> {
        info!("check_type({:?}, {:?})", actual, expected);
        if expected == actual || expected == TypeValue::Any || actual == TypeValue::Any {
            Ok(())
        } else {
            Err(Error::TypeCheck("incorrect signature"))
        }
    }

    pub fn peek_and_check_type(&mut self, depth: usize, expected: TypeValue) -> Result<(), Error> {
        info!("peek_and_check_type({}, {:?})", depth, expected);
        let t = self.peek_type(depth)?;
        info!("   -> type: {:?}", t);
        self.check_type(t, expected)?;
        Ok(())
    }

    pub fn check_type_stack_end(&mut self) -> Result<(), Error> {
        info!("check_stack_type_end()");
        let label = self.top_label()?;
        info!("   -> type_stack: {} stack_limit: {}", self.type_stack.len(), label.stack_limit);
        if self.type_stack.len() != label.stack_limit {
            return Err(Error::TypeCheck("type_stack length != label.stack_limit"))
        }
        Ok(())
    }

    pub fn check_signature(&mut self, sig: &[TypeValue]) -> Result<(), Error> {
        info!("check_signature({:?})", sig);

        for i in 0..sig.len() {
            self.peek_and_check_type(i, sig[i])?;
        }
        Ok(())

    }

    pub fn pop_and_check_signature(&mut self, sig: &[TypeValue]) -> Result<(), Error> {
        info!("pop_and_check_signature({:?})", sig);
        self.check_signature(sig)?;        
        self.drop_types(sig.len())?;
        Ok(())
    }

    pub fn pop_and_check_call(&mut self, parameters: &[TypeValue], returns: &[TypeValue]) -> Result<(), Error> {
        info!("pop_and_check_call({:?}, {:?})", parameters, returns);
        Ok({
            self.check_signature(parameters)?;
            self.drop_types(parameters.len())?;            
            self.push_types(returns)?;
        })
    }

    pub fn begin_function(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("begin_function({:?})", sig);
        self.type_stack.reset()?;
        self.label_stack.reset()?;
        self.push_label(LabelType::Func, sig)?;
        Ok(())
    }

    pub fn enter(&self) -> Result<(), Error> {
        Ok({
            info!("--- L: {} T: {} ---", self.label_stack.len(), self.type_stack.len());
        })
    }

    pub fn exit(&self) -> Result<(), Error> {
        Ok({
            self.dump_type_stack()?;
        })
    }    

    pub fn on_call(&mut self, parameters: &[TypeValue], result_types: &[TypeValue]) -> Result<(), Error> {
        info!("on_call({:?}, {:?})", parameters, result_types);
        Ok({
            self.pop_and_check_call(parameters, result_types)?;
        })
    }

    pub fn on_return(&mut self) -> Result<(), Error> {
        info!("on_return()");
        Ok({
            let label = self.get_label(0)?;
            info!("checking {:?}", label);            
            self.pop_and_check_signature(&[label.signature])?;
            self.set_unreachable(true)?;
        })
    }

    pub fn on_end(&mut self) -> Result<(), Error> {
        info!("on_end()");
        Ok({
            let label = self.get_label(0)?;                
            if let LabelType::If = label.label_type {
                if label.signature != VOID {
                    return Err(Error::TypeCheck("if without else cannot have type signature"))
                }
            }
            self.pop_and_check_signature(&[label.signature])?;
            self.check_type_stack_end()?;
            self.reset_type_stack_to_label(label)?;
            if label.signature != VOID {
                self.push_type(label.signature)?;
            }
            self.pop_label()?;            
        })
    }

    pub fn on_i32_const(&mut self) -> Result<(), Error> {
        info!("on_i32_const()");
        Ok({
            self.push_type(I32)?;
        })
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
        if tv == TypeValue::Void || tv == TypeValue::Any {
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
        if wanted == TypeValue::Void || wanted == TypeValue::Any {
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
        info!("erase({},{})", bottom, top);
        for i in bottom..top {            
            self.set(i, VOID)?;
        }
        Ok(())
    }
}