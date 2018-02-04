use Error;
use opcode::*;
use ops::*;
use reader::Reader;
use writer::Writer;

use std::convert::TryFrom;

pub struct Interp {
    stack: [u8; 256],
    stack_pos: usize,
}

impl Interp {
    pub fn new() -> Self {
        Interp {
            stack: [0u8; 256],
            stack_pos: 0,
        }
    }

    pub fn push(&mut self, val: u8) -> Result<(), Error> {
        self.stack[self.stack_pos] = val;
        self.stack_pos += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u8, Error> {
        self.stack_pos -= 1;
        Ok(self.stack[self.stack_pos])
    }

    pub fn peek(&self, offset: usize) -> Result<u8, Error> {
        Ok(self.stack[self.stack_pos - (1 + offset)])
    }

    pub fn load(&mut self, r: &mut Reader, w: &mut Writer) -> Result<(), Error> {
        while r.remaining() > 0 {
            let op = r.read_opcode()?;
            match op {
                BLOCK | LOOP | IF => {
                    w.write_opcode(op)?;
                    w.write_u8(r.read_var_i7()? as u8)?;
                },
                END => {
                    w.write_opcode(op)?;
                }
                BR | BR_IF => {
                    w.write_opcode(op)?;
                    let depth = r.read_var_u32()?;
                    w.write_u32(depth)?;
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
                I32_CONST => {
                    w.write_opcode(op)?;
                    w.write_i32(r.read_var_i32()?)?;
                }
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
                    r.read_var_u1();
                },
                _ => {
                    w.write_opcode(op)?;
                },
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
                    BLOCK | LOOP | IF => print!(" 0x{:02x}", r.read_u8()?),
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

    pub fn run(&mut self, r: &Reader) -> Result<(), Error> {
        Ok(())
    }
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
    fn test_push_pop() {
        let mut i = Interp::new();
        i.push(1).unwrap();
        i.push(2).unwrap();
        i.push(3).unwrap();
        assert_eq!(i.peek(0).unwrap(), 3);
        assert_eq!(i.peek(1).unwrap(), 2);
        assert_eq!(i.peek(2).unwrap(), 1);
        assert_eq!(i.pop().unwrap(), 3);
        assert_eq!(i.pop().unwrap(), 2);
        assert_eq!(i.pop().unwrap(), 1);
    }
    #[test]
    fn test_interp() {
        let code = [
            NOP, 
            MEM_SIZE, 0x00,
            I32_LOAD, 0x02, 0x00,
            I32_ADD,
            I32_SUB,
            CALL, 0x01,
            CALL_INDIRECT, 0x01, 0x00,
            BLOCK, 0x00,
                BR_TABLE, 0x02, 0x00, 0x00, 0x00,
                LOOP, 0x00,
                    BR, 0x01,
                    IF, 0x00,
                        BR_IF, 0x02,
                    END, 
                    IF, 0x00,
                        NOP, 
                        IF, 0x00,
                            NOP, 
                        END, 
                    ELSE, 
                        NOP, 
                        IF, 
                            NOP, 
                        END, 
                    END, 
                END, 
            END, 
            NOP,
        ];  
        // let code = [BLOCK, 0x40, END];
        let mut out = [0u8; 1024];
        let mut r = Reader::new(&code);
        let mut w = Writer::new(&mut out);
        let mut interp = Interp::new();
        interp.load(&mut r, &mut w).unwrap();
        let mut r = Reader::new(w.as_ref());
        interp.dump(&mut r).unwrap();
    }
}