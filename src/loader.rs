use {Error, Event, TypeValue, Delegate, DelegateResult};

use module::*;
use opcode::*;
use writer::Writer;
use stack::Stack;

use core::fmt;
use core::ops::Index;
use core::convert::TryFrom;

pub const FIXUP_OFFSET: u32 = 0xffff_ffff;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Fixup {
    depth: u32,
    offset: u32,
}

impl fmt::Debug for Fixup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fixup {{ depth: {}, offset: 0x{:08x} }}", self.depth, self.offset)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Label {
    opcode: u8,
    signature: TypeValue,
    offset: u32,
    stack_limit: u32,
    unreachable: bool,
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opc = Opcode::try_from(self.opcode).unwrap();
        write!(f, "Label {{ opcode: {} signature: {:?}, offset: 0x{:08x}, stack_limit: {} }}", opc.text, self.signature, self.offset, self.stack_limit)
    }
}

#[derive(Debug)]
pub struct Context {
    parameters: [TypeValue; 16],
    parameters_count: usize,
    locals: [TypeValue; 16],
    locals_count: usize,
    return_type: TypeValue,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            parameters: [VOID; 16],
            parameters_count: 0,
            locals: [VOID; 16],
            locals_count: 0,
            return_type: VOID,
        }
    }
}

impl Context {
    fn reset(&mut self) {
        *self = Context::default()
    }

    fn len(&self) -> usize {
        self.parameters_count + self.locals_count
    }

    fn locals(&self) -> &[TypeValue] {
        &self.locals[..self.locals_count]
    }

    fn parameters(&self) -> &[TypeValue] {
        &self.parameters[..self.parameters_count]
    }

    fn set_parameters(&mut self, parameters: &[u8]) {
        for (i, p) in parameters.iter().enumerate() {
            self.parameters[i] = TypeValue::from(*p as i8);
        }
        self.parameters_count = parameters.len();
    }

    fn add_local(&mut self, n: u32, t: TypeValue) {
        for i in 0..n {
            self.locals[self.locals_count] = t;
            self.locals_count += 1;
        }
    }

    fn set_return(&mut self, t: TypeValue) {
        self.return_type = t;
    }

    fn return_type(&self) -> TypeValue {
        self.return_type
    }

    fn keep(&self) -> u32 {
        if self.return_type == VOID {
            0
        } else {
            1
        }
    }
}

impl<'t> From<Type<'t>> for Context {
    fn from(other: Type<'t>) -> Self {
        let mut c = Context::default();
        c.set_parameters(other.parameters);
        if other.returns.len() > 0 {
            c.set_return(TypeValue::from(other.returns[0] as i8));
        }
        c
    }
}

impl Index<usize> for Context {
    type Output = TypeValue;

    fn index(&self, i: usize) -> &Self::Output {
        if i < self.parameters_count {
            &self.parameters[i]
        } else {
            &self.locals[i - self.parameters_count]
        }
    }
}

pub struct Loader<'m, 'ls, 'ts> {
    w: Writer<'m>,
    module: Module<'m>,
    label_stack: Stack<'ls, Label>,
    type_stack: Stack<'ts, TypeValue>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
    section_fixup: usize,
    body_fixup: usize,
    context: Context,
}

impl<'m, 'ls, 'ts> Loader<'m, 'ls, 'ts> {
    pub fn new(module_buf: &'m mut [u8], label_buf: &'ls mut [Label], type_buf: &'ts mut[TypeValue]) -> Self {
        let mut w = Writer::new(module_buf);
        let module = Module::new(w.split());
        let label_stack = Stack::new(label_buf);
        let type_stack = Stack::new(type_buf);
        let fixups = [None; 256];
        let fixups_pos = 0;
        let section_fixup = 0;
        let body_fixup = 0;
        let context = Context::default();
        Loader { 
            w, 
            module, 
            label_stack, 
            type_stack, 
            fixups, 
            fixups_pos, 
            section_fixup, 
            body_fixup,
            context,
        }
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

    fn pos(&self) -> usize {
        self.w.pos()
    }

    fn write_i8(&mut self, value: i8) -> Result<(), Error> {
        self.w.write_i8(value)
    }

    fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        self.w.write_u8(value)
    }

