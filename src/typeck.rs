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
    pub label_type: LabelType,
    pub signature: TypeValue,
    pub stack_limit: usize, 
    pub unreachable: bool,
}

pub struct TypeChecker<'m> {
    label_stack: Stack<'m, Label>,
    type_stack: Stack<'m, TypeValue>,
    br_table_sig: Option<TypeValue>,
}

impl<'m> TypeChecker<'m> {
    pub fn new(label_stack: Stack<'m, Label>, type_stack: Stack<'m, TypeValue>) -> Self {
        let br_table_sig = None;
        TypeChecker { label_stack, type_stack, br_table_sig }
    }

    pub fn get_label(&self, depth: usize) -> Result<Label, Error> {
        info!("  get_label({})", depth);
        Ok(self.label_stack.peek(depth)?)
    }

    pub fn get_label_ref(&mut self, depth: usize) -> Result<&mut Label, Error> {
        info!("  get_label_ref({})", depth);
        Ok(self.label_stack.pick(depth)?)
    }

    pub fn top_label(&self) -> Result<Label, Error> {
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
            info!("  PUSH_LABEL: {} {:?}", d, label);
            self.label_stack.push(label)?;
        })
    }

    pub fn set_unreachable(&mut self, value: bool) -> Result<(), Error> {        
        info!("  set_unreachable({})", value);
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
        info!("  is_unreachable() -> {}", value);
        Ok(value)
    }

    pub fn push_type(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("  PUSH {}: {:?}", self.type_stack.len(), t);
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
        info!("  POP  {}: {:?}", d, v);
        Ok(v)
    }

    pub fn pop_label(&mut self) -> Result<Label, Error> {
        let d = self.label_stack.len();
        let label = self.label_stack.pop()?;
        info!("  POP_LABEL:  {} {:?}", d, label);
        Ok(label)
    }    

    pub fn reset_type_stack_to_label(&mut self, label: Label) -> Result<(), Error> {    
        info!("  reset_type_stack_to_label({:?})", label);
        self.type_stack.set_pos(label.stack_limit)?;
        Ok(())
    }

    pub fn drop_types(&mut self, drop_count: usize) -> Result<(), Error> {
        info!("  drop_types({})", drop_count);
        let label = self.top_label()?;
        info!("    stack_limit: {} drop_count: {} type_stack: {}", label.stack_limit, drop_count, self.type_stack.len());
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
        info!("  peek_type({})", depth);
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
        info!("  check_type({:?}, {:?})", actual, expected);
        if expected == actual || expected == TypeValue::Any || actual == TypeValue::Any {
            Ok(())
        } else {
            Err(Error::TypeCheck("incorrect signature"))
        }
    }

    pub fn peek_and_check_type(&mut self, depth: usize, expected: TypeValue) -> Result<(), Error> {
        info!("  peek_and_check_type({}, {:?})", depth, expected);
        let t = self.peek_type(depth)?;
        info!("   -> type: {:?}", t);
        self.check_type(t, expected)?;
        Ok(())
    }

    pub fn check_type_stack_end(&mut self) -> Result<(), Error> {
        info!("  check_stack_type_end()");
        let label = self.top_label()?;
        info!("   -> type_stack: {} stack_limit: {}", self.type_stack.len(), label.stack_limit);
        if self.type_stack.len() != label.stack_limit {
            return Err(Error::TypeCheck("type_stack length != label.stack_limit"))
        }
        Ok(())
    }

    pub fn check_label_type(&mut self, label: Label, label_type: LabelType) -> Result<(), Error> {
        info!("  check_label_type({:?}, {:?})", label, label_type);
        if label.label_type == label_type {
            Ok(())
        } else {
            Err(Error::TypeCheck("mismatched label type"))
        }
    }

    pub fn check_signature(&mut self, sig: &[TypeValue]) -> Result<(), Error> {
        info!("  check_signature({:?})", sig);

        for i in 0..sig.len() {
            self.peek_and_check_type(i, sig[i])?;
        }
        Ok(())

    }

    pub fn pop_and_check_signature(&mut self, sig: &[TypeValue]) -> Result<(), Error> {
        info!("  pop_and_check_signature({:?})", sig);
        self.check_signature(sig)?;        
        self.drop_types(sig.len())?;
        Ok(())
    }

    pub fn pop_and_check_call(&mut self, parameters: &[TypeValue], returns: &[TypeValue]) -> Result<(), Error> {
        info!("  pop_and_check_call({:?}, {:?})", parameters, returns);
        Ok({
            self.check_signature(parameters)?;
            self.drop_types(parameters.len())?;            
            self.push_types(returns)?;
        })
    }

    pub fn pop_and_check_one_type(&mut self, expected: TypeValue) -> Result<(), Error> {
        info!("  pop_and_check_one_type({:?})", expected);
        Ok({
            self.peek_and_check_type(0, expected)?;
            self.drop_types(1)?;
        })
    }

    pub fn pop_and_check_two_types(&mut self, expected1: TypeValue, expected2: TypeValue) -> Result<(), Error> {
        info!("  pop_and_check_two_types({:?}, {:?})", expected1, expected2);
        Ok({
            self.peek_and_check_type(0, expected2)?;
            self.peek_and_check_type(1, expected1)?;
            self.drop_types(2)?;
        })
    }    

    pub fn pop_and_check_three_types(&mut self, expected1: TypeValue, expected2: TypeValue, expected3: TypeValue) -> Result<(), Error> {
        info!("  pop_and_check_two_types({:?}, {:?}, {:?})", expected1, expected2, expected3);
        Ok({
            self.peek_and_check_type(0, expected3)?;
            self.peek_and_check_type(1, expected2)?;
            self.peek_and_check_type(2, expected1)?;
            self.drop_types(3)?;
        })
    }    

    pub fn begin_function(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("begin_function({:?})", sig);
        self.type_stack.reset()?;
        self.label_stack.reset()?;
        self.push_label(LabelType::Func, sig)?;
        Ok(())
    }

    pub fn end_function(&mut self) -> Result<(), Error> {
        info!("end_function()");
        Ok({
            let label = self.top_label()?;
            self.check_label_type(label, LabelType::Func)?;
            self.on_end_label(label)?;
        })
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

    pub fn on_select(&mut self) -> Result<(), Error> {
        info!("on_select()");
        Ok({            
        //   Type type = Type::Any;
        //   result |= PeekAndCheckType(0, Type::I32);
        //   result |= PeekType(1, &type);
        //   result |= PeekAndCheckType(2, type);
        //   PrintStackIfFailed(result, "select", Type::I32, type, type);
        //   result |= DropTypes(3);
        //   PushType(type);            
            self.peek_and_check_type(0, TypeValue::I32)?;
            let t = self.peek_type(1)?;
            self.peek_and_check_type(2, t)?;
            self.drop_types(3)?;
            self.push_type(t)?;
        })
    }

    pub fn on_drop(&mut self) -> Result<(), Error> {
        info!("on_drop()");
        Ok({
            self.drop_types(1)?;
        })
    }


    pub fn on_block(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("on_block({:?})", sig);
        Ok({
            self.push_label(LabelType::Block, sig)?;
        })
    }

    pub fn on_loop(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("on_loop({:?})", sig);
        Ok({
            self.push_label(LabelType::Loop, sig)?;
        })
    }

    pub fn on_call(&mut self, parameters: &[TypeValue], result_types: &[TypeValue]) -> Result<(), Error> {
        info!("on_call({:?}, {:?})", parameters, result_types);
        Ok({
            self.pop_and_check_call(parameters, result_types)?;
        })
    }

    pub fn on_call_indirect(&mut self, parameters: &[TypeValue], result_types: &[TypeValue]) -> Result<(), Error> {
        info!("on_call_indirect({:?}, {:?})", parameters, result_types);
        Ok({
            self.pop_and_check_one_type(TypeValue::I32)?;
            self.pop_and_check_call(parameters, result_types)?;
        })
    }

    pub fn on_return(&mut self) -> Result<(), Error> {
        info!("on_return()");
        Ok({
            let label = self.get_label(0)?;
            info!("checking {:?}", label);         
            if label.signature != VOID {   
                self.pop_and_check_signature(&[label.signature])?;
            }
            self.set_unreachable(true)?;
        })
    }

    pub fn on_end_label(&mut self, label: Label) -> Result<(), Error> {
        info!("on_end_label({:?})", label);
        Ok({
            if label.signature != VOID {
                self.pop_and_check_signature(&[label.signature])?;
            }
            self.check_type_stack_end()?;
            self.reset_type_stack_to_label(label)?;
            if label.signature != VOID {
                self.push_type(label.signature)?;
            }
            self.pop_label()?;               
        })
    }

    pub fn on_if(&mut self, sig: TypeValue) -> Result<(), Error> {
        info!("on_if()");
        Ok({
            self.pop_and_check_one_type(I32)?;
            self.push_label(LabelType::If, sig)?;
        })
    }

    pub fn on_else(&mut self) -> Result<(), Error> {
        info!("on_else()");
        Ok({
            let label = self.get_label(0)?;                
            self.check_label_type(label, LabelType::If)?;
            if label.signature != VOID {
                self.pop_and_check_signature(&[label.signature])?;
            }
            self.check_type_stack_end()?;
            self.reset_type_stack_to_label(label)?;

            let label = self.get_label_ref(0)?;                
            label.label_type = LabelType::Else;
            label.unreachable = false;

            //   result |= CheckLabelType(label, LabelType::If);
            //   result |= PopAndCheckSignature(label->sig, "if true branch");
            //   result |= CheckTypeStackEnd("if true branch");
            //   ResetTypeStackToLabel(label);
            //   label->label_type = LabelType::Else;
            //   label->unreachable = false;
        })
    }

    pub fn on_end(&mut self) -> Result<(), Error> {
        info!("on_end()");
        Ok({
            let label = self.get_label(0)?;                
            if let LabelType::If = label.label_type {
                info!("IF signature: {:?}", label.signature);
                if label.signature != VOID {                    
                    return Err(Error::TypeCheck("if without else cannot have type signature"))
                }
            }
            self.on_end_label(label)?;
            info!("on_end() done");
        })
    }
    pub fn on_br(&mut self, depth: usize) -> Result<(), Error> {
        info!("on_br({})", depth);
        Ok({
            let label = self.get_label(depth)?;
            if label.label_type != LabelType::Loop && label.signature != TypeValue::Void {
                self.check_signature(&[label.signature])?;
            }
            self.set_unreachable(true)?;
            //   CHECK_RESULT(GetLabel(depth, &label));
            //   if (label->label_type != LabelType::Loop) {
            //     result |= CheckSignature(label->sig);
            //   }
            //   PrintStackIfFailed(result, "br", label->sig);
            //   CHECK_RESULT(SetUnreachable());
        })
    }
    pub fn on_br_if(&mut self, depth: usize) -> Result<(), Error> {
        info!("on_br_if({})", depth);
        Ok({
            let label = self.get_label(depth)?;
            if label.label_type != LabelType::Loop && label.signature != TypeValue::Void {
                self.pop_and_check_signature(&[label.signature])?;
                self.push_type(label.signature)?;
            }
            self.set_unreachable(true)?;            
            //   Result result = PopAndCheck1Type(Type::I32, "br_if");
            //   Label* label;
            //   CHECK_RESULT(GetLabel(depth, &label));
            //   if (label->label_type != LabelType::Loop) {
            //     result |= PopAndCheckSignature(label->sig, "br_if");
            //     PushTypes(label->sig);
            //   }
        })
    }

    pub fn begin_br_table(&mut self) -> Result<(), Error> {
        info!("begin_br_table()");
        Ok({
            self.br_table_sig = Some(TypeValue::Any);
            self.pop_and_check_one_type(TypeValue::I32)?;
        })
    }

    pub fn on_br_table_target(&mut self, depth: usize) -> Result<(), Error> {
        info!("on_br_table_target()");
        Ok({
            let label = self.get_label(depth)?;
            let label_sig = if label.label_type == LabelType::Loop {
                TypeValue::Void
            } else {
                label.signature
            };
            if label.signature != TypeValue::Void {
                self.check_signature(&[label.signature])?;
            }
            if let Some(br_table_sig) = self.br_table_sig {
                if self.check_type(br_table_sig, label_sig).is_err() {
                    return Err(Error::TypeCheck("br_table labels have inconsistent types"));
                }
            } else {
                panic!("br_table_target without begin_br_table call");
            }
            self.br_table_sig = Some(label_sig);
            info!("  => done");

            // CHECK_RESULT(GetLabel(depth, &label));
            //   Type label_sig;
            //   if (label->label_type == LabelType::Loop) {
            //     label_sig = Type::Void;
            //   } else {
            //     assert(label->sig.size() <= 1);
            //     label_sig = label->sig.size() == 0 ? Type::Void : label->sig[0];

            //     result |= CheckSignature(label->sig);
            //     PrintStackIfFailed(result, "br_table", label_sig);
            //   }

            //   // Make sure this label's signature is consistent with the previous labels'
            //   // signatures.
            //   if (Failed(CheckType(br_table_sig_, label_sig))) {
            //     result |= Result::Error;
            //     PrintError("br_table labels have inconsistent types: expected %s, got %s",
            //                GetTypeName(br_table_sig_), GetTypeName(label_sig));
            //   }
            //   br_table_sig_ = label_sig;

        })
    }

    pub fn end_br_table(&mut self) -> Result<(), Error> {
        info!("end_br_table()");
        Ok({
            self.set_unreachable(true)?;
            self.br_table_sig = None;
        })
    }

    pub fn on_get_local(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("on_get_local({})", t);
        Ok({
            self.push_type(t)?;
        })
    }

    pub fn on_set_local(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("on_set_local({})", t);
        Ok({
            self.pop_and_check_one_type(t)?;
        })
    }

    pub fn on_tee_local(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("on_tee_local({})", t);
        Ok({
            self.pop_and_check_one_type(t)?;
            self.push_type(t)?;
        })
    }    

    pub fn on_get_global(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("on_get_global({})", t);
        Ok({
            self.push_type(t)?;
        })
    }

    pub fn on_set_global(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("on_set_global({})", t);
        Ok({
            self.pop_and_check_one_type(t)?;
        })
    }


    pub fn check_opcode1(&mut self, op: &Opcode) -> Result<(), Error> {
        info!("check_opcode1({:?})", op.text);
        Ok({
            self.pop_and_check_one_type(op.t1)?;
            if op.tr != VOID {
                self.push_type(op.tr)?;
            }
        })
    }

    pub fn check_opcode2(&mut self, op: &Opcode) -> Result<(), Error> {
        info!("check_opcode2({:?})", op.text);
        Ok({
            self.pop_and_check_two_types(op.t1, op.t2)?;
            if op.tr != VOID {
                self.push_type(op.tr)?;            
            }
        })
    }

    pub fn on_load(&mut self, op: &Opcode) -> Result<(), Error> {
        self.check_opcode1(op)
    }

    pub fn on_store(&mut self, op: &Opcode) -> Result<(), Error> {
        self.check_opcode2(op)
    }    

    pub fn on_const(&mut self, t: TypeValue) -> Result<(), Error> {
        info!("on_const({:?})", t);
        Ok({
            self.push_type(t)?;
        })
    }
    
    pub fn on_unary(&mut self, op: &Opcode) -> Result<(), Error> {
        self.check_opcode1(op)
    }

    pub fn on_binary(&mut self, op: &Opcode) -> Result<(), Error> {
        self.check_opcode2(op)
    }

    pub fn on_current_memory(&mut self) -> Result<(), Error> {
        self.push_type(TypeValue::I32)
    }

    pub fn on_grow_memory(&mut self, op: &Opcode) -> Result<(), Error> {
        self.check_opcode1(op)
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
        info!("  erase({},{})", bottom, top);
        Ok({
            if top == self.len() {
                self.set_pos(bottom)?;
            }
            info!("new len: {}", self.len());
            // for i in bottom..top {            
            //     self.set(i, VOID)?;
            // }
        })
    }
}