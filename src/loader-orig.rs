use {Error, Event, TypeValue, Delegate, DelegateResult};

use module::*;
use opcode::*;
use writer::Writer;
use stack::Stack;

use core::fmt;
use core::convert::TryFrom;

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


pub const FIXUP_OFFSET: u32 = 0xffff_ffff;

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


pub struct Loader<'m, 's, 't> {
    module: &'m Module<'m>,
    label_stack: Stack<'s, Label>,
    type_stack: Stack<'t, TypeValue>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
}

impl<'m, 's, 't> Loader<'m, 's, 't> {
    pub fn new(module: &'m Module<'m>, label_stack: Stack<'s, Label>, type_stack: Stack<'t, TypeValue>) -> Self {
        Loader {
            module: module,
            label_stack: label_stack,
            type_stack: type_stack,
            fixups: [None; 256],
            fixups_pos: 0,
        }
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

    pub fn push_type<T: Into<TypeValue>>(&mut self, type_value: T) -> Result<(), Error> {
        let tv = type_value.into();
        info!("-- type: {} <= {:?}", self.type_stack.len(), tv);
        Ok(self.type_stack.push(tv)?)
    }

    pub fn pop_type(&mut self) -> Result<TypeValue, Error> {
        let depth = self.type_stack.len();
        let tv = self.type_stack.pop()?;
        info!("-- type: {} => {:?}", depth, tv);
        Ok(tv)
    }

    pub fn pop_type_expecting(&mut self, tv: TypeValue) -> Result<(), Error> {
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

    pub fn expect_type(&self, wanted: TypeValue) -> Result<(), Error> {
        if wanted == TypeValue::Void || wanted == TypeValue::None {
            Ok(())
        } else {
            let got = self.type_stack.top()?;
            if wanted != got {
                Err(Error::UnexpectedType { wanted, got })
            } else {
                Ok(())
            }
        }
    }

    pub fn expect_type_stack_depth(&self, wanted: u32) -> Result<(), Error> {
        let got = self.type_stack.len() as u32;
        if wanted != got {
            Err(Error::UnexpectedTypeStackDepth { wanted, got })
        } else {
            Ok(())
        }
    }

    pub fn type_drop_keep(&mut self, drop: u32, keep: u32) -> Result<(), Error> {
        info!("drop_keep {}, {}", drop,keep);
        self.type_stack.drop_keep(drop as usize, keep as usize)?;
        Ok(())
    }

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
        let offset = if offset == FIXUP_OFFSET { w.pos() } else { offset as usize};
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

    pub fn type_check(&mut self, opc: Opcode, return_type: TypeValue) -> Result<(), Error> {
        match opc.code {
            IF => {
                self.pop_type_expecting(I32)?;
            },
            ELSE | END => {
                let label = self.label_stack.top()?;
                info!("LABEL @ {}: {:?}", self.label_stack.len(), label);
                // info!("label depth: {}", self.label_stack.len());
                // info!("type depth: {}", self.type_stack.len());
                if label.opcode == IF && label.signature != VOID {
                    return Err(Error::InvalidIfSignature)
                }
                self.expect_type(label.signature)?;
                if label.signature == TypeValue::Void {
                    self.expect_type_stack_depth(label.stack_limit)?;
                } else {
                    self.expect_type_stack_depth(label.stack_limit + 1)?;                    
                }                
            },
            BR => {
                let label = self.label_stack.top()?;
                let (drop, keep) = self.get_drop_keep(&label)?;
                self.type_stack.drop_keep(drop as usize, keep as usize)?;
                self.set_unreachable(true)?;
            },
            BR_IF => {
                self.pop_type_expecting(I32)?;
                let label = self.label_stack.top()?;
                let (drop, keep) = self.get_drop_keep(&label)?;
                self.type_stack.drop_keep(drop as usize, keep as usize)?;
                self.set_unreachable(true)?;
            },            
            RETURN => {
                self.expect_type(return_type)?;                
                let drop = self.type_stack.len() as u32;
                let (drop, keep) = if return_type == TypeValue::Void {
                    (drop, 0)
                } else {
                    (drop - 1, 1)
                };
                self.type_drop_keep(drop, keep)?;
                self.set_unreachable(true)?;                
            },
            _ => {
                self.pop_type_expecting(opc.t1)?;
                self.pop_type_expecting(opc.t2)?;
                if opc.tr != TypeValue::None {
                    self.push_type(opc.tr)?;
                }
            }
        }
        Ok(())
    }

    // pub fn load(&mut self,
    //     index: u32, 
    //     locals: &[TypeValue], 
    //     r: &mut Reader,
    //     w: &mut Writer
    // ) -> Result<(), Error> {
    //     // push function start onto control stack

    //     let signature_type = self.module.function_signature_type(index).unwrap();
    //     let parameters = &signature_type.parameters;
    //     let return_type = match signature_type.returns.len() {
    //         0 => VOID,
    //         1 => TypeValue::from(signature_type.returns[0] as i8),
    //         _ => return Err(Error::InvalidReturnType),
    //     };

    //     self.push_label(0, return_type, r.pos() as u32)?;

    //     let mut locals_count = 0;

    //     for p in parameters.iter() {
    //         self.push_type(TypeValue::from(*p as i8))?;
    //         locals_count += 1;
    //     }

    //     for local in locals.iter() {
    //         self.push_type(*local)?;
    //         locals_count += 1;
    //     }        
    //     w.write_alloca(locals.len())?;

    //     while r.remaining() > 0 {
    //         let op = r.read_opcode()?;
    //         let opc = Opcode::try_from(op)?;
    //         info!("{:04x}/{:04x}: V:{} | {} ", r.pos(), w.pos(), self.type_stack.len(), opc.text);
    //         // info!("type check");
    //         self.type_check(opc, return_type)?;   
    //         // info!("type check done");
    //         match op {
    //             BLOCK => {
    //                 self.push_label(op, r.read_var_i7()?, FIXUP_OFFSET)?;
    //             },
    //             LOOP => {
    //                 self.push_label(op, r.read_var_i7()?, w.pos() as u32)?;
    //             },
    //             IF => {
    //                 self.push_label(op, r.read_var_i7()?, FIXUP_OFFSET)?;
    //                 info!("IF: DEPTH -> {}", self.label_depth());
    //                 w.write_opcode(INTERP_BR_UNLESS)?;
    //                 info!("IF: ADD FIXUP {} 0x{:04x}", 0, w.pos());
    //                 self.add_fixup(0, w.pos() as u32)?;
    //                 w.write_u32(FIXUP_OFFSET)?;
    //             },                
    //             END => {
    //                 // w.write_opcode(op)?;
    //                 // info!("FIXUP {} 0x{:04x}", self.label_depth(), w.pos());
    //                 // info!("END");
    //                 self.fixup(w)?;
    //                 self.pop_label()?;
    //             },
    //             ELSE => {
    //                 w.write_opcode(BR)?;
    //                 self.fixup(w)?;
    //                 let label = self.pop_label()?;
    //                 self.push_label(op, label.signature, FIXUP_OFFSET)?;                    
    //                 info!("ELSE: ADD FIXUP {} 0x{:04x}", 0, w.pos());
    //                 self.add_fixup(0, w.pos() as u32)?;
    //                 w.write_u32(FIXUP_OFFSET)?;
    //             }
    //             BR | BR_IF => {
    //                 let depth = r.read_var_u32()?;
    //                 let label = self.label_stack.peek(depth as usize)?;
    //                 let (drop, keep) = self.get_drop_keep(&label)?;
    //                 info!("drop_keep: {}, {}", drop, keep);
    //                 w.write_drop_keep(drop, keep)?;
    //                 w.write_opcode(op)?;
    //                 info!("BR / BR_IF ADD FIXUP {} 0x{:04x}", depth, w.pos());
    //                 self.add_fixup(depth, w.pos() as u32)?;
    //                 w.write_u32(FIXUP_OFFSET)?;
    //             },
    //             BR_TABLE => {
    //                 // Emits BR_TABLE LEN [DROP OFFSET; LEN] [DROP OFFSET] KEEP

    //                 // Verify top of stack contains the index
    //                 self.pop_type_expecting(I32)?;
                    
    //                 w.write_opcode(op)?;
    //                 let n = r.read_var_u32()? as usize;
    //                 w.write_u32(n as u32)?;

    //                 let mut sig: Option<TypeValue> = None;
    //                 let mut sig_keep = 0;

    //                 for _ in 0..n {
    //                     let depth = r.read_var_u32()?;
    //                     let label = self.label_stack.peek(depth as usize)?;
    //                     self.expect_type(label.signature)?;
    //                     let (drop, keep) = self.get_drop_keep(&label)?;
    //                     info!("drop_keep: {}, {}", drop, keep);

    //                     if sig.is_none() {
    //                         sig = Some(label.signature);
    //                         sig_keep = keep;
    //                     }
                        
    //                     w.write_u32(drop as u32)?;
    //                     info!("BR_TABLE ADD FIXUP {} 0x{:04x}", depth, w.pos());
    //                     self.add_fixup(depth, w.pos() as u32)?;
    //                     w.write_u32(FIXUP_OFFSET)?;
    //                 }
    //                 {
    //                     // Add default drop + offset
    //                     let depth = r.read_var_u32()?;
    //                     let label = self.label_stack.peek(depth as usize)?;
    //                     self.expect_type(label.signature)?;
    //                     let (drop, keep) = self.get_drop_keep(&label)?;
    //                     info!("drop_keep: {}, {}", drop, keep);

    //                     w.write_u32(drop as u32)?;
    //                     info!("BR_TABLE ADD FIXUP {} 0x{:04x}", depth, w.pos());
    //                     self.add_fixup(depth, w.pos() as u32)?;
    //                     w.write_u32(FIXUP_OFFSET)?;
    //                 }
    //                 w.write_u32(sig_keep as u32)?;


    //             },
    //             UNREACHABLE => return Err(Error::Unreachable),
    //             RETURN => {
    //                 let depth = self.type_stack.len() as u32;
    //                 if return_type == VOID {
    //                     w.write_drop_keep(depth, 0)?;
    //                 } else {
    //                     w.write_drop_keep(depth - 1, 1)?;
    //                 }
    //                 w.write_opcode(RETURN)?;
    //             },
    //             GET_LOCAL | SET_LOCAL | TEE_LOCAL => {
    //                 // Emits OP DEPTH_TO_LOCAL
    //                 let id = r.read_var_u32()?;
    //                 let len = locals_count;
    //                 if id >= len {
    //                     return Err(Error::InvalidLocal { id: id })
    //                 }

    //                 let ty = if id < parameters.len() as u32 {
    //                     TypeValue::from(parameters[id as usize] as i8)
    //                 } else {
    //                     locals[(id as usize) - parameters.len()]
    //                 };
    //                 match op {
    //                     GET_LOCAL => self.push_type(ty)?,
    //                     SET_LOCAL => self.pop_type_expecting(ty)?,
    //                     TEE_LOCAL => {
    //                         self.pop_type_expecting(ty)?;
    //                         self.push_type(ty)?;
    //                     }
    //                     _ => unreachable!()
    //                 }
    //                 let depth = (self.type_stack.len() as u32) - id;
    //                 w.write_opcode(op)?;
    //                 w.write_u32(depth as u32)?;
    //             },
    //             GET_GLOBAL | SET_GLOBAL => {
    //                 let id = r.read_var_u32()?;
    //                 let global = if let Some(global) = self.module.global(id) {
    //                     global
    //                 } else {
    //                     return Err(Error::InvalidGlobal { id: id })
    //                 };
    //                 let ty = TypeValue::from(global.global_type);
    //                 match op {
    //                     GET_GLOBAL => self.push_type(ty)?,
    //                     SET_GLOBAL => self.pop_type_expecting(ty)?,
    //                     _ => unreachable!()
    //                 }
    //                 w.write_opcode(op)?;
    //                 w.write_u32(id as u32)?;
    //             },
    //             CALL => {
    //                 let id = r.read_var_u32()?;
    //                 let signature = if let Some(signature) = self.module.function_signature_type(id) {
    //                     signature
    //                 } else {
    //                     return Err(Error::InvalidFunction { id: id })
    //                 };
    //                 let (parameters, returns) = (signature.parameters, signature.returns);
    //                 if returns.len() > 1 {
    //                     return Err(Error::UnexpectedReturnLength { got: returns.len() as u32})
    //                 }
    //                 for p in parameters.iter() {
    //                     self.pop_type_expecting(TypeValue::from(*p as i8))?;
    //                 }
    //                 for r in returns.iter() {
    //                     self.push_type(TypeValue::from(*r as i8))?;
    //                 }

    //                 w.write_opcode(op)?;
    //                 w.write_u32(id as u32)?;
    //             },
    //             CALL_INDIRECT => {
    //                 // Emits OP SIG

    //                 let id = r.read_var_u32()?;
    //                 let _ = r.read_var_u1()?;
                    
    //                 let signature = if let Some(signature) = self.module.function_signature_type(id) {
    //                     signature
    //                 } else {
    //                     return Err(Error::InvalidFunction { id: id })
    //                 };

    //                 let ret_count = signature.returns.len() as u32;
    //                 if ret_count > 1 {
    //                     return Err(Error::UnexpectedReturnLength { got: ret_count })
    //                 }
    //                 // Load function index
    //                 self.pop_type_expecting(I32)?;
    //                 for p in signature.parameters() {
    //                     self.pop_type_expecting(p)?;
    //                 }
    //                 for r in signature.returns() {
    //                     self.push_type(r)?;
    //                 }                                     
    //                 w.write_opcode(op)?;                    
    //                 w.write_u32(id as u32)?;
    //             },
    //             I32_LOAD | I32_STORE | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U => {
    //                 w.write_opcode(op)?;
    //                 let a = r.read_var_u32()?;
    //                 info!("  {:02x}", a);
    //                 let b = r.read_var_u32()?;
    //                 info!("  {:02x}", b);
    //                 w.write_u32(a)?;
    //                 w.write_u32(b)?;
    //             },
    //             MEM_GROW | MEM_SIZE => {
    //                 w.write_opcode(op)?;
    //                 r.read_var_u1()?;
    //             },
    //             I32_CONST => {
    //                 w.write_opcode(op)?;
    //                 let v = r.read_var_i32()?;
    //                 info!(" {:08x}", v);
    //                 w.write_i32(v)?;
    //             },
    //             DROP => {
    //                 w.write_opcode(op)?;
    //                 self.pop_type()?;
    //             },
    //             _ => {
    //                 w.write_opcode(op)?;
    //             },
    //         }
    //     }        

    //     info!("{:04x}: V:{} | {} ", w.pos(), self.type_stack.len(), "EXIT");
    //     // Check Exit        
    //     self.expect_type(return_type)?;
    //     self.expect_type_stack_depth(if return_type == VOID { 0 } else { 1 })?;
    //     // self.expect_type_stack_depth(locals.len())?;

    //     // if locals.len() > 0 {
    //     //     if signature != VOID {
    //     //         w.write_drop_keep(locals.len(), 1)?;
    //     //     } else {
    //     //         w.write_drop_keep(locals.len(), 0)?;
    //     //     }
    //     // }

    //     // info!("Checking Fixups");
    //     for entry in self.fixups.iter() {
    //         if let &Some(entry) = entry {   
    //             info!("{:?}", entry);
    //             panic!("Orphan Fixup: {:?}", entry);
    //         }
    //     }
    //     Ok(())
    // }
}

impl<'m, 's, 't> Delegate for Loader<'m, 's, 't> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        Ok(())
    }

}


