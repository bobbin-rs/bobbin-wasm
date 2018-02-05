use Error;
use opcode::*;
use reader::Reader;
use writer::Writer;
use stack::Stack;

use core::fmt;
use core::convert::TryFrom;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Fixup {
    depth: usize,
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
    signature: i8,
    offset: u32,
    stack_limit: usize,
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Label {{ signature: 0x{:02x}, offset: 0x{:08x}, stack_limit: {} }}", self.signature, self.offset, self.stack_limit)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum TypeValue {
    None = 0x00,
    I32 = -0x01,
    I64 = -0x02,
    F32 = -0x03,
    F64 = -0x04,
    Void = 0x40,
}

impl Default for TypeValue {
    fn default() -> Self {
        TypeValue::None
    }
}

impl From<i8> for TypeValue {
    fn from(other: i8) -> Self {
        match other {
             0x00 => TypeValue::None,
            -0x01 => TypeValue::I32,
            -0x02 => TypeValue::I64,
            -0x03 => TypeValue::F32,
            -0x04 => TypeValue::F64,
            -0x40 => TypeValue::Void,
            _ => panic!("Unrecognized TypeValue: 0x{:02x}", other)
        }
    }
}



pub struct Interp<'s, 't> {
    label_stack: Stack<'s, Label>,
    type_stack: Stack<'t, TypeValue>,
    fixups: [Option<Fixup>; 256],
    fixups_pos: usize,
}

impl<'s, 't> Interp<'s, 't> {
    pub fn new(label_stack: Stack<'s, Label>, type_stack: Stack<'t, TypeValue>) -> Self {
        Interp {
            label_stack: label_stack,
            type_stack: type_stack,
            fixups: [None; 256],
            fixups_pos: 0,
        }
    }

    pub fn push_label(&mut self, signature: i8, offset: u32) -> Result<(), Error> {
        let stack_limit = self.type_stack.len();
        let label = Label {
            signature,
            offset,
            stack_limit,
        };
        println!("push_label: {:?}", label);
        Ok(self.label_stack.push(label)?)
    }

    pub fn pop_label(&mut self) -> Result<Label, Error> {
        Ok(self.label_stack.pop()?)
    }

    pub fn label_depth(&self) -> usize {
        self.label_stack.len()
    }

    pub fn peek_label(&self, offset: usize) -> Result<Label, Error> {
        Ok(self.label_stack.peek(offset)?)
    }

    pub fn push_type<T: Into<TypeValue>>(&mut self, type_value: T) -> Result<(), Error> {
        Ok(self.type_stack.push(type_value.into())?)
    }

    pub fn add_fixup(&mut self, rel_depth: u32, offset: u32) -> Result<(), Error> {
        let depth = self.label_depth() - rel_depth as usize;
        let fixup = Fixup { depth: depth, offset: offset };
        println!("add_fixup: {:?}", fixup);
        for entry in self.fixups.iter_mut() {
            if entry.is_none() {
                *entry = Some(fixup);
                return Ok(());
            }
        }
        Err(Error::FixupsFull)
    }

    pub fn fixup(&mut self, w: &mut Writer) -> Result<(), Error> {
        let depth = self.label_depth();        
        let offset = self.peek_label(0)?.offset;
        let offset = if offset == FIXUP_OFFSET { w.pos() } else { offset as usize};
        println!("fixup: {} -> 0x{:08x}", depth, offset);
        for entry in self.fixups.iter_mut() {
            let del = if let &mut Some(entry) = entry {
                if entry.depth == depth {
                    println!(" {:?}", entry);
                    w.write_u32_at(offset as u32, entry.offset as usize)?;
                    true
                } else {
                    // println!(" ! {} 0x{:04x}", entry.depth, entry.offset);                    
                    false
                }
            } else {
                false
            };
            if del {
                *entry = None;
            }
        }
        println!("fixup done");
        Ok(())
    }

