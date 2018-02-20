#![allow(dead_code)]
use error::Error;
use types::*;
use opcode::*;
use module::*;

use module_inst::*;
use typeck::{TypeChecker, LabelType};
use cursor::Cursor;
use writer::Writer;
use stack::Stack;

use core::fmt;
use core::ops::{Range, Index};

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
    parameters: [TypeValue; 64],
    parameters_count: usize,
    locals: [TypeValue; 64],
    locals_count: usize,
    return_type: TypeValue,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            parameters: [VOID; 64],
            parameters_count: 0,
            locals: [VOID; 64],
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

    fn add_parameter(&mut self, p: TypeValue) {
        self.parameters[self.parameters_count] = p;
        self.parameters_count += 1;
    }

    fn set_parameters(&mut self, parameters: &[TypeValue]) {
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

impl<'t> From<Signature<'t>> for Context {
    fn from(other: Signature<'t>) -> Self {
        info!("{}", other);
        let mut c = Context::default();
        for p in other.parameters() {
            c.add_parameter(*p);
        }
        for r in other.returns() {
            info!("add return: {}", r);
            c.set_return(*r);
            info!("return: {}", c.return_type());
            break;
        }
        info!("{:?}", c);
        c
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
            write!(f, "{}", self.parameters[i])?;
        }
        write!(f, ") -> ")?;
        if self.return_type != VOID {
            write!(f, "{}", self.return_type)?;
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

pub struct CompiledCode<'a> {
    buf: &'a [u8]
}

impl<'a> CompiledCode<'a> {
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn body_count(&self) -> usize {
        Cursor::new(self.buf).read_u32() as usize
    }

    pub fn body_range(&self, index: usize) -> Range<usize> {
        info!("body_range({})", index);
        let mut cur = Cursor::new(self.buf);
        assert!(index < cur.read_u32() as usize);
        cur.advance(index * 8);
        let body_beg = cur.read_u32() as usize;
        let body_end = cur.read_u32() as usize;
        info!("{:08x} to {:08x}", body_beg, body_end);
        body_beg .. body_end
    }

    pub fn iter(&self) -> RangeIter {
        RangeIter { code: self, count: self.body_count(), index: 0 }
    }
}

pub struct RangeIter<'a> {
    code: &'a CompiledCode<'a>,
    count: usize,
    index: usize,
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.index < self.count {
            let index = self.index;
            self.index += 1;

            Some(self.code.body_range(index))
        } else {
            None
        }
    }
}

