use {Error, Event, TypeValue, Delegate, DelegateResult};

use module::*;
use opcode::*;
use writer::Writer;
use stack::Stack;

use core::fmt;
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


pub struct Loader<'w, 'ls, 'ts> {
    w: Writer<'w>,
    module: Module<'w>,
    label_stack: Stack<'ls, Label>,
    type_stack: Stack<'ts, TypeValue>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
    section_fixup: usize,
    body_fixup: usize,
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
        Loader { w, module, label_stack, type_stack, fixups, fixups_pos, section_fixup, body_fixup }
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.w.write_u8(op)
    }    
    fn write_type(&mut self, tv: TypeValue) -> Result<(), Error> {
        self.w.write_var_i7(tv.into())
    }
    fn write_block(&mut self, signature: TypeValue) -> Result<(), Error> {
        self.write_opcode(BLOCK)?;
        self.write_type(signature)?;
        Ok(())
    }        
    fn write_br(&mut self, depth: usize) -> Result<(), Error> {
        self.write_opcode(BR)?;
        self.w.write_var_u32(depth as u32)?;
        Ok(())
    }
    fn write_br_if(&mut self, depth: usize) -> Result<(), Error> {
        self.write_opcode(BR_IF)?;
        self.w.write_var_u32(depth as u32)?;
        Ok(())
    }
    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error> {
        info!("drop_keep {}, {}", drop_count, keep_count);
        if drop_count == 1 && keep_count == 0 {
            self.write_opcode(DROP)?;            
        } else if drop_count > 0 {
            self.write_opcode(INTERP_DROP_KEEP)?;
            self.w.write_u32(drop_count as u32)?;
            self.w.write_u32(keep_count as u32)?;
        }
        Ok(())
    }

    fn write_end(&mut self) -> Result<(), Error> { self.write_opcode(END) }
    fn write_i32_const(&mut self, value: i32)-> Result<(), Error> {
        self.write_opcode(I32_CONST)?;
        self.w.write_var_i32(value)?;
        Ok(())
    }

    fn write_alloca(&mut self, count: u32) -> Result<(), Error> {
        Ok(
            if count > 0 {
                self.write_opcode(INTERP_ALLOCA)?;
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
                self.w.write_u8(c as u8)?;
            },
            TypeParameter { n: _, t } => {
                self.w.write_u8(t as u8)?;
            },
            TypeReturnsStart { c } => {
                self.w.write_u8(c as u8)?;
            },
            TypeReturn { n: _, t } => {
                self.w.write_u8(t as u8)?;
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
                    self.w.write_u8(*b)?;
                }
                self.w.write_u8(index.kind())?;
                self.w.write_u32(index.index())?;
            },
            CodeStart { c } => {
                self.w.write_u32(c)?;
            },
            Body { n: _, offset: _, size: _, locals } => {
                self.body_fixup = self.write_fixup_u32()?;
                self.w.write_u8(locals as u8)?;
            },
            Local { i: _, n, t } => {
                self.w.write_u8(n as u8)?;
                self.w.write_i8(t as i8)?;
            },
            InstructionsStart { locals } => {
                self.write_alloca(locals)?;
            }
            Instruction(i) => self.dispatch_instruction(i)?,
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
        info!("{:08x}, {:?} {:?}", i.offset, i.op, i.imm);
        Ok(())
    }
}