    pub fn load(&mut self, r: &mut Reader, w: &mut Writer) -> Result<(), Error> {
        while r.remaining() > 0 {
            let op = r.read_opcode()?;
            let opc = Opcode::try_from(op)?;
            println!("{:04x}: 0x{:02x} {}", w.pos(), opc.code, opc.text);
            match op {
                BLOCK => {
                    self.push_label(r.read_var_i7()?, FIXUP_OFFSET)?;
                    println!("DEPTH -> {}", self.label_depth());
                },
                LOOP => {
                    self.push_label(r.read_var_i7()?, w.pos() as u32)?;
                    println!("DEPTH -> {}", self.label_depth());
                },
                IF => {
                    self.push_label(r.read_var_i7()?, FIXUP_OFFSET)?;
                    println!("IF: DEPTH -> {}", self.label_depth());
                    w.write_opcode(INTERP_BR_UNLESS)?;
                    println!("IF: ADD FIXUP {} 0x{:04x}", 0, w.pos());
                    self.add_fixup(0, w.pos() as u32)?;
                    w.write_u32(0xffffffff)?;
                },                
                END => {
                    // w.write_opcode(op)?;
                    println!("FIXUP {} 0x{:04x}", self.label_depth(), w.pos());
                    self.fixup(w)?;
                    self.pop_label()?;
                    println!("DEPTH -> {}", self.label_depth());
                },
                ELSE => {
                    w.write_opcode(BR)?;
                    self.fixup(w)?;
                    println!("ELSE: ADD FIXUP {} 0x{:04x}", 0, w.pos());
                    self.add_fixup(0, w.pos() as u32)?;
                    w.write_u32(0xffffffff)?;
                }
                BR | BR_IF => {
                    w.write_opcode(op)?;
                    let depth = r.read_var_u32()?;
                    println!("BR / BR_IF ADD FIXUP {} 0x{:04x}", depth, w.pos());
                    self.add_fixup(depth, w.pos() as u32)?;
                    w.write_u32(0xfffffff)?;
                },
                BR_TABLE => {
                    w.write_opcode(op)?;
                    let n = r.read_var_u32()?;
                    w.write_u32(n)?;
                    for _ in 0..n {
                        w.write_u32(r.read_var_u32()?)?;
                    }
                    w.write_u32(r.read_var_u32()?)?;
                },
                GET_LOCAL | SET_LOCAL | TEE_LOCAL => {
                    w.write_opcode(op)?;
                    w.write_u32(r.read_var_u32()?)?;
                },
                GET_GLOBAL | SET_GLOBAL => {
                    w.write_opcode(op)?;
                    w.write_u32(r.read_var_u32()?)?;
                },
                CALL => {
                    w.write_opcode(op)?;
                    w.write_u32(r.read_var_u32()?)?;
                },
                CALL_INDIRECT => {
                    w.write_opcode(op)?;
                    w.write_u32(r.read_var_u32()?)?;
                    r.read_var_u1()?;
                },
                I32_LOAD | I32_STORE | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U => {
                    w.write_opcode(op)?;
                    w.write_u32(r.read_var_u32()?)?;
                    w.write_u32(r.read_var_u32()?)?;
                },
                MEM_GROW | MEM_SIZE => {
                    w.write_opcode(op)?;
                    r.read_var_u1()?;
                },
                I32_CONST => {
                    w.write_opcode(op)?;
                    w.write_i32(r.read_var_i32()?)?;
                    self.push_type(I32)?;
                },
                _ => {
                    if opc.is_unop() {

                    } else if opc.is_binop() {

                    }
                    w.write_opcode(op)?;
                },
            }
        }
        println!("remaining fixups\n---");
        for entry in self.fixups.iter() {
            if let &Some(entry) = entry {
                println!("{:?}", entry);
            }
        }
        Ok(())
    }

    pub fn dump(&self, r: &mut Reader) -> Result<(), Error> {
        while r.remaining() > 0 {
            let pc = r.pos();
            let b = r.read_opcode()?;
            if let Ok(op) = Opcode::try_from(b) {
                print!("0x{:02x}: {}", pc, op.text);
                match op.code {
                    BLOCK | LOOP => print!(" 0x{:02x}", r.read_u8()?),
                    IF => print!(" 0x{:04x}", r.read_u32()?),
                    ELSE => print!(" 0x{:04x}", r.read_u32()?),
                    BR | BR_IF => print!(" {:08x}", r.read_u32()?),
                    BR_TABLE => {
                        for _ in 0..r.read_u32()? {
                            print!(" {}", r.read_u32()?);
                        }
                        print!(" default {}", r.read_u32()?);
                    }
                    I32_CONST => print!(" {}", r.read_i32()?),
                    GET_LOCAL | SET_LOCAL | TEE_LOCAL => print!(" {}", r.read_u32()?),
                    GET_GLOBAL | SET_GLOBAL => print!(" {}", r.read_u32()?),
                    CALL => print!(" {}", r.read_u32()?),
                    CALL_INDIRECT => print!(" {:02x}", r.read_u32()?),
                    I32_LOAD | I32_STORE | I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U => {
                        print!(" {}", r.read_u32()?);
                        print!(" 0x{:08x}", r.read_u32()?);
                    },
                    MEM_GROW | MEM_SIZE => {},
                    INTERP_BR_UNLESS => print!(" {:08x}", r.read_u32()?),
                    _ => {},
                }
                println!("");
            } else {
                println!("0x{:02x}", b);

            }
        }
        Ok(())
    }

