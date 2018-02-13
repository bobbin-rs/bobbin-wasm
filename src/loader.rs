use {Error, Event, TypeValue, Delegate, DelegateResult};

use module;
use module::*;
use module::ModuleWrite;
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
        for _ in 0..n {
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

pub struct Config {
    pub compile: bool
}

impl Default for Config {
    fn default() -> Config {
        let compile = true;

        Config { compile }
    }
}

pub struct Loader<'m> {
    cfg: Config,
    w: Writer<'m>,
    module: Module<'m>,
    label_stack: Stack<'m, Label>,
    type_stack: Stack<'m, TypeValue>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
    section_fixup: usize,
    body_fixup: usize,
    context: Context,
}

impl<'m> Loader<'m> {
    pub fn new(module_buf: &'m mut [u8]) -> Self {
        Loader::new_with_config(Config::default(), module_buf)
    }
    pub fn new_with_config(cfg: Config, module_buf: &'m mut [u8]) -> Self {
        let mut w = Writer::new(module_buf);
        let module = module::Module::new();

        // These should be not be allocated from module storage
        let label_stack = w.alloc_stack(16);        
        let type_stack = w.alloc_stack(16);

        // TODO: Break out into separate struct
        let fixups = [None; 256];
        let fixups_pos = 0;

        let section_fixup = 0;
        let body_fixup = 0;
        let context = Context::default();
        Loader { 
            cfg,
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

    pub fn module(self) -> (Module<'m>, &'m mut[u8]) {
        (self.module, self.w.into_slice())
    }

    fn pos(&self) -> usize {
        self.w.pos()
    }

    // fn write_i8(&mut self, value: i8) -> Result<(), Error> {
    //     self.w.write_i8(value)
    // }

    // fn write_u8(&mut self, value: u8) -> Result<(), Error> {
    //     self.w.write_u8(value)
    // }

    // fn write_u32(&mut self, value: u32) -> Result<(), Error> {
    //     self.w.write_u32(value)
    // }

    // fn write_i32_const(&mut self, value: i32)-> Result<(), Error> {
    //     self.w.write_opcode(I32_CONST)?;
    //     self.w.write_var_i32(value)?;
    //     Ok(())
    // }



    fn push_label<T: Into<TypeValue>>(&mut self, opcode: u8, signature: T, offset: u32) -> Result<(), Error> {
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

    fn pop_label(&mut self) -> Result<Label, Error> {
        // let depth = self.label_stack.len();
        let label = self.label_stack.pop()?;
        // info!("-- label: {} => {:?}", depth, label);
        Ok(label)
    }

    fn label_depth(&self) -> u32 {
        self.label_stack.len() as u32
    }

    fn peek_label(&self, offset: usize) -> Result<Label, Error> {
        Ok(self.label_stack.peek(offset)?)
    }

    fn set_unreachable(&mut self, value: bool) -> Result<(), Error> {        
        // info!("UNREACHABLE: {}", value);
        Ok(self.label_stack.pick(0)?.unreachable = value)
    }

    fn is_unreachable(&self) -> Result<bool, Error> {
        Ok(self.label_stack.peek(0)?.unreachable)
    }


    fn add_fixup(&mut self, rel_depth: u32, offset: u32) -> Result<(), Error> {
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

    fn fixup(&mut self) -> Result<(), Error> {
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
        // info!("get_drop_keep: type_stack: {} stack_limit: {}", self.type_stack.len(), label.stack_limit);
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


    fn write_fixup_u32(&mut self) -> Result<usize, Error> {
        let pos = self.w.pos();
        self.w.write_u32(FIXUP_OFFSET)?;
        Ok(pos)
    }

    fn apply_fixup_u32(&mut self, fixup: usize) -> Result<u32, Error> {
        let len = self.w.pos() - (fixup + 4);
        self.w.write_u32_at(len as u32, fixup)?;
        Ok(len as u32)
    }
}

impl<'m> Delegate for Loader<'m> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use Event::*;
        // info!("{:08x}: {:?}", self.w.pos(), evt);
        match evt {
            Start { name, version } => {
                self.module.set_name(self.w.copy_str(name));
                self.module.set_version(version);
            },
            SectionStart { s_type, s_beg: _, s_end:_ , s_len: _ } => {
                self.section_fixup = self.w.write_section_start(s_type)?;
            },
            SectionEnd => {
                self.w.write_section_end(self.section_fixup)?;
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
            ImportsStart { c } => {
                self.w.write_u32(c)?;
            },            
            Import { n: _, module, export, desc } => {
                self.w.write_import(module::Import { module, export, desc })?;
            }
            FunctionsStart { c } => {
                self.w.write_u32(c)?;
            },
            Function { n: _, index } => {
                self.w.write_u32(index.0)?;
            },
            TablesStart { c } => {
                self.w.write_u32(c)?;
            },
            Table { n: _, element_type, limits } => {
                self.w.write_table(module::Table { element_type, limits })?
            },
            MemsStart { c } => {
                self.w.write_u32(c)?;
            },
            Mem { n: _, limits } => {
                self.w.write_memory(module::Memory { limits })?;
            },
            GlobalsStart { c } => {
                self.w.write_u32(c)?;
            },
            Global { n: _, t, mutability, init } => {
                self.w.write_global_type(module::GlobalType { type_value: t, mutability })?;
                self.w.write_initializer(init)?;

            }
            ExportsStart { c } => {
                self.w.write_u32(c)?;
            },
            Export { n: _, id, index } => {
                self.w.write_identifier(id)?;                
                self.w.write_u8(index.kind())?;
                self.w.write_u32(index.index())?;
            },
            StartFunction { index } => {
                self.w.write_u32(index.0)?;                
            },
            ElementsStart { c } => {
                self.w.write_u32(c)?;
            },
            Element { n: _, index, offset, data } => {
                self.w.write_u32(index.0)?;
                self.w.write_initializer(offset)?;
                if let Some(data) = data {
                    self.w.write_bytes(data)?;
                }
            },
            CodeStart { c } => {
                info!("{:08x}: Code Start", self.w.pos());
                self.w.write_u32(c)?;
            },
            Body { n, offset: _, size: _, locals: _ } => {
                self.context = Context::from(self.module.function_signature_type(n).unwrap());
                self.body_fixup = self.w.write_code_start()?;
                // self.w.write_u8(locals as u8)?;

                info!("{:08x}: V:{} | func[{}]", self.w.pos(), self.type_stack.len(), n);                
            },
            Local { i: _, n, t } => {
                if !self.cfg.compile { return Ok(()) }

                self.context.add_local(n, t);
                // info!("add_local: {} {}", n, t); 

                // self.w.write_u8(n as u8)?;
                // self.w.write_i8(t as i8)?;
            },
            InstructionsStart => {
                if !self.cfg.compile { return Ok(()) }

                let mut locals_count = 0;
                info!("{:?}", self.context);

                let return_type = self.context.return_type;

                // Push Parameters

                for p in self.context.parameters() {
                    self.type_stack.push_type(TypeValue::from(*p as i8))?;
                    locals_count += 1;
                }

                // Push Locals

                for local in self.context.locals() {
                    self.type_stack.push_type(*local)?;
                    locals_count += 1;
                }                        
                info!("ALLOCA {} @ {:08x}", locals_count, self.w.pos());
                self.w.write_alloca(locals_count as u32)?;

                // Push Stack Entry Label

                self.push_label(0, return_type, FIXUP_OFFSET)?;               
            },
            Instruction(i) => {
                if !self.cfg.compile { return Ok(()) }
                self.dispatch_instruction(i)?;
            },
            InstructionsEnd => {
                if !self.cfg.compile { return Ok(()) }
                info!("{:08x}: V:{} | {} ", self.w.pos(), self.type_stack.len(), "EXIT");
                let return_type = self.context.return_type;

                let drop = self.context.len() as u32;
                let keep = if return_type == VOID { 0 } else { 1 };
                self.type_stack.drop_keep(drop as usize, keep as usize)?;
                self.w.write_drop_keep(drop as u32, keep as u32)?;
        
                for entry in self.fixups.iter() {
                    if let &Some(entry) = entry {   
                        info!("{:?}", entry);
                        panic!("Orphan Fixup: {:?}", entry);
                    }
                }

                self.type_stack.pop_type_expecting(return_type)?;
            },
            BodyEnd => {
                assert!(self.type_stack.len() == 0);
                assert!(self.label_stack.len() == 0);
                // let fixup = self.body_fixup;
                // self.apply_fixup_u32(fixup)?;                                
                self.w.write_code_end(self.body_fixup)?;
                info!("code end: {:08x}", self.w.pos());
            },
            DataSegmentsStart { c } => {
                self.w.write_u32(c)?;
            }
            DataSegment { n: _, index, offset, data } => {
                self.w.write_u32(index.0)?;
                self.w.write_initializer(offset)?;
                self.w.write_bytes(data)?;
            },            
            _ => {},    
        }
        Ok(())
    }
}

impl<'m> Loader<'m> {
    fn dispatch_instruction(&mut self, i: Instruction) -> DelegateResult {
        use opcode::Immediate::*;

        let mut depth = self.label_stack.len();
        if i.op.code == END || i.op.code == ELSE {
            depth -= 1;
        }
        info!("{:08x}: V:{} | {:0width$}{}{:?} {:02x}" , self.w.pos(), self.type_stack.len(), "", i.op.text, i.imm, i.op.code, width=depth);
        self.type_check(&i)?;   

        let op = i.op.code;
        match i.imm {
            None => match op {
                END => {
                //     // w.write_opcode(op)?;
                //     // info!("FIXUP {} 0x{:04x}", self.label_depth(), w.pos());
                    info!("END");
                //     // self.fixup()?;
                //     // self.pop_label()?;
                },
                ELSE => {
                    self.w.write_opcode(BR)?;
                    // self.fixup()?;
                    // let label = self.pop_label()?;
                    // self.push_label(op, label.signature, FIXUP_OFFSET)?;    
                    let pos = self.w.pos();                
                    self.add_fixup(0, pos as u32)?;
                    self.w.write_u32(FIXUP_OFFSET)?;
                },
                RETURN => {
                    let depth = self.type_stack.len() as u32;
                    let return_type = self.context.return_type;
                    info!("RETURN {:?} - depth {}", return_type, depth);
                    if return_type == VOID {
                        self.w.write_drop_keep(depth, 0)?;
                    } else {
                        self.w.write_drop_keep(depth - 1, 1)?;
                    }

                    let return_type = self.context.return_type();
                    self.type_stack.expect_type(return_type)?;
                    // if return_type != VOID {
                    //     self.type_stack.pop_type_expecting(return_type)?;
                    // }
                    self.set_unreachable(true)?;     
                                    
                    self.w.write_opcode(RETURN)?;
                },                
                _ => {
                    self.w.write_opcode(op)?;
                }           
                // _ => {},
            },
            Block { signature: _ } => match op {
                BLOCK => {
                    // self.push_label(op, signature, FIXUP_OFFSET)?;                    
                },
                LOOP => {
                    // let pos = self.w.pos();
                    // self.push_label(op, signature, pos as u32)?;                    
                },
                IF => {
                    // self.push_label(op, signature, FIXUP_OFFSET)?;

                    self.w.write_opcode(INTERP_BR_UNLESS)?;
                    let pos = self.w.pos();
                    self.add_fixup(0, pos as u32)?;
                    self.w.write_u32(FIXUP_OFFSET)?;                    
                },
                _ => unreachable!(),
            },
            Branch { depth } => {
                // let label = self.label_stack.peek(depth as usize)?;
                // let (drop, keep) = self.get_drop_keep(&label)?;
                // self.w.write_drop_keep(drop, keep)?;

                self.w.write_opcode(op)?;
                let pos = self.pos();
                self.add_fixup(depth, pos as u32)?;
                self.w.write_u32(FIXUP_OFFSET)?;                
            },
            BranchTable { count } => {
                // Emits BR_TABLE LEN [DROP OFFSET; LEN] [DROP OFFSET] KEEP

                // Verify top of stack contains the index
                self.type_stack.pop_type_expecting(I32)?;
                
                self.w.write_opcode(op)?;
                self.w.write_u32(count as u32)?;
            },
            BranchTableDepth { n: _, depth } => {
                let label = self.label_stack.peek(depth as usize)?;
                self.type_stack.expect_type(label.signature)?;
                let (drop, keep) = self.get_drop_keep(&label)?;

                self.w.write_u32(drop as u32)?;
                self.w.write_u32(keep as u32)?;
                let pos = self.w.pos();
                self.add_fixup(depth, pos as u32)?;
                self.w.write_u32(FIXUP_OFFSET)?;                
            },
            BranchTableDefault { depth } => {
                let label = self.label_stack.peek(depth as usize)?;
                self.type_stack.expect_type(label.signature)?;

                let (drop, keep) = self.get_drop_keep(&label)?;
                self.w.write_u32(drop as u32)?;
                self.w.write_u32(keep as u32)?;
                let pos = self.w.pos();
                self.add_fixup(depth, pos as u32)?;
                self.w.write_u32(FIXUP_OFFSET)?;                
            },
            Local { index } => {
                // Emits OP DEPTH_TO_LOCAL
                let id = index.0;
                info!("id: {} depth: {} stack: {}", id, depth, self.type_stack.len());
                let depth = (self.type_stack.len() as u32) - id;

                // TODO: Move to Type Check
                if id >= self.context.len() as u32 {
                    return Err(Error::InvalidLocal { id: id })
                }

                let ty = self.context[id as usize];
                match op {
                    GET_LOCAL => self.type_stack.push_type(ty)?,
                    SET_LOCAL => self.type_stack.pop_type_expecting(ty)?,
                    TEE_LOCAL => {
                        self.type_stack.pop_type_expecting(ty)?;
                        self.type_stack.push_type(ty)?;
                    }
                    _ => unreachable!()
                }
                // let depth = self.type_stack.len();
                self.w.write_opcode(op)?;
                self.w.write_u32(depth)?;                
            }
            Global { index } => {
                let id = index.0 as u32;

                // TODO: Move to type_check
                let ty = {
                    let global = if let Some(global) = self.module.global(id) {
                        global
                    } else {
                        return Err(Error::InvalidGlobal { id: id })
                    };
                    global.global_type.type_value
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
            CallIndirect { index } => {
                // Emits OP SIG

                let id = index.0 as u32;                
                let signature = if let Some(signature) = self.module.function_signature_type(id) {
                    signature
                } else {
                    return Err(Error::InvalidFunction { id: id })
                };

                let ret_count = signature.returns.len() as u32;
                if ret_count > 1 {
                    return Err(Error::UnexpectedReturnLength { got: ret_count })
                }
                // Load function index
                self.type_stack.pop_type_expecting(I32)?;
                for p in signature.parameters() {
                    self.type_stack.pop_type_expecting(p)?;
                }
                for r in signature.returns() {
                    self.type_stack.push_type(r)?;
                }                         

                self.w.write_opcode(op)?;                    
                self.w.write_u32(id as u32)?;                
            },
            I32Const { value } => {
                self.w.write_opcode(op)?;
                self.w.write_i32(value)?;
            },
            F32Const { value: _ } => { return Err(Error::Unimplemented) },
            I64Const { value: _ } => { return Err(Error::Unimplemented) },
            F64Const { value: _ } => { return Err(Error::Unimplemented) },
            LoadStore { align, offset } => {
                self.w.write_opcode(op)?;
                self.w.write_u32(align)?;
                self.w.write_u32(offset)?;
            },
            Memory { reserved: _ } => {
                self.w.write_opcode(op)?;
            },
        } 
        Ok(())
    }

    fn type_check(&mut self, i: &Instruction) -> Result<(), Error> {
        let opc = i.op.code;
        match opc {
            BLOCK => {
                if let Immediate::Block { signature } = i.imm {
                    self.push_label(opc, signature, FIXUP_OFFSET)?;
                } else {
                    panic!("Wrong immediate for BLOCK: {:?}", i.imm);
                }
            },
            LOOP => {
                if let Immediate::Block { signature } = i.imm {
                    let pos = self.w.pos();
                    self.push_label(opc, signature, pos as u32)?;
                } else {
                    panic!("Wrong immediate type for LOOP: {:?}", i.imm);
                }
            }
            IF => {
                if let Immediate::Block { signature } = i.imm {
                    self.type_stack.pop_type_expecting(I32)?;
                    self.push_label(opc, signature, FIXUP_OFFSET)?;
                } else {
                    panic!("Wrong immediate type for IF: {:?}", i.imm);                    
                }
            },
            ELSE => {
                // All fixups go to the BR that will be the next opcode to be emitted
                self.fixup()?;
                let label = self.pop_label()?;
                
                self.type_stack.expect_type(label.signature)?;
                if label.signature == VOID {
                    self.type_stack.expect_type_stack_depth(label.stack_limit)?;
                } else {
                    self.type_stack.expect_type_stack_depth(label.stack_limit + 1)?;                    
                }

                // Reset Stack to Label
                while self.type_stack.len() > label.stack_limit as usize {
                    self.type_stack.pop()?;
                }
                if label.signature != VOID {
                    self.type_stack.push(label.signature)?;
                }

                self.push_label(opc, label.signature, FIXUP_OFFSET)?;
            },
            END => {
                // All fixups go to the next instruction
                self.fixup()?;
                let label = self.pop_label()?;

                // IF without ELSE can only have type signature VOID
                if label.opcode == IF && label.signature != VOID {
                    return Err(Error::InvalidIfSignature)
                }

                self.type_stack.expect_type(label.signature)?;
                // Reset Stack to Label
                while self.type_stack.len() > label.stack_limit as usize {
                    self.type_stack.pop()?;
                }
                if label.signature != VOID {
                    self.type_stack.push(label.signature)?;
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
                // let return_type = self.context.return_type();
                // self.type_stack.expect_type(return_type)?;
                // // if return_type != VOID {
                // //     self.type_stack.pop_type_expecting(return_type)?;
                // // }
                // self.set_unreachable(true)?;                
            },
            DROP => {
                self.type_stack.pop_type()?;                
            }
            _ => {
                self.type_stack.pop_type_expecting(i.op.t1)?;
                self.type_stack.pop_type_expecting(i.op.t2)?;
                if i.op.tr != TypeValue::None {
                    self.type_stack.push_type(i.op.tr)?;
                }
            }
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