    fn write_u32(&mut self, value: u32) -> Result<(), Error> {
        self.w.write_u32(value)
    }

    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.w.write_u8(op)
    }    
    fn write_type(&mut self, tv: TypeValue) -> Result<(), Error> {
        self.w.write_var_i7(tv.into())
    }
    fn write_block(&mut self, signature: TypeValue) -> Result<(), Error> {
        self.w.write_opcode(BLOCK)?;
        self.write_type(signature)?;
        Ok(())
    }        
    fn write_br(&mut self, depth: usize) -> Result<(), Error> {
        self.w.write_opcode(BR)?;
        self.w.write_var_u32(depth as u32)?;
        Ok(())
    }
    fn write_br_if(&mut self, depth: usize) -> Result<(), Error> {
        self.w.write_opcode(BR_IF)?;
        self.w.write_var_u32(depth as u32)?;
        Ok(())
    }
    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error> {
        info!("drop_keep {}, {}", drop_count, keep_count);
        if drop_count == 1 && keep_count == 0 {
            self.w.write_opcode(DROP)?;            
        } else if drop_count > 0 {
            self.w.write_opcode(INTERP_DROP_KEEP)?;
            self.w.write_u32(drop_count as u32)?;
            self.w.write_u32(keep_count as u32)?;
        }
        Ok(())
    }

    fn write_end(&mut self) -> Result<(), Error> { self.w.write_opcode(END) }
    fn write_i32_const(&mut self, value: i32)-> Result<(), Error> {
        self.w.write_opcode(I32_CONST)?;
        self.w.write_var_i32(value)?;
        Ok(())
    }

    fn write_alloca(&mut self, count: u32) -> Result<(), Error> {
        Ok(
            if count > 0 {
                self.w.write_opcode(INTERP_ALLOCA)?;
                self.w.write_u32(count as u32)?;
            }
        )
    }    

    pub fn push_label<T: Into<TypeValue>>(&mut self, opcode: u8, signature: T, offset: u32) -> Result<(), Error> {
        let stack_limit = self.type_stack.len() as u32;
        let label = Label {
            opcode,
            signature: signature.into(),
            offset,
            stack_limit,
            unreachable: false,
        };
        // info!("-- label: {} <= {:?}", self.label_stack.len(), label);
        Ok(self.label_stack.push(label)?)
    }

    pub fn pop_label(&mut self) -> Result<Label, Error> {
        // let depth = self.label_stack.len();
        let label = self.label_stack.pop()?;
        // info!("-- label: {} => {:?}", depth, label);
        Ok(label)
    }

    pub fn label_depth(&self) -> u32 {
        self.label_stack.len() as u32
    }

    pub fn peek_label(&self, offset: usize) -> Result<Label, Error> {
        Ok(self.label_stack.peek(offset)?)
    }

    pub fn set_unreachable(&mut self, value: bool) -> Result<(), Error> {        
        info!("UNREACHABLE: {}", value);
        Ok(self.label_stack.pick(0)?.unreachable = value)
    }

    pub fn is_unreachable(&self) -> Result<bool, Error> {
        Ok(self.label_stack.peek(0)?.unreachable)
    }

    // pub fn push_type<T: Into<TypeValue>>(&mut self, type_value: T) -> Result<(), Error> {
    //     let tv = type_value.into();
    //     info!("-- type: {} <= {:?}", self.type_stack.len(), tv);
    //     Ok(self.type_stack.push(tv)?)
    // }

    // pub fn pop_type(&mut self) -> Result<TypeValue, Error> {
    //     let depth = self.type_stack.len();
    //     let tv = self.type_stack.pop()?;
    //     info!("-- type: {} => {:?}", depth, tv);
    //     Ok(tv)
    // }

    // pub fn pop_type_expecting(&mut self, tv: TypeValue) -> Result<(), Error> {
    //     if tv == TypeValue::Void || tv == TypeValue::None {
    //        Ok(()) 
    //     } else {
    //         let t = self.pop_type()?;
    //         if t == tv {
    //             Ok(())
    //         } else {
    //             Err(Error::UnexpectedType { wanted: tv, got: t })
    //         }
    //     }
    // }

    // pub fn expect_type(&self, wanted: TypeValue) -> Result<(), Error> {
    //     if wanted == TypeValue::Void || wanted == TypeValue::None {
    //         Ok(())
    //     } else {
    //         let got = self.type_stack.top()?;
    //         if wanted != got {
    //             Err(Error::UnexpectedType { wanted, got })
    //         } else {
    //             Ok(())
    //         }
    //     }
    // }

    // pub fn expect_type_stack_depth(&self, wanted: u32) -> Result<(), Error> {
    //     let got = self.type_stack.len() as u32;
    //     if wanted != got {
    //         Err(Error::UnexpectedTypeStackDepth { wanted, got })
    //     } else {
    //         Ok(())
    //     }
    // }

    // pub fn type_drop_keep(&mut self, drop: u32, keep: u32) -> Result<(), Error> {
    //     info!("drop_keep {}, {}", drop,keep);
    //     self.type_stack.drop_keep(drop as usize, keep as usize)?;
    //     Ok(())
    // }

    pub fn add_fixup(&mut self, rel_depth: u32, offset: u32) -> Result<(), Error> {
        let depth = self.label_depth() - rel_depth;
        let fixup = Fixup { depth: depth, offset: offset };
        // info!("add_fixup: {:?}", fixup);
        for entry in self.fixups.iter_mut() {
            if entry.is_none() {
                *entry = Some(fixup);
                return Ok(());
            }
        }
        Err(Error::FixupsFull)
    }

    pub fn fixup(&mut self) -> Result<(), Error> {
        let depth = self.label_depth();        
        let offset = self.peek_label(0)?.offset;
        let offset = if offset == FIXUP_OFFSET { self.w.pos() } else { offset as usize};
        // info!("fixup: {} -> 0x{:08x}", depth, offset);
        for entry in self.fixups.iter_mut() {
            let del = if let &mut Some(entry) = entry {
                if entry.depth == depth {
                    // info!(" {:?}", entry);
                    self.w.write_u32_at(offset as u32, entry.offset as usize)?;
                    true
                } else {
                    // info!(" ! {} 0x{:04x}", entry.depth, entry.offset);                    
                    false
                }
            } else {
                false
            };
            if del {
                *entry = None;
            }
        }
        // info!("fixup done");
        Ok(())
    }

    fn get_drop_keep(&mut self, label: &Label) -> Result<(u32, u32), Error> {
        let drop = self.type_stack.len() as u32 - label.stack_limit;
        let drop = if self.is_unreachable()? { 0 } else { drop };
        Ok(
            if label.opcode == LOOP {
                (drop, 0)
            } else if label.signature == VOID {
                (drop, 0)
            } else {
                (drop - 1, 1)
            }
        )
    }

    pub fn type_check(&mut self, i: &Instruction) -> Result<(), Error> {
        let opc = i.op;
        match opc.code {
            IF => {
                self.type_stack.pop_type_expecting(I32)?;
            },
            ELSE | END => {
                let label = self.label_stack.top()?;
                info!("LABEL @ {}: {:?}", self.label_stack.len(), label);
                // info!("label depth: {}", self.label_stack.len());
                // info!("type depth: {}", self.type_stack.len());
                if label.opcode == IF && label.signature != VOID {
                    return Err(Error::InvalidIfSignature)
                }
                self.type_stack.expect_type(label.signature)?;
                if label.signature == TypeValue::Void {
                    self.type_stack.expect_type_stack_depth(label.stack_limit)?;
                } else {
                    self.type_stack.expect_type_stack_depth(label.stack_limit + 1)?;                    
                }                
            },
            BR => {
                let label = self.label_stack.top()?;
                let (drop, keep) = self.get_drop_keep(&label)?;
                self.type_stack.drop_keep(drop as usize, keep as usize)?;
                self.set_unreachable(true)?;
            },
            BR_IF => {
                self.type_stack.pop_type_expecting(I32)?;
                let label = self.label_stack.top()?;
                let (drop, keep) = self.get_drop_keep(&label)?;
                self.type_stack.drop_keep(drop as usize, keep as usize)?;
                self.set_unreachable(true)?;
            },            
            RETURN => {
                let return_type = self.context.return_type();
                self.type_stack.expect_type(return_type)?;                
                let drop = self.type_stack.len() as u32;
                let (drop, keep) = if return_type == TypeValue::Void {
                    (drop, 0)
                } else {
                    (drop - 1, 1)
                };
                
                self.type_stack.type_drop_keep(drop, keep)?;
                self.set_unreachable(true)?;                
            },
            _ => {
                self.type_stack.pop_type_expecting(opc.t1)?;
                self.type_stack.pop_type_expecting(opc.t2)?;
                if opc.tr != TypeValue::None {
                    self.type_stack.push_type(opc.tr)?;
                }
            }
        }
        Ok(())
    }

    pub fn write_fixup_u32(&mut self) -> Result<usize, Error> {
        let pos = self.w.pos();
        self.w.write_u32(FIXUP_OFFSET)?;
        Ok(pos)
    }

    pub fn apply_fixup_u32(&mut self, fixup: usize) -> Result<u32, Error> {
        let len = self.w.pos() - (fixup + 4);
        self.w.write_u32_at(len as u32, fixup)?;
        Ok(len as u32)
    }
}