    // pub fn run(&mut self, r: &Reader) -> Result<(), Error> {
    //     Ok(())
    // }
}

trait ReadInterp {
    fn read_opcode(&mut self) -> Result<u8, Error>;
}

impl<'r> ReadInterp for Reader<'r> {
    fn read_opcode(&mut self) -> Result<u8, Error> {
        self.read_u8()
    }    
}

trait WriteInterp {
    fn write_opcode(&mut self, op: u8) -> Result<(), Error>;
}

impl<'w> WriteInterp for Writer<'w> {
    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.write_u8(op)
    }    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interp() {
        // let code = [
        //     NOP, 
        //     MEM_SIZE, 0x00,
        //     I32_LOAD, 0x02, 0x00,
        //     I32_ADD,
        //     I32_SUB,
        //     CALL, 0x01,
        //     CALL_INDIRECT, 0x01, 0x00,
        //     BLOCK, 0x00, // Depth -> 1
        //         BR_TABLE, 0x02, 0x00, 0x00, 0x00,
        //         LOOP, 0x00, // Depth -> 2
        //             BR, 0x00, // Add Fixup 2
        //             IF, 0x00, // Depth -> 3
        //                 BR_IF, 0x02, // Add Fixup 1
        //             END, // Fixup 3, Depth -> 2
        //             IF, 0x00, // Depth -> 3
        //                 NOP, 
        //                 IF, 0x00, // Depth -> 4
        //                     NOP, 
        //                 END, // Depth -> 3
        //             ELSE, // replace with BR, add fixup 3
        //                 NOP, 
        //                 IF, // depth -> 4
        //                     NOP, 
        //                 END, // depth -> 3
        //             END,  // Fixup 3, Depth -> 2
        //         END, // Fixup 2, Depth -> 1
        //     END, // Fixup 1, Depth -> 0
        //     NOP,
        // ];  

        // 0x00: nop
        // 0x01: mem_size
        // 0x02: i32.load 2 0x00000000
        // 0x0b: i32.add
        // 0x0c: i32.sub
        // 0x0d: call 1
        // 0x12: call_indirect 01
        // 0x17: br_table 0 0 default 0
        // 0x28: br 00000028
        // 0x2d: br_unless 00000037
        // 0x32: br_if 0000004e
        // 0x37: br_unless 0000004e
        // 0x3c: nop
        // 0x3d: br_unless 00000043
        // 0x42: nop
        // 0x43: br 0000004e
        // 0x48: nop
        // 0x49: br_unless 0000004e
        // 0x4e: nop

        // let code = [BLOCK, 0x40, NOP, BR, 0, NOP, END];
        // 0x00: nop
        // 0x01: br 00000007
        // 0x06: nop
        // let code = [IF, 0x00, NOP, BR_IF, 0x00, NOP, ELSE, NOP, BR_IF, 0x00, NOP, END];
        // 0x00: br_unless 00000018
        // 0x05: nop
        // 0x06: br_if 00000018
        // 0x0b: nop
        // 0x0c: br 00000018
        // 0x11: nop
        // 0x12: br_if 00000018
        // 0x17: nop
        // let code = [
        //     NOP,
        //     LOOP, 0x00, // Depth -> 1
        //         NOP,
        //         BR, 0x00,
        //     END, // Add BR to top of loop, Fixup 1, Depth -> 0
        // ];
        // 0x00: nop
        // 0x01: nop
        // 0x02: br 00000001

        let code = [I32_CONST, 0x12];

        let mut out = [0u8; 1024];
        let mut r = Reader::new(&code);
        let mut w = Writer::new(&mut out);

        let mut labels_buf = [Label::default(); 256];
        let label_stack = Stack::new(&mut labels_buf);

        let mut type_buf = [TypeValue::default(); 256];
        let type_stack = Stack::new(&mut type_buf);

        let mut interp = Interp::new(label_stack, type_stack);
        interp.load(&mut r, &mut w).unwrap();
        println!("STACK");
        interp.type_stack.dump();
        println!("");

        let mut r = Reader::new(w.as_ref());
        interp.dump(&mut r).unwrap();
    }
}