pub trait WriteLoader {
    fn write_opcode(&mut self, op: u8) -> Result<(), Error>;
    fn write_type(&mut self, tv: TypeValue) -> Result<(), Error>;
    fn write_block(&mut self, signature: TypeValue) -> Result<(), Error>;
    fn write_br(&mut self, depth: usize) -> Result<(), Error>;
    fn write_br_if(&mut self, depth: usize) -> Result<(), Error>;
    fn write_end(&mut self) -> Result<(), Error>;
    fn write_drop_keep(&mut self, drop_count: u32, keep_size: u32) -> Result<(), Error>;
    fn write_alloca(&mut self, count: usize) -> Result<(), Error>;
    fn write_i32_const(&mut self, value: i32)-> Result<(), Error>;
}

impl<'w> WriteLoader for Writer<'w> {
    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.write_u8(op)
    }    
    fn write_type(&mut self, tv: TypeValue) -> Result<(), Error> {
        self.write_var_i7(tv.into())
    }
    fn write_block(&mut self, signature: TypeValue) -> Result<(), Error> {
        self.write_opcode(BLOCK)?;
        self.write_type(signature)?;
        Ok(())
    }        
    fn write_br(&mut self, depth: usize) -> Result<(), Error> {
        self.write_opcode(BR)?;
        self.write_var_u32(depth as u32)?;
        Ok(())
    }
    fn write_br_if(&mut self, depth: usize) -> Result<(), Error> {
        self.write_opcode(BR_IF)?;
        self.write_var_u32(depth as u32)?;
        Ok(())
    }
    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error> {
        info!("drop_keep {}, {}", drop_count, keep_count);
        if drop_count == 1 && keep_count == 0 {
            self.write_opcode(DROP)?;            
        } else if drop_count > 0 {
            self.write_opcode(INTERP_DROP_KEEP)?;
            self.write_u32(drop_count as u32)?;
            self.write_u32(keep_count as u32)?;
        }
        Ok(())
    }
    fn write_alloca(&mut self, count: usize) -> Result<(), Error> {
        Ok(
            if count > 0 {
                self.write_opcode(INTERP_ALLOCA)?;
                self.write_u32(count as u32)?;
            }
        )
    }
    fn write_end(&mut self) -> Result<(), Error> { self.write_opcode(END) }
    fn write_i32_const(&mut self, value: i32)-> Result<(), Error> {
        self.write_opcode(I32_CONST)?;
        self.write_var_i32(value)?;
        Ok(())
    }
}