impl<'m, 'ls, 'ts> Delegate for Loader<'m, 'ls, 'ts> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use Event::*;
        info!("{:08x}: {:?}", self.w.pos(), evt);
        match evt {
            Start { ref name, version } => {
                self.module.set_name(name);
                self.module.set_version(version);
            },
            SectionStart { s_type, s_beg: _, s_end:_ , s_len: _ } => {
                self.w.write_u8(s_type as u8)?;                
                self.section_fixup = self.write_fixup_u32()?;
            },
            SectionEnd => {
                let fixup = self.section_fixup;
                self.apply_fixup_u32(fixup)?;                
                self.module.extend(self.w.split());
            },
            TypesStart { c } => {
                self.w.write_u32(c)?;
            },
            TypeParametersStart { c } => {
                self.write_u8(c as u8)?;
            },
            TypeParameter { n: _, t } => {
                self.write_u8(t as u8)?;
            },
            TypeReturnsStart { c } => {
                self.write_u8(c as u8)?;
            },
            TypeReturn { n: _, t } => {
                self.write_u8(t as u8)?;
            },
            FunctionsStart { c } => {
                self.w.write_u32(c)?;
            },
            Function { n: _, index } => {
                self.w.write_u32(index.0)?;
            },
            ExportsStart { c } => {
                self.w.write_u32(c)?;
            },
            Export { n: _, id, index } => {
                let id = id.0;
                self.w.write_u32(id.len() as u32)?;
                for b in id {
                    self.write_u8(*b)?;
                }
                self.write_u8(index.kind())?;
                self.w.write_u32(index.index())?;
            },
            CodeStart { c } => {
                self.w.write_u32(c)?;
            },
            Body { n, offset: _, size: _, locals } => {
                self.context = Context::from(self.module.function_signature_type(n).unwrap());

                self.body_fixup = self.write_fixup_u32()?;
                self.write_u8(locals as u8)?;
            },
            Local { i: _, n, t } => {
                self.context.add_local(n, t);

                self.write_u8(n as u8)?;
                self.write_i8(t as i8)?;
            },
            InstructionsStart => {
                let locals = self.context.locals.len();
                self.write_alloca(locals as u32)?;
            }
            Instruction(i) => self.dispatch_instruction(i)?,
            InstructionsEnd => {},
            BodyEnd => {
                let fixup = self.body_fixup;
                self.apply_fixup_u32(fixup)?;                                
            },
            _ => {},    
        }
        Ok(())
    }
}

