use {Error, Event, Value, TypeValue, Delegate, DelegateResult};

use opcode::*;
use module;
use module::*;
use module::ModuleWrite;
use typeck::{TypeChecker, LabelType};
use writer::Writer;
use stack::Stack;
use small_vec::SmallVec;

use core::fmt;
use core::ops::Index;

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
    offset: u32,
    fixup_offset: u32,
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Label {{ offset: 0x{:08x}, fixup_offset: 0x{:08x} }}", self.offset, self.fixup_offset)
    }
}

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
            self.parameters[i] = TypeValue::from(*p);
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
            c.set_return(TypeValue::from(other.returns[0]));
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

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context {{ (")?;
        for i in 0..self.parameters_count {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{:?}", self.parameters[i])?;
        }
        write!(f, ") -> ")?;
        if self.return_type != VOID {
            write!(f, "{:?}", self.return_type)?;
        }        
        if self.locals_count > 0 {
            write!(f, "locals[")?;
            for i in 0..self.locals_count {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{:?}", self.locals[i])?;
            }
            write!(f, "]")?;
            
        }
        write!(f, " }}")?;
        Ok(())
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

pub struct Compiler<'m> {
    cfg: Config,
    w: Writer<'m>,
    module: Module<'m>,
    label_stack: Stack<'m, Label>,
    type_checker: TypeChecker<'m>,
    functions: SmallVec<'m, u32>,
    table: SmallVec<'m, u32>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
    section_fixup: usize,
    body_fixup: usize,
    context: Context,
}

impl<'m> Compiler<'m> {
    pub fn new(module_buf: &'m mut [u8]) -> Self {
        Compiler::new_with_config(Config::default(), module_buf)
    }
    pub fn new_with_config(cfg: Config, code_buf: &'m mut [u8]) -> Self {
        let mut w = Writer::new(code_buf);
        let module = module::Module::new();

        // These should be not be allocated from module storage
        let label_stack = w.alloc_stack(16);        
        let type_checker = TypeChecker::new(w.alloc_stack(16), w.alloc_stack(16));

        let functions = w.alloc_smallvec(16);
        let table = w.alloc_smallvec(16);

        // TODO: Break out into separate struct
        let fixups = [None; 256];
        let fixups_pos = 0;

        let section_fixup = 0;
        let body_fixup = 0;
        let context = Context::default();
        Compiler { 
            cfg,
            w, 
            module, 
            label_stack,
            type_checker,
            functions,
            table,
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

    fn push_label(&mut self, offset: u32) -> Result<(), Error> {
        self.push_label_fixup(offset, 0x0000_0000)
    }

    fn push_label_fixup(&mut self, offset: u32, fixup_offset: u32) -> Result<(), Error> {
        let label = Label {
            offset,
            fixup_offset,
        };
        // info!("-- label: {} <= {:?}", self.label_stack.len(), label);
        Ok(self.label_stack.push(label)?)
    }

    fn top_label(&mut self) -> Result<Label, Error> {
        Ok(self.label_stack.peek(0)?)
    }

    fn top_label_ref(&mut self) -> Result<&mut Label, Error> {
        Ok(self.label_stack.pick(0)?)
    }

    fn pop_label(&mut self) -> Result<Label, Error> {
        let depth = self.label_stack.len();
        let label = self.label_stack.pop()?;
        info!("-- label: {} => {:?}", depth, label);
        Ok(label)
    }

    fn label_depth(&self) -> u32 {
        self.label_stack.len() as u32
    }

    fn peek_label(&self, offset: usize) -> Result<Label, Error> {
        Ok(self.label_stack.peek(offset)?)
    }

    fn add_fixup(&mut self, rel_depth: u32, offset: u32) -> Result<(), Error> {
        let depth = self.label_depth() - rel_depth;
        let fixup = Fixup { depth: depth, offset: offset };
        info!("add_fixup: {:?}", fixup);
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
        info!("fixup: {} -> 0x{:08x}", depth, offset);
        for entry in self.fixups.iter_mut() {
            let del = if let &mut Some(entry) = entry {
                if entry.depth == depth {
                    info!(" {:?}", entry);
                    self.w.write_u32_at(offset as u32, entry.offset as usize)?;
                    true
                } else {
                    info!(" ! {} 0x{:04x}", entry.depth, entry.offset);                    
                    false
                }
            } else {
                false
            };
            if del {
                *entry = None;
            }
        }
        info!("fixup done");
        Ok(())
    }

    fn translate_local_index(&self, local_index: u32) -> Result<u32, Error> {
        Ok({
            (self.type_checker.type_stack_size() + self.context.len() - local_index as usize) as u32
        })
    }

    fn get_br_drop_keep_count(&mut self, depth: usize) -> Result<(u32, u32), Error> {        
        Ok({
            let label = self.type_checker.get_label(depth)?;
            let keep = if label.label_type != LabelType::Loop {
                if label.signature != VOID { 1 } else { 0 }
            } else { 
                0 
            };
            let drop = if self.type_checker.is_unreachable()? {
                0
            } else {
                self.type_checker.type_stack_size() - label.stack_limit - keep
            };
            info!("get_br_drop_keep_count() -> ({}, {})", drop, keep);
            (drop as u32, keep as u32)
        })
    }

    fn get_return_drop_keep_count(&mut self) -> Result<(u32, u32), Error> {
        info!("get_return_drop_keep_count()");
        Ok({
            let len = self.label_stack.len();
            let (drop, keep) = self.get_br_drop_keep_count(len - 1)?;            
            let (drop, keep) = (drop + (self.context.len() as u32), keep);
            info!("  -> ({}, {})", drop, keep);
            (drop, keep)
        })
    }

    fn write_br_offset(&mut self, depth: u32, offset: u32) -> Result<(), Error> {
        info!("write_br_offset({}, {:08x}) @ {:08x}", depth, offset, self.w.pos());
        Ok({
            if offset == FIXUP_OFFSET {
                let pos = self.w.pos();
                self.add_fixup(depth, pos as u32)?;
            }
            self.w.write_u32(offset)?;
        })
    }

    fn write_br_table_offset(&mut self, depth: u32) -> Result<(), Error> {
        info!("write_br_table_offset({}) @ {:08x}", depth, self.w.pos());
        Ok({
            let (drop, keep) = self.get_br_drop_keep_count(depth as usize)?;
            let label = self.peek_label(depth as usize)?;
            self.write_br_offset(depth, label.offset)?;
            self.w.write_u32(drop)?;
            self.w.write_u32(keep)?;

//   CHECK_RESULT(GetBrDropKeepCount(depth, &drop_count, &keep_count));
//   CHECK_RESULT(EmitBrOffset(depth, GetLabel(depth)->offset));
//   CHECK_RESULT(EmitI32(drop_count));
//   CHECK_RESULT(EmitI8(keep_count));            
        })
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

impl<'m> Compiler<'m> {
    pub fn compile(&mut self, _m: &::inplace::Module) -> Result<&'m [u8], Error> {

        Ok(self.w.split_mut())
    }
}

impl<'m> Delegate for Compiler<'m> {
    fn dispatch(&mut self, evt: Event) -> DelegateResult {
        use Event::*;
        // info!("{:08x}: {:?}", self.w.pos(), evt);
        match evt {
            Start { name, version } => {
                self.module.set_name(self.w.copy_str(name));
                self.module.set_version(version);
            },
            SectionStart { s_type, s_beg: _, s_end:_ , s_len: _, s_count: _ } => {
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
                match desc {
                    ImportDesc::Type(index) => {
                        self.functions.push(index);
                    },
                    _ => {},
                }
                self.w.write_import(module::Import { module, export, desc })?;
            }
            FunctionsStart { c } => {
                self.w.write_u32(c)?;
            },
            Function { n: _, index } => {
                self.functions.push(index.0);
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
            Export { n: _, id, desc } => {
                self.w.write_identifier(id)?;    
            // 0x00 => ExportDesc::Function(index),
            // 0x01 => ExportDesc::Table(index),
            // 0x02 => ExportDesc::Memory(index),
            // 0x03 => ExportDesc::Global(index),                
                let (kind, index) = match desc {
                    ExportDesc::Function(i) => (0x00, i),
                    ExportDesc::Table(i) => (0x01, i),
                    ExportDesc::Memory(i) => (0x02, i),
                    ExportDesc::Global(i) => (0x03, i),
                };
                self.w.write_u8(kind)?;
                self.w.write_u32(index)?;
            },
            StartFunction { index } => {
                self.w.write_u32(index.0)?;                
            },
            ElementsStart { c } => {
                self.w.write_u32(c)?;
            },
            Element { n: _, index, offset, data } => {
                info!("Initializing Table {}", index.0);
                let Value(elt_offset) = offset.value()?;

                self.w.write_u32(index.0)?;
                self.w.write_initializer(offset)?;
                if let Some(data) = data {
                    self.w.write_bytes(data)?;

                    let mut i = 0;
                    let mut o = elt_offset as usize;
                    while i < data.len() {
                        let d = data[i] as u32;
                        info!("   {:08x}: {:08x}", o, d);
                        self.table[o] = d;
                        o += 1;
                        i += 1;
                    }            
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
                info!("{:08x}: V:{} | func[{}] {:?}", self.w.pos(), self.type_checker.type_stack_size(), n, self.context);

                self.type_checker.begin_function(self.context.return_type)?;
                self.push_label(FIXUP_OFFSET)?;
            },
            Local { i: _, n, t } => {
                if !self.cfg.compile { return Ok(()) }

                info!("add_local: {} {}", n, t); 
                self.context.add_local(n, t);
                

                // self.w.write_u8(n as u8)?;
                // self.w.write_i8(t as i8)?;
            },
            InstructionsStart => {
                if !self.cfg.compile { return Ok(()) }

                self.w.write_alloca(self.context.locals_count as u32)?;                

            },
            Instruction(i) => {
                if !self.cfg.compile { return Ok(()) }
                // assert_eq!(self.depth, self.type_stack.len());
                self.dispatch_instruction(i)?;
                // assert_eq!(self.depth, self.type_stack.len());
            },
            InstructionsEnd => {
                if !self.cfg.compile { return Ok(()) }
                info!("{:08x}: L: {} V:{} | {} ", self.w.pos(), self.label_stack.len(), self.type_checker.type_stack_size(), "EXIT");

                //   CHECK_RESULT(GetReturnDropKeepCount(&drop_count, &keep_count));
                //   CHECK_RESULT(typechecker_.EndFunction());
                //   CHECK_RESULT(EmitDropKeep(drop_count, keep_count));
                //   CHECK_RESULT(EmitOpcode(Opcode::Return));
                //   PopLabel();

                self.fixup()?;
                let (drop, keep) = self.get_return_drop_keep_count()?;
                self.type_checker.end_function()?;
                self.w.write_drop_keep(drop, keep)?;                                    
                self.w.write_opcode(RETURN)?;
                self.pop_label()?;
        
                for entry in self.fixups.iter() {
                    if let &Some(entry) = entry {   
                        panic!("Orphan Fixup: {:?}", entry);
                    }
                }
            },
            BodyEnd => {
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

impl<'m> Compiler<'m> {
    fn dispatch_instruction(&mut self, i: Instruction) -> DelegateResult {
        use opcode::Immediate::*;

        {
            let mut indent = self.label_stack.len();
            if i.op.code == END || i.op.code == ELSE {
                indent -= 1;
            }
            info!("{:08x}: L: {} V:{} | {:0width$}{}{:?}" , self.w.pos(), self.label_stack.len(), self.type_checker.type_stack_size(),  "", i.op.text, i.imm, width=indent);
        }

        let op = i.op.code;
        match i.imm {
            None => match op {
                SELECT => {
                    self.type_checker.on_select()?;
                    self.w.write_opcode(op)?;
                },
                DROP => {
                    self.type_checker.on_drop()?;
                    self.w.write_opcode(op)?;
                },                
                END => {
                    info!("END");
                    let ty_label = self.type_checker.get_label(0)?;
                    let label_type = ty_label.label_type;
                    self.type_checker.on_end()?;
                    if label_type == LabelType::If || label_type == LabelType::Else {
                        let label = self.top_label()?;
                        let pos = self.w.pos();
                        info!("fixup_offset: {:08x} at {:08x}", pos, label.fixup_offset);
                        self.w.write_u32_at(pos as u32, label.fixup_offset as usize)?;                        
                    }
                    info!("FIXUP");
                    self.fixup()?;
                    info!("POP_LABEL");
                    self.pop_label()?;
                    info!("end done");

                    //   TypeChecker::Label* label;
                    //   CHECK_RESULT(typechecker_.GetLabel(0, &label));
                    //   LabelType label_type = label->label_type;
                    //   if (label_type == LabelType::If || label_type == LabelType::Else) {
                    //     CHECK_RESULT(EmitI32At(TopLabel()->fixup_offset, GetIstreamOffset()));
                    //   }
                    //   FixupTopLabel();
                    //   PopLabel();


                },                
                ELSE => {
                //   CHECK_RESULT(typechecker_.OnElse());
                //   Label* label = TopLabel();
                //   IstreamOffset fixup_cond_offset = label->fixup_offset;
                //   CHECK_RESULT(EmitOpcode(Opcode::Br));
                //   label->fixup_offset = GetIstreamOffset();
                //   CHECK_RESULT(EmitI32(kInvalidIstreamOffset));
                //   CHECK_RESULT(EmitI32At(fixup_cond_offset, GetIstreamOffset()));                    
                    self.type_checker.on_else()?;
                    let mut label = self.top_label()?;

                    // Get offset of BR_UNLESS OFFSET
                    let fixup_cond_offset = label.fixup_offset;

                    // Write BR to end of block
                    self.w.write_opcode(BR)?;
                    // Write BR OFFSET fixup 
                    let br_offset = self.w.pos() as u32;
                    {
                        let mut label = self.top_label_ref()?;
                        label.fixup_offset = br_offset;
                    }
                    self.w.write_u32(FIXUP_OFFSET)?;

                    // Fixup BR_UNLESS OFFSET
                    let br_pos = self.w.pos();
                    info!("fixup_offset: {:08x} at {:08x}", br_pos, fixup_cond_offset);
                    self.w.write_u32_at(br_pos as u32, fixup_cond_offset as usize)?;
                },
                RETURN => {                    
                    // Index drop_count, keep_count;
                    // CHECK_RESULT(GetReturnDropKeepCount(&drop_count, &keep_count));
                    // CHECK_RESULT(typechecker_.OnReturn());
                    // CHECK_RESULT(EmitDropKeep(drop_count, keep_count));
                    // CHECK_RESULT(EmitOpcode(Opcode::Return));                    

                    let (drop, keep) = self.get_return_drop_keep_count()?;
                    self.type_checker.on_return()?;
                    self.w.write_drop_keep(drop, keep)?;                                    
                    self.w.write_opcode(RETURN)?;
                },                
                _ => {
                    info!("{:?} {}", i.op, i.op.is_binop());
                    if i.op.is_binop() {
                        self.type_checker.on_binary(i.op)?;
                        self.w.write_opcode(op)?;
                    } else if i.op.is_unop() {
                        self.type_checker.on_unary(i.op)?;
                        self.w.write_opcode(op)?;
                    } else {
                        panic!("{} not implemented", i.op.text);
                    }
                    
                }           
                // _ => {},
            },
            Block { signature: sig } => match op {                
                BLOCK => {
                    self.type_checker.on_block(sig)?;
                    self.push_label(FIXUP_OFFSET)?;                    
                },
                LOOP => {
                    self.type_checker.on_loop(sig)?;
                    let pos = self.w.pos();
                    self.push_label(pos as u32)?;                    
                },
                IF => {
                    // CHECK_RESULT(typechecker_.OnIf(&sig));
                    // CHECK_RESULT(EmitOpcode(Opcode::InterpBrUnless));
                    // IstreamOffset fixup_offset = GetIstreamOffset();
                    // CHECK_RESULT(EmitI32(kInvalidIstreamOffset));
                    // PushLabel(kInvalidIstreamOffset, fixup_offset);
                                        
                    self.type_checker.on_if(sig)?;
                    self.w.write_opcode(INTERP_BR_UNLESS)?;                    
                    let pos = self.w.pos();
                    // push label with fixup pointer to BR_UNLESS offset
                    self.w.write_u32(FIXUP_OFFSET)?;                    
                    self.push_label_fixup(FIXUP_OFFSET, pos as u32)?; 
                },
                _ => unreachable!(),
            },
            Branch { depth } => match op {
                BR => {
                    let (drop, keep) = self.get_br_drop_keep_count(depth as usize)?;
                    self.type_checker.on_br(depth as usize)?;
                    self.w.write_drop_keep(drop, keep)?;

                    self.w.write_opcode(op)?;
                    let pos = self.w.pos();
                    self.add_fixup(depth as u32, pos as u32)?;
                    self.w.write_u32(FIXUP_OFFSET)?;    

                    // CHECK_RESULT(GetBrDropKeepCount(depth, &drop_count, &keep_count));
                    // CHECK_RESULT(typechecker_.OnBr(depth));
                    // CHECK_RESULT(EmitBr(depth, drop_count, keep_count));                    
                },
                BR_IF => {
                    self.type_checker.on_br_if(depth as usize)?;
                    let (drop, keep) = self.get_br_drop_keep_count(depth as usize)?;

                    self.w.write_opcode(INTERP_BR_UNLESS)?;
                    let fixup_br_offset = self.w.pos();
                    self.w.write_u32(FIXUP_OFFSET)?;    
                    self.w.write_drop_keep(drop, keep)?;

                    self.w.write_opcode(BR)?;
                    let pos = self.w.pos();
                    self.add_fixup(depth as u32, pos as u32)?;
                    self.w.write_u32(FIXUP_OFFSET)?;    

                    let pos = self.w.pos();
                    self.w.write_u32_at(pos as u32, fixup_br_offset)?;
                    
                    //   CHECK_RESULT(typechecker_.OnBrIf(depth));
                    //   CHECK_RESULT(GetBrDropKeepCount(depth, &drop_count, &keep_count));
                    //   /* flip the br_if so if <cond> is true it can drop values from the stack */
                    //   CHECK_RESULT(EmitOpcode(Opcode::InterpBrUnless));
                    //   IstreamOffset fixup_br_offset = GetIstreamOffset();
                    //   CHECK_RESULT(EmitI32(kInvalidIstreamOffset));
                    //   CHECK_RESULT(EmitBr(depth, drop_count, keep_count));
                    //   CHECK_RESULT(EmitI32At(fixup_br_offset, GetIstreamOffset()));                    
                },  
                _ => unimplemented!(),              
                // let label = self.label_stack.peek(depth as usize)?;
                // let (drop, keep) = self.get_drop_keep(&label)?;
                // self.w.write_drop_keep(drop, keep)?;

                // self.w.write_opcode(op)?;
                // let pos = self.w.pos();
                // self.add_fixup(depth, pos as u32)?;
                // self.w.write_u32(FIXUP_OFFSET)?;                
            },
            BranchTable { table } => {
                // BR_TABLE COUNT:u32 TABLE_OFFSET:u32
                // INTERP_DATA SIZE:u32
                // [OFFSET:u32 DROP:u32 KEEP:u32]
                // OFFSET:u32 DROP:u32 KEEP:u32

                let count = table.len() as u32;

                self.type_checker.begin_br_table()?;
                self.w.write_opcode(BR_TABLE)?;
                self.w.write_u32(count as u32)?;

                // Write offset of branch table
                let table_pos = self.w.pos();
                self.w.write_u32(FIXUP_OFFSET)?;
                
                // Write INTERP_DATA + SIZE
                self.w.write_opcode(INTERP_DATA)?;
                self.w.write_u32((count + 1) * BR_TABLE_ENTRY_SIZE)?;

                // Fixup branch table offset
                let pos = self.w.pos();
                self.w.write_u32_at(pos as u32, table_pos)?;

                // Branch Table Starts Here

                for depth in table.iter() {
                    self.type_checker.on_br_table_target(*depth as usize)?;
                    self.write_br_table_offset(*depth as u32)?;
                }

                self.type_checker.end_br_table()?;
                info!("branch table default done");   
            },
            BranchTableStart { count } => {
                // BR_TABLE COUNT:u32 TABLE_OFFSET:u32
                // INTERP_DATA SIZE:u32
                // [OFFSET:u32 DROP:u32 KEEP:u32]
                // OFFSET:u32 DROP:u32 KEEP:u32

                self.type_checker.begin_br_table()?;
                self.w.write_opcode(BR_TABLE)?;
                self.w.write_u32(count as u32)?;

                // Write offset of branch table
                let table_pos = self.w.pos();
                self.w.write_u32(FIXUP_OFFSET)?;
                
                // Write INTERP_DATA + SIZE
                self.w.write_opcode(INTERP_DATA)?;
                self.w.write_u32((count + 1) * BR_TABLE_ENTRY_SIZE)?;

                // Fixup branch table offset
                let pos = self.w.pos();
                self.w.write_u32_at(pos as u32, table_pos)?;

                // Branch Table Starts Here            
            },
            BranchTableDepth { n: _, depth } => {
                self.type_checker.on_br_table_target(depth as usize)?;
                self.write_br_table_offset(depth as u32)?;
            },
            BranchTableDefault { depth } => {
                info!("branch table default");
                self.type_checker.on_br_table_target(depth as usize)?;
                self.write_br_table_offset(depth as u32)?;

                self.type_checker.end_br_table()?;              
                info!("branch table default done");
            },
            Local { index } => {
                // Emits OP DEPTH_TO_LOCAL
                let id = index.0;
                let ty = self.context[id as usize];
                let local_id = match op {
                    GET_LOCAL => {                        
                        let local_id = self.translate_local_index(id)?;
                        self.type_checker.on_get_local(ty)?;
                        local_id
                    },
                    SET_LOCAL => {
                        self.type_checker.on_set_local(ty)?;
                        self.translate_local_index(id)?
                    },
                    TEE_LOCAL => {
                        self.type_checker.on_tee_local(ty)?;
                        self.translate_local_index(id)?
                    }
                    _ => unreachable!()
                };
                info!("-- local_id: {}", local_id);
                self.w.write_opcode(op)?;
                self.w.write_u32(local_id)?;                

            }
            Global { index } => {
                match op {                    
                    GET_GLOBAL => {

                        if let Some(global) = self.module.global(index.0) {
                            let global_type = global.global_type;
                            self.type_checker.on_get_global(global_type.type_value)?;
                            self.w.write_opcode(GET_GLOBAL)?    ;
                            self.w.write_u32(index.0)?;
                        } else {
                            return Err(Error::InvalidGlobal{ id: index.0});
                        }

                        //   CHECK_RESULT(CheckGlobal(global_index));
                        //   Type type = GetGlobalTypeByModuleIndex(global_index);
                        //   CHECK_RESULT(typechecker_.OnGetGlobal(type));
                        //   CHECK_RESULT(EmitOpcode(Opcode::GetGlobal));
                        //   CHECK_RESULT(EmitI32(TranslateGlobalIndexToEnv(global_index)));   
                    },
                    SET_GLOBAL => {
                        //   CHECK_RESULT(CheckGlobal(global_index));
                        //   Global* global = GetGlobalByModuleIndex(global_index);
                        //   if (!global->mutable_) {
                        //     PrintError("can't set_global on immutable global at index %" PRIindex ".",
                        //                global_index);
                        //     return wabt::Result::Error;
                        //   }
                        //   CHECK_RESULT(typechecker_.OnSetGlobal(global->typed_value.type));
                        //   CHECK_RESULT(EmitOpcode(Opcode::SetGlobal));
                        //   CHECK_RESULT(EmitI32(TranslateGlobalIndexToEnv(global_index)));                        
                        if let Some(global) = self.module.global(index.0) {
                            let global_type = global.global_type;
                            if global_type.mutability != 0 {
                                return Err(Error::InvalidGlobal { id: index.0 });
                            }
                            self.type_checker.on_set_global(global_type.type_value)?;
                            self.w.write_opcode(SET_GLOBAL)?    ;
                            self.w.write_u32(index.0)?;
                        } else {
                            return Err(Error::InvalidGlobal{ id: index.0});
                        }                        

                    },
                    _ => unimplemented!(),
                }              
            },
            Call { index } => {
                let id = index.0 as u32;
                info!("CALL {}", id);
                let signature = if let Some(signature) = self.module.function_signature_type(id) {
                    signature
                } else {
                    return Err(Error::InvalidFunction { id: id })
                };
                let (parameters, returns) = (signature.parameters, signature.returns);
                if returns.len() > 1 {
                    return Err(Error::UnexpectedReturnLength { got: returns.len() as u32})
                }

                let mut p_arr = [TypeValue::Any; 16];
                let mut r_arr = [TypeValue::Any; 1];
                for i in 0..parameters.len() {
                    p_arr[i] = TypeValue::from(parameters[i]);
                }
                for i in 0..returns.len() {
                    r_arr[i] = TypeValue::from(returns[i]);
                }
                let p_slice = &p_arr[..parameters.len()];
                let r_slice = &r_arr[..returns.len()];

                self.type_checker.on_call(p_slice, r_slice)?;


                self.w.write_opcode(op)?;
                self.w.write_u32(id as u32)?;
            },
            CallIndirect { index, reserved: _ } => {
                //   if (module_->table_index == kInvalidIndex) {
                //     PrintError("found call_indirect operator, but no table");
                //     return wabt::Result::Error;
                //   }
                //   FuncSignature* sig = GetSignatureByModuleIndex(sig_index);
                //   CHECK_RESULT(
                //       typechecker_.OnCallIndirect(&sig->param_types, &sig->result_types));

                //   CHECK_RESULT(EmitOpcode(Opcode::CallIndirect));
                //   CHECK_RESULT(EmitI32(module_->table_index));
                //   CHECK_RESULT(EmitI32(TranslateSigIndexToEnv(sig_index)));
                info!("CALL_INDIRECT: {}", index.0);
                if let Some(sig_type) = self.module.signature_type(index.0) {
                    let (parameters, returns) = (sig_type.parameters, sig_type.returns);

                    let mut p_arr = [TypeValue::Any; 16];
                    let mut r_arr = [TypeValue::Any; 1];
                    for i in 0..parameters.len() {
                        p_arr[i] = TypeValue::from(parameters[i]);
                    }
                    for i in 0..returns.len() {
                        r_arr[i] = TypeValue::from(returns[i]);
                    }
                    let p_slice = &p_arr[..parameters.len()];
                    let r_slice = &r_arr[..returns.len()];
                    info!("  => {:?} => {:?}", p_slice, r_slice);
                    self.type_checker.on_call_indirect(p_slice, r_slice)?;

                    self.w.write_opcode(CALL_INDIRECT)?;
                    self.w.write_u32(index.0)?;                    
                }            
            },
            I32Const { value } => {
                self.type_checker.on_const(I32)?;
                self.w.write_opcode(op)?;
                self.w.write_i32(value)?;
            },
            F32Const { value: _ } => { return Err(Error::Unimplemented) },
            I64Const { value: _ } => { return Err(Error::Unimplemented) },
            F64Const { value: _ } => { return Err(Error::Unimplemented) },
            LoadStore { align, offset } => {
                match op {
                    I32_LOAD | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U => {
                        // CHECK_RESULT(CheckHasMemory(opcode));
                        // CHECK_RESULT(CheckAlign(alignment_log2, opcode.GetMemorySize()));
                        // CHECK_RESULT(typechecker_.OnLoad(opcode));
                        // CHECK_RESULT(EmitOpcode(opcode));
                        // CHECK_RESULT(EmitI32(module_->memory_index));
                        // CHECK_RESULT(EmitI32(offset));

                        self.type_checker.on_load(i.op)?;
                    },
                    I32_STORE | I32_STORE8 | I32_STORE16 => {
                        //   CHECK_RESULT(CheckHasMemory(opcode));
                        //   CHECK_RESULT(CheckAlign(alignment_log2, opcode.GetMemorySize()));
                        //   CHECK_RESULT(typechecker_.OnStore(opcode));
                        //   CHECK_RESULT(EmitOpcode(opcode));
                        //   CHECK_RESULT(EmitI32(module_->memory_index));
                        //   CHECK_RESULT(EmitI32(offset));
                        self.type_checker.on_store(i.op)?;
                    },
                    _ => unimplemented!(),
                }
                self.w.write_opcode(op)?;
                self.w.write_u32(align)?;
                self.w.write_u32(offset)?;
            },
            Memory { reserved: _ } => {
                match op {
                    MEM_SIZE => {
                        self.type_checker.on_current_memory()?;
                    },
                    MEM_GROW => {
                        self.type_checker.on_grow_memory(i.op)?;
                    },
                    _ => unimplemented!()
                }
                self.w.write_opcode(op)?;
            },
        } 
        Ok(())
    }
}