impl<'a> AsRef<[u8]> for CompiledCode<'a> {
    fn as_ref(&self) -> &[u8] {
        self.buf
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

pub struct Compiler<'c> {
    cfg: Config,
    label_stack: Stack<'c, Label>,
    type_checker: TypeChecker<'c>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
    section_fixup: usize,
    body_fixup: usize,
    context: Context,
}

impl<'c> Compiler<'c> {
    pub fn new(buf: &'c mut [u8]) -> Self {
        Compiler::new_with_config(buf, Config::default() )
    }
    pub fn new_with_config(buf: &'c mut [u8], cfg: Config, ) -> Self {
        let mut w = Writer::new(buf);

        let label_stack = w.alloc_stack(16);        
        let type_checker = TypeChecker::new(w.alloc_stack(16), w.alloc_stack(16));

        // TODO: Break out into separate struct
        let fixups = [None; 256];
        let fixups_pos = 0;

        let section_fixup = 0;
        let body_fixup = 0;
        let context = Context::default();
        Compiler { 
            cfg,
            label_stack,
            type_checker,
            fixups, 
            fixups_pos, 
            section_fixup, 
            body_fixup,
            context,
        }
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

    fn fixup(&mut self, w: &mut Writer) -> Result<(), Error> {
        let depth = self.label_depth();        
        let offset = self.peek_label(0)?.offset;
        let offset = if offset == FIXUP_OFFSET { w.pos() } else { offset as usize};
        info!("fixup: {} -> 0x{:08x}", depth, offset);
        for entry in self.fixups.iter_mut() {
            let del = if let &mut Some(entry) = entry {
                if entry.depth == depth {
                    info!(" {:?}", entry);
                    w.write_u32_at(offset as u32, entry.offset as usize)?;
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

    fn write_br_offset(&mut self, w: &mut Writer, depth: u32, offset: u32) -> Result<(), Error> {
        info!("write_br_offset({}, {:08x}) @ {:08x}", depth, offset, w.pos());
        Ok({
            if offset == FIXUP_OFFSET {
                let pos = w.pos();
                self.add_fixup(depth, pos as u32)?;
            }
            w.write_u32(offset)?;
        })
    }

    fn write_br_table_offset(&mut self, w: &mut Writer, depth: u32) -> Result<(), Error> {
        info!("write_br_table_offset({}) @ {:08x}", depth, w.pos());
        Ok({
            let (drop, keep) = self.get_br_drop_keep_count(depth as usize)?;
            let label = self.peek_label(depth as usize)?;
            self.write_br_offset(w, depth, label.offset)?;
            w.write_u32(drop)?;
            w.write_u32(keep)?;

//   CHECK_RESULT(GetBrDropKeepCount(depth, &drop_count, &keep_count));
//   CHECK_RESULT(EmitBrOffset(depth, GetLabel(depth)->offset));
//   CHECK_RESULT(EmitI32(drop_count));
//   CHECK_RESULT(EmitI8(keep_count));            
        })
    }

    fn write_fixup_u32(&mut self, w: &mut Writer) -> Result<usize, Error> {
        let pos = w.pos();
        w.write_u32(FIXUP_OFFSET)?;
        Ok(pos)
    }

    fn apply_fixup_u32(&mut self, w: &mut Writer, fixup: usize) -> Result<u32, Error> {
        let len = w.pos() - (fixup + 4);
        w.write_u32_at(len as u32, fixup)?;
        Ok(len as u32)
    }
}

impl<'c> Compiler<'c> {
    pub fn compile<'buf>(&mut self, code_buf: &'buf mut [u8], 
        types: &[Type],
        functions: &[FuncInst], 
        globals: &[GlobalInst],        
        m: &Module        
    ) -> Result<(&'buf mut [u8], CompiledCode<'buf>), Error> {
        let mut w = Writer::new(code_buf);

        let code_section = m.code_section().ok_or_else(|| Error::MissingSection { id: SectionType::Code })?;

        // Write Index

        w.write_u32(0)?;

        let mut n = 0;
        for _ in code_section.iter().enumerate() {
            w.write_u32(0)?; // Offset
            w.write_u32(0)?; // Size
            n += 1;
        }
        w.write_u32_at(n, 0)?;

        info!("{:08x}: Code Start", w.pos());
        
        for (n, body) in code_section.iter().enumerate() {
            info!("--- Code Item {} ---", n);
            let body_beg = w.pos() as u32;
            self.context = Context::from(m.function_signature_type(n).unwrap());

            for local in body.locals() {
                self.context.add_local(local.n, local.t);
            }                  

            info!("{:08x}: V:{} | func[{}] {:?}", w.pos(), self.type_checker.type_stack_size(), n, self.context);  

            self.compile_body(&mut w, types, functions, globals, &body)?;
            let body_end = w.pos() as u32;
            info!("body beg: {:08x}", body_beg);
            info!("body end: {:08x}", body_end);
            w.write_u32_at(body_beg, 4 + n * 8)?;
            w.write_u32_at(body_end, 4 + n * 8 + 4)?;
            info!("--- Code Item {} Done ---", n);
        }

        let buf = w.split_mut();
        let rest = w.into_slice();

        Ok((rest, CompiledCode { buf: buf }))
    }

    pub fn compile_body<'buf>(
        &mut self, 
        w: &mut Writer<'buf>, 
        types: &[Type],
        functions: &[FuncInst], 
        globals: &[GlobalInst],            
        body: &Body
    ) -> Result<(), Error> {
        // self.body_fixup = w.write_code_start()?;

        self.type_checker.begin_function(self.context.return_type)?;
        self.push_label(FIXUP_OFFSET)?;


        // InstructionsStart

        w.write_alloca(self.context.locals_count as u32)?;                

        for i in body.iter() {
            // Instruction
            // if !(i.range.end == body.range.end && i.opcode == END) {
                self.compile_instruction(w, types, functions, globals, body, i)?;
            // }
        }
        // InstructionsEnd
        info!("{:08x}: L: {} V:{} | {} ", w.pos(), self.label_stack.len(), self.type_checker.type_stack_size(), "EXIT");

        //   CHECK_RESULT(GetReturnDropKeepCount(&drop_count, &keep_count));
        //   CHECK_RESULT(typechecker_.EndFunction());
        //   CHECK_RESULT(EmitDropKeep(drop_count, keep_count));
        //   CHECK_RESULT(EmitOpcode(Opcode::Return));
        //   PopLabel();

        self.fixup(w)?;
        let (drop, keep) = self.get_return_drop_keep_count()?;
        self.type_checker.end_function()?;
        w.write_drop_keep(drop, keep)?;                                    
        w.write_opcode(RETURN)?;
        self.pop_label()?;

        for entry in self.fixups.iter() {
            if let &Some(entry) = entry {   
                panic!("Orphan Fixup: {:?}", entry);
            }
        }        

        // BodyEnd
        // w.write_code_end(self.body_fixup)?;
        // info!("code end: {:08x}", w.pos());

        Ok(())
    }

    fn compile_instruction<'w>(
        &mut self, 
        w: &mut Writer<'w>, 
        types: &[Type],
        functions: &[FuncInst], 
        globals: &[GlobalInst],        
        body: &Body, 
        i: Instr
    ) -> Result<(), Error> {
        use opcode::Immediate::*;
        use core::convert::TryFrom;

        let op = Opcode::try_from(i.opcode).unwrap();
        {
            let mut indent = self.label_stack.len();
            if op.code == END || op.code == ELSE {
                indent -= 1;
            }
            info!("{:08x}: L: {} V:{} | {:0width$}{}{:?}" , w.pos(), self.label_stack.len(), self.type_checker.type_stack_size(),  "", op.text, i.imm, width=indent);
        }

        let opc = i.opcode;
        match i.imm {
            None => match opc {
                SELECT => {
                    self.type_checker.on_select()?;
                    w.write_opcode(opc)?;
                },
                DROP => {
                    self.type_checker.on_drop()?;
                    w.write_opcode(opc)?;
                },                
                END => if i.range.end == body.range.end && i.opcode == END {
                    info!("Skipping implicit END");
                } else {
                    info!("END");

                    let ty_label = self.type_checker.get_label(0)?;
                    let label_type = ty_label.label_type;
                    self.type_checker.on_end()?;
                    if label_type == LabelType::If || label_type == LabelType::Else {
                        let label = self.top_label()?;
                        let pos = w.pos();
                        info!("fixup_offset: {:08x} at {:08x}", pos, label.fixup_offset);
                        w.write_u32_at(pos as u32, label.fixup_offset as usize)?;                        
                    }
                    info!("FIXUP");
                    self.fixup(w)?;
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
                    w.write_opcode(BR)?;
                    // Write BR OFFSET fixup 
                    let br_offset = w.pos() as u32;
                    {
                        let mut label = self.top_label_ref()?;
                        label.fixup_offset = br_offset;
                    }
                    w.write_u32(FIXUP_OFFSET)?;

                    // Fixup BR_UNLESS OFFSET
                    let br_pos = w.pos();
                    info!("fixup_offset: {:08x} at {:08x}", br_pos, fixup_cond_offset);
                    w.write_u32_at(br_pos as u32, fixup_cond_offset as usize)?;
                },
                RETURN => {                    
                    // Index drop_count, keep_count;
                    // CHECK_RESULT(GetReturnDropKeepCount(&drop_count, &keep_count));
                    // CHECK_RESULT(typechecker_.OnReturn());
                    // CHECK_RESULT(EmitDropKeep(drop_count, keep_count));
                    // CHECK_RESULT(EmitOpcode(Opcode::Return));                    

                    let (drop, keep) = self.get_return_drop_keep_count()?;
                    self.type_checker.on_return()?;
                    w.write_drop_keep(drop, keep)?;                                    
                    w.write_opcode(RETURN)?;
                },                
                _ => {
                    info!("{:?} {}", op, op.is_binop());
                    if op.is_binop() {
                        self.type_checker.on_binary(&op)?;
                        w.write_opcode(opc)?;
                    } else if op.is_unop() {
                        self.type_checker.on_unary(&op)?;
                        w.write_opcode(opc)?;
                    } else {
                        panic!("{} not implemented", op.text);
                    }
                    
                }           
                // _ => {},
            },
            Block { signature: sig } => match opc {                
                BLOCK => {
                    self.type_checker.on_block(sig)?;
                    self.push_label(FIXUP_OFFSET)?;                    
                },
                LOOP => {
                    self.type_checker.on_loop(sig)?;
                    let pos = w.pos();
                    self.push_label(pos as u32)?;                    
                },
                IF => {
                    // CHECK_RESULT(typechecker_.OnIf(&sig));
                    // CHECK_RESULT(EmitOpcode(Opcode::InterpBrUnless));
                    // IstreamOffset fixup_offset = GetIstreamOffset();
                    // CHECK_RESULT(EmitI32(kInvalidIstreamOffset));
                    // PushLabel(kInvalidIstreamOffset, fixup_offset);
                                        
                    self.type_checker.on_if(sig)?;
                    w.write_opcode(INTERP_BR_UNLESS)?;                    
                    let pos = w.pos();
                    // push label with fixup pointer to BR_UNLESS offset
                    w.write_u32(FIXUP_OFFSET)?;                    
                    self.push_label_fixup(FIXUP_OFFSET, pos as u32)?; 
                },
                _ => unreachable!(),
            },
            Branch { depth } => match opc {
                BR => {
                    let (drop, keep) = self.get_br_drop_keep_count(depth as usize)?;
                    self.type_checker.on_br(depth as usize)?;
                    w.write_drop_keep(drop, keep)?;

                    w.write_opcode(opc)?;
                    let pos = w.pos();
                    self.add_fixup(depth as u32, pos as u32)?;
                    w.write_u32(FIXUP_OFFSET)?;    

                    // CHECK_RESULT(GetBrDropKeepCount(depth, &drop_count, &keep_count));
                    // CHECK_RESULT(typechecker_.OnBr(depth));
                    // CHECK_RESULT(EmitBr(depth, drop_count, keep_count));                    
                },
                BR_IF => {
                    self.type_checker.on_br_if(depth as usize)?;
                    let (drop, keep) = self.get_br_drop_keep_count(depth as usize)?;

                    w.write_opcode(INTERP_BR_UNLESS)?;
                    let fixup_br_offset = w.pos();
                    w.write_u32(FIXUP_OFFSET)?;    
                    w.write_drop_keep(drop, keep)?;

                    w.write_opcode(BR)?;
                    let pos = w.pos();
                    self.add_fixup(depth as u32, pos as u32)?;
                    w.write_u32(FIXUP_OFFSET)?;    

                    let pos = w.pos();
                    w.write_u32_at(pos as u32, fixup_br_offset)?;
                    
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
                // w.write_drop_keep(drop, keep)?;

                // w.write_opcode(op)?;
                // let pos = w.pos();
                // self.add_fixup(depth, pos as u32)?;
                // w.write_u32(FIXUP_OFFSET)?;                
            },
            BranchTable { table } => {
                // BR_TABLE COUNT:u32 TABLE_OFFSET:u32
                // INTERP_DATA SIZE:u32
                // [OFFSET:u32 DROP:u32 KEEP:u32]
                // OFFSET:u32 DROP:u32 KEEP:u32

                let count = table.len() as u32;

                self.type_checker.begin_br_table()?;
                w.write_opcode(BR_TABLE)?;
                w.write_u32(count as u32)?;

                // Write offset of branch table
                let table_pos = w.pos();
                w.write_u32(FIXUP_OFFSET)?;
                
                // Write INTERP_DATA + SIZE
                w.write_opcode(INTERP_DATA)?;
                w.write_u32((count + 1) * BR_TABLE_ENTRY_SIZE)?;

                // Fixup branch table offset
                let pos = w.pos();
                w.write_u32_at(pos as u32, table_pos)?;

                // Branch Table Starts Here

                for depth in table.iter() {
                    self.type_checker.on_br_table_target(*depth as usize)?;
                    self.write_br_table_offset(w, *depth as u32)?;
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
                w.write_opcode(BR_TABLE)?;
                w.write_u32(count as u32)?;

                // Write offset of branch table
                let table_pos = w.pos();
                w.write_u32(FIXUP_OFFSET)?;
                
                // Write INTERP_DATA + SIZE
                w.write_opcode(INTERP_DATA)?;
                w.write_u32((count + 1) * BR_TABLE_ENTRY_SIZE)?;

                // Fixup branch table offset
                let pos = w.pos();
                w.write_u32_at(pos as u32, table_pos)?;

                // Branch Table Starts Here            
            },
            BranchTableDepth { n: _, depth } => {
                self.type_checker.on_br_table_target(depth as usize)?;
                self.write_br_table_offset(w, depth as u32)?;
            },
            BranchTableDefault { depth } => {
                info!("branch table default");
                self.type_checker.on_br_table_target(depth as usize)?;
                self.write_br_table_offset(w, depth as u32)?;

                self.type_checker.end_br_table()?;              
                info!("branch table default done");
            },
            Local { index } => {
                // Emits OP DEPTH_TO_LOCAL
                let id = index.0;
                let ty = self.context[id as usize];
                let local_id = match opc {
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
                w.write_opcode(opc)?;
                w.write_u32(local_id)?;                

            }
            Global { index } => {
                match opc {                    
                    GET_GLOBAL => {
                        let index = index.0 as usize;
                        if index < globals.len() {
                            let global = &globals[index as usize];
                            let global_type = global.global_type();
                            info!("Global: {:?}", global);
                            info!("Global Type: {:?}", global_type);
                            self.type_checker.on_get_global(global_type.type_value)?;
                            w.write_opcode(GET_GLOBAL)?;
                            w.write_u32(index as u32)?;
                        } else {
                            return Err(Error::InvalidGlobal{ id: index as u32});                            
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
                        let index = index.0 as usize;
                        if index < globals.len() {                        
                            let global = &globals[index as usize];
                            let global_type = global.global_type();
                            info!("Global: {:?}", global);
                            info!("Global Type: {:?}", global_type);
                            if global_type.mutability != 0 {
                                return Err(Error::InvalidGlobal { id: index as u32 });
                            }
                            self.type_checker.on_set_global(global_type.type_value)?;
                            w.write_opcode(SET_GLOBAL)?    ;
                            w.write_u32(index as u32)?;
                        } else {
                            return Err(Error::InvalidGlobal{ id: index as u32});
                        }                        

                    },
                    _ => unimplemented!(),
                }              
            },
            Call { index } => {
                info!("CALL {}", index.0);

                let index = index.0 as usize;
                if index < functions.len() {
                    let type_index = functions[index].type_index();
                    let func_type = &types[type_index];
                    info!("Type Index: {:?}", type_index);
                    info!("Type: {:?}", func_type);            
                    self.type_checker.on_call(func_type.parameters, func_type.returns)?;

                    w.write_opcode(opc)?;
                    w.write_u32(index as u32)?;
                } else {
                    return Err(Error::InvalidFunction { id: index as u32})
                }
            },
            CallIndirect { index, reserved: _ } => {
                info!("CALL_INDIRECT: {}", index.0);

                let index = index.0 as usize;
                let func_type = &types[index];
                info!("Type: {:?}", func_type);
                self.type_checker.on_call_indirect(func_type.parameters, func_type.returns)?;
                w.write_opcode(CALL_INDIRECT)?;
                w.write_u32(index as u32)?;                        

            },
            I32Const { value } => {
                self.type_checker.on_const(I32)?;
                w.write_opcode(opc)?;
                w.write_i32(value)?;
            },
            F32Const { value: _ } => { return Err(Error::Unimplemented) },
            I64Const { value: _ } => { return Err(Error::Unimplemented) },
            F64Const { value: _ } => { return Err(Error::Unimplemented) },
            LoadStore { align, offset } => {
                match opc {
                    I32_LOAD | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U => {
                        // CHECK_RESULT(CheckHasMemory(opcode));
                        // CHECK_RESULT(CheckAlign(alignment_log2, opcode.GetMemorySize()));
                        // CHECK_RESULT(typechecker_.OnLoad(opcode));
                        // CHECK_RESULT(EmitOpcode(opcode));
                        // CHECK_RESULT(EmitI32(module_->memory_index));
                        // CHECK_RESULT(EmitI32(offset));

                        self.type_checker.on_load(&op)?;
                    },
                    I32_STORE | I32_STORE8 | I32_STORE16 => {
                        //   CHECK_RESULT(CheckHasMemory(opcode));
                        //   CHECK_RESULT(CheckAlign(alignment_log2, opcode.GetMemorySize()));
                        //   CHECK_RESULT(typechecker_.OnStore(opcode));
                        //   CHECK_RESULT(EmitOpcode(opcode));
                        //   CHECK_RESULT(EmitI32(module_->memory_index));
                        //   CHECK_RESULT(EmitI32(offset));
                        self.type_checker.on_store(&op)?;
                    },
                    _ => unimplemented!(),
                }
                w.write_opcode(opc)?;
                w.write_u32(align)?;
                w.write_u32(offset)?;
            },
            Memory { reserved: _ } => {
                match opc {
                    MEM_SIZE => {
                        self.type_checker.on_current_memory()?;
                    },
                    MEM_GROW => {
                        self.type_checker.on_grow_memory(&op)?;
                    },
                    _ => unimplemented!()
                }
                w.write_opcode(opc)?;
            },
        } 
        Ok(())
    }
}



pub trait ModuleWrite {
    fn write_section_type(&mut self, st: SectionType) -> Result<(), Error>;
    fn write_section_start(&mut self, st: SectionType) -> Result<usize, Error>;
    fn write_section_end(&mut self, fixup: usize) -> Result<(), Error>;
    fn write_type(&mut self, t: TypeValue) -> Result<(), Error>;
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn write_identifier(&mut self, id: Identifier) -> Result<(), Error>;
    fn write_initializer(&mut self, init: Initializer) -> Result<(), Error>;
    fn write_opcode(&mut self, op: u8) -> Result<(), Error>;
    fn write_limits(&mut self, limits: Limits) -> Result<(), Error>;
    fn write_table(&mut self, table: Table) -> Result<(), Error>;
    fn write_memory(&mut self, memory: Memory) -> Result<(), Error>;
    fn write_global_type(&mut self, global_type: GlobalType) -> Result<(), Error>;    
    fn write_import_desc(&mut self, desc: ImportDesc) -> Result<(), Error>;
    fn write_import(&mut self, import: Import) -> Result<(), Error>;
    fn write_code_start(&mut self) -> Result<usize, Error>;
    fn write_code_end(&mut self, fixup: usize) -> Result<(), Error>;

    // Code

    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error>;
    fn write_alloca(&mut self, count: u32) -> Result<(), Error>;
    
}

impl<'a> ModuleWrite for Writer<'a> {
    fn write_section_start(&mut self, st: SectionType) -> Result<usize, Error> {
        self.write_section_type(st)?;
        let pos = self.pos();
        self.write_u32(FIXUP_OFFSET)?;
        Ok(pos)
    }    

    fn write_section_end(&mut self, fixup: usize) -> Result<(), Error> {
        Ok({
            let len = self.pos() - (fixup + 4);
            self.write_u32_at(len as u32, fixup)?;
        })
    }

    fn write_section_type(&mut self, st: SectionType) -> Result<(), Error> {
        self.write_u8(st as u8)
    }    
    fn write_type(&mut self, t: TypeValue) -> Result<(), Error> {
        self.write_u8(t as u8)
    }
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Error> {
        Ok({
            self.write_u32(buf.len() as u32)?;
            for b in buf {
                self.write_u8(*b)?;
            }
        })
    }

    fn write_identifier(&mut self, id: Identifier) -> Result<(), Error> {
        self.write_bytes(id.0)
    }

    fn write_initializer(&mut self, init: Initializer) -> Result<(), Error> {
        Ok({
            self.write_opcode(init.opcode)?;
            self.write_i32(init.immediate)?;
            self.write_opcode(init.end)?;
        })
    }

    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.write_u8(op)
    }

    fn write_limits(&mut self, limits: Limits) -> Result<(), Error> {
        Ok({
            if let Some(max) = limits.max {
                self.write_u32(1)?;
                self.write_u32(limits.min)?;
                self.write_u32(max)?;
            } else {
                self.write_u32(0)?;
                self.write_u32(limits.min)?;            
            }
        })
    }    

    fn write_table(&mut self, table: Table) -> Result<(), Error> {
        Ok({
            self.write_i8(table.element_type as i8)?;
            self.write_limits(table.limits)?;
        })
    }

    fn write_memory(&mut self, memory: Memory) -> Result<(), Error> {
        Ok({
            self.write_limits(memory.limits)?;
        })
    }

    fn write_global_type(&mut self, global_type: GlobalType) -> Result<(), Error> {
        Ok({
            self.write_i8(global_type.type_value as i8)?;
            self.write_u8(global_type.mutability)?;
            
        })
    }

    fn write_import_desc(&mut self, desc: ImportDesc) -> Result<(), Error> {
        Ok({
            match desc {
                ImportDesc::Type(t) => {
                    self.write_u8(0x00)?;
                    self.write_u32(t)?;
                },
                ImportDesc::Table(t) => {
                    self.write_u8(0x01)?;
                    self.write_table(t)?;
                },
                ImportDesc::Memory(m) => {
                    self.write_u8(0x02)?;
                    self.write_memory(m)?;
                },
                ImportDesc::Global(g) => {
                    self.write_u8(0x03)?;                    
                    self.write_global_type(g)?;
                }
            }
        })
    }
    fn write_import(&mut self, import: Import) -> Result<(), Error> {
        Ok({
            self.write_identifier(import.module)?;        
            self.write_identifier(import.export)?;
            self.write_import_desc(import.desc)?;
        })
    }
    
    fn write_code_start(&mut self) -> Result<usize, Error> {
        Ok({
            let pos = self.pos();
            self.write_u32(FIXUP_OFFSET)?;
            pos
        })
    }

    fn write_code_end(&mut self, fixup: usize) -> Result<(), Error> {
        Ok({
            let len = self.pos() - (fixup + 4);
            // info!("code_end pos: {:08x}", self.pos());
            // info!("code_end len: {:08x}", len);
            self.write_u32_at(len as u32, fixup)?;
        })
    }

    // Code

    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error> {
        // info!("drop_keep {}, {}", drop_count, keep_count);
        if drop_count == 1 && keep_count == 0 {
            self.write_opcode(DROP)?;            
        } else if drop_count > 0 {
            self.write_opcode(INTERP_DROP_KEEP)?;
            self.write_u32(drop_count as u32)?;
            self.write_u32(keep_count as u32)?;
        }
        Ok(())
    }

    fn write_alloca(&mut self, count: u32) -> Result<(), Error> {
        Ok(
            if count > 0 {
                self.write_opcode(INTERP_ALLOCA)?;
                self.write_u32(count as u32)?;
            }
        )
    }        
}