impl<'m, 'ls, 'ts> Loader<'m, 'ls, 'ts> {
    pub fn dispatch_instruction(&mut self, i: Instruction) -> DelegateResult {
        use opcode::Immediate::*;
        info!("{:08x}: V:{} | {}{:?}", i.offset, self.type_stack.len(), i.op.text, i.imm);

        // self.type_check(&i)?;   
        
        let op = i.op.code;
        match i.imm {
            None => {},
            Block { signature } => match op {
                BLOCK => {
                    self.push_label(op, signature, FIXUP_OFFSET)?;                    
                },
                LOOP => {
                    let pos = self.w.pos();
                    self.push_label(op, signature, pos as u32)?;                    
                },
                IF => {
                    self.push_label(op, signature, FIXUP_OFFSET)?;
                    info!("IF: DEPTH -> {}", self.label_depth());
                    self.w.write_opcode(INTERP_BR_UNLESS)?;
                    let pos = self.w.pos();
                    info!("IF: ADD FIXUP {} 0x{:04x}", 0, pos);
                    self.add_fixup(0, pos as u32)?;
                    self.w.write_u32(FIXUP_OFFSET)?;                    
                },
                _ => unreachable!(),
            },
            Branch { depth } => {
                let label = self.label_stack.peek(depth as usize)?;
                let (drop, keep) = self.get_drop_keep(&label)?;
                info!("drop_keep: {}, {}", drop, keep);
                self.write_drop_keep(drop, keep)?;
                self.w.write_opcode(op)?;
                let pos = self.pos();
                info!("BR / BR_IF ADD FIXUP {} 0x{:04x}", depth, self.pos());
                self.add_fixup(depth, pos as u32)?;
                self.w.write_u32(FIXUP_OFFSET)?;                
            },
            BranchTable { count } => {},
            BranchTableDepth { n, depth } => {},
            BranchTableDefault { depth } => {},
            Local { index } => {
                // Emits OP DEPTH_TO_LOCAL
                let id = index.0;

                if id >= self.context.len() as u32 {
                    return Err(Error::InvalidLocal { id: id })
                }

                let ty = self.context[id as usize];

                // let ty = if id < self.context.parameters.len() as u32 {
                //     TypeValue::from(self.context.parameters()[id as usize] as i8)
                // } else {
                //     locals[(id as usize) - parameters.len()]
                // };
                match op {
                    GET_LOCAL => self.type_stack.push_type(ty)?,
                    SET_LOCAL => self.type_stack.pop_type_expecting(ty)?,
                    TEE_LOCAL => {
                        self.type_stack.pop_type_expecting(ty)?;
                        self.type_stack.push_type(ty)?;
                    }
                    _ => unreachable!()
                }
                let depth = (self.type_stack.len() as u32) - id;
                self.w.write_opcode(op)?;
                self.w.write_u32(depth)?;                
            }
            Global { index } => {
                let id = index.0 as u32;
                let ty = {
                    let global = if let Some(global) = self.module.global(id) {
                        global
                    } else {
                        return Err(Error::InvalidGlobal { id: id })
                    };
                    TypeValue::from(global.global_type)
                };
                match op {
                    GET_GLOBAL => self.type_stack.push_type(ty)?,
                    SET_GLOBAL => self.type_stack.pop_type_expecting(ty)?,
                    _ => unreachable!()
                }
                self.w.write_opcode(op)?;
                self.w.write_u32(id as u32)?;                
            },
            Call { index } => {
                let id = index.0 as u32;
                let signature = if let Some(signature) = self.module.function_signature_type(id) {
                    signature
                } else {
                    return Err(Error::InvalidFunction { id: id })
                };
                let (parameters, returns) = (signature.parameters, signature.returns);
                if returns.len() > 1 {
                    return Err(Error::UnexpectedReturnLength { got: returns.len() as u32})
                }
                for p in parameters.iter() {
                    self.type_stack.pop_type_expecting(TypeValue::from(*p as i8))?;
                }
                for r in returns.iter() {
                    self.type_stack.push_type(TypeValue::from(*r as i8))?;
                }

                self.w.write_opcode(op)?;
                self.w.write_u32(id as u32)?;
            },
            CallIndirect { index } => {},
            I32Const { value } => {},
            F32Const { value } => {},
            I64Const { value } => {},
            F64Const { value } => {},
            LoadStore { align, offset } => {},
            Memory { reserved } => {},
        }

        // self.type_check(n, &i)?;   
        // info!("type check done");
        // match op {         
        //     END => {
        //         // w.write_opcode(op)?;
        //         // info!("FIXUP {} 0x{:04x}", self.label_depth(), w.pos());
        //         // info!("END");
        //         self.fixup(w)?;
        //         self.pop_label()?;
        //     },
        //     ELSE => {
        //         w.write_opcode(BR)?;
        //         self.fixup(w)?;
        //         let label = self.pop_label()?;
        //         self.push_label(op, label.signature, FIXUP_OFFSET)?;                    
        //         info!("ELSE: ADD FIXUP {} 0x{:04x}", 0, w.pos());
        //         self.add_fixup(0, w.pos() as u32)?;
        //         w.write_u32(FIXUP_OFFSET)?;
        //     },
        //     BR_TABLE => {
        //         // Emits BR_TABLE LEN [DROP OFFSET; LEN] [DROP OFFSET] KEEP

        //         // Verify top of stack contains the index
        //         self.pop_type_expecting(I32)?;
                
        //         w.write_opcode(op)?;
        //         let n = r.read_var_u32()? as usize;
        //         w.write_u32(n as u32)?;

        //         let mut sig: Option<TypeValue> = None;
        //         let mut sig_keep = 0;

        //         for _ in 0..n {
        //             let depth = r.read_var_u32()?;
        //             let label = self.label_stack.peek(depth as usize)?;
        //             self.expect_type(label.signature)?;
        //             let (drop, keep) = self.get_drop_keep(&label)?;
        //             info!("drop_keep: {}, {}", drop, keep);

        //             if sig.is_none() {
        //                 sig = Some(label.signature);
        //                 sig_keep = keep;
        //             }
                    
        //             w.write_u32(drop as u32)?;
        //             info!("BR_TABLE ADD FIXUP {} 0x{:04x}", depth, w.pos());
        //             self.add_fixup(depth, w.pos() as u32)?;
        //             w.write_u32(FIXUP_OFFSET)?;
        //         }
        //         {
        //             // Add default drop + offset
        //             let depth = r.read_var_u32()?;
        //             let label = self.label_stack.peek(depth as usize)?;
        //             self.expect_type(label.signature)?;
        //             let (drop, keep) = self.get_drop_keep(&label)?;
        //             info!("drop_keep: {}, {}", drop, keep);

        //             w.write_u32(drop as u32)?;
        //             info!("BR_TABLE ADD FIXUP {} 0x{:04x}", depth, w.pos());
        //             self.add_fixup(depth, w.pos() as u32)?;
        //             w.write_u32(FIXUP_OFFSET)?;
        //         }
        //         w.write_u32(sig_keep as u32)?;


        //     },
        //     UNREACHABLE => return Err(Error::Unreachable),
        //     RETURN => {
        //         let depth = self.type_stack.len() as u32;
        //         if return_type == VOID {
        //             w.write_drop_keep(depth, 0)?;
        //         } else {
        //             w.write_drop_keep(depth - 1, 1)?;
        //         }
        //         w.write_opcode(RETURN)?;
        //     },
        //     GET_GLOBAL | SET_GLOBAL => {
        //         let id = r.read_var_u32()?;
        //         let global = if let Some(global) = self.module.global(id) {
        //             global
        //         } else {
        //             return Err(Error::InvalidGlobal { id: id })
        //         };
        //         let ty = TypeValue::from(global.global_type);
        //         match op {
        //             GET_GLOBAL => self.push_type(ty)?,
        //             SET_GLOBAL => self.pop_type_expecting(ty)?,
        //             _ => unreachable!()
        //         }
        //         w.write_opcode(op)?;
        //         w.write_u32(id as u32)?;
        //     },
        //     CALL => {
        //         let id = r.read_var_u32()?;
        //         let signature = if let Some(signature) = self.module.function_signature_type(id) {
        //             signature
        //         } else {
        //             return Err(Error::InvalidFunction { id: id })
        //         };
        //         let (parameters, returns) = (signature.parameters, signature.returns);
        //         if returns.len() > 1 {
        //             return Err(Error::UnexpectedReturnLength { got: returns.len() as u32})
        //         }
        //         for p in parameters.iter() {
        //             self.pop_type_expecting(TypeValue::from(*p as i8))?;
        //         }
        //         for r in returns.iter() {
        //             self.push_type(TypeValue::from(*r as i8))?;
        //         }

        //         w.write_opcode(op)?;
        //         w.write_u32(id as u32)?;
        //     },
        //     CALL_INDIRECT => {
        //         // Emits OP SIG

        //         let id = r.read_var_u32()?;
        //         let _ = r.read_var_u1()?;
                
        //         let signature = if let Some(signature) = self.module.function_signature_type(id) {
        //             signature
        //         } else {
        //             return Err(Error::InvalidFunction { id: id })
        //         };

        //         let ret_count = signature.returns.len() as u32;
        //         if ret_count > 1 {
        //             return Err(Error::UnexpectedReturnLength { got: ret_count })
        //         }
        //         // Load function index
        //         self.pop_type_expecting(I32)?;
        //         for p in signature.parameters() {
        //             self.pop_type_expecting(p)?;
        //         }
        //         for r in signature.returns() {
        //             self.push_type(r)?;
        //         }                                     
        //         w.write_opcode(op)?;                    
        //         w.write_u32(id as u32)?;
        //     },
        //     I32_LOAD | I32_STORE | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U => {
        //         w.write_opcode(op)?;
        //         let a = r.read_var_u32()?;
        //         info!("  {:02x}", a);
        //         let b = r.read_var_u32()?;
        //         info!("  {:02x}", b);
        //         w.write_u32(a)?;
        //         w.write_u32(b)?;
        //     },
        //     MEM_GROW | MEM_SIZE => {
        //         w.write_opcode(op)?;
        //         r.read_var_u1()?;
        //     },
        //     I32_CONST => {
        //         w.write_opcode(op)?;
        //         let v = r.read_var_i32()?;
        //         info!(" {:08x}", v);
        //         w.write_i32(v)?;
        //     },
        //     DROP => {
        //         w.write_opcode(op)?;
        //         self.pop_type()?;
        //     },
        //     _ => {
        //         w.write_opcode(op)?;
        //     },
        // }        
        Ok(())
    }
}

pub trait LoaderWrite {
    fn write_opcode(&mut self, op: u8) -> Result<(), Error>;
}

impl<'a> LoaderWrite for Writer<'a> {
    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.write_u8(op)
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
        info!("-- type: {} <= {:?}", self.len(), tv);
        Ok(self.push(tv)?)
    }

    fn pop_type(&mut self) -> Result<TypeValue, Error> {
        let depth = self.len();
        let tv = self.pop()?;
        info!("-- type: {} => {:?}", depth, tv);
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
        info!("drop_keep {}, {}", drop,keep);
        self.drop_keep(drop as usize, keep as usize)?;
        Ok(())
    }    
}