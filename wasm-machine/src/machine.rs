use Error;
use TypeValue;
use Value;
use stack::Stack;

use reader::Reader;
use opcode::*;
use byteorder::{ByteOrder, LittleEndian};

pub struct Machine<'g, 'f, 's, 'm, 'st, 'c> {
    globals: &'g [TypeValue],
    functions: &'f [u32],
    signatures: &'s [(&'s [TypeValue], &'s [TypeValue])],
    memory: &'m mut [u8],
    stack: &'st mut Stack<'st, Value>,
    code: &'c mut Reader<'c>,
}

impl<'g, 'f, 's, 'm, 'st, 'c> Machine<'g, 'f, 's, 'm, 'st, 'c> {
    pub fn new(
        globals: &'g [TypeValue],
        functions: &'f [u32],
        signatures: &'s [(&'s [TypeValue], &'s [TypeValue])],
        memory: &'m mut [u8],
        stack: &'st mut Stack<'st, Value>,
        code: &'c mut Reader<'c>,
    ) -> Self {
        Machine { globals, functions, signatures, memory, stack, code }
    }

    // Code

    pub fn done(&self) -> bool {
        self.code.done()
    }

    // Stack

    pub fn push<T: Into<Value>>(&mut self, value: T) -> Result<(), Error> {
        let value = value.into();
        println!("PUSH @ {:02} {:?}", self.stack.len(), value);
        Ok(self.stack.push(value.into())?)
    }

    pub fn pop<T: From<Value>>(&mut self) -> Result<T, Error> {
        Ok(self.stack.pop()?.into())
    }

    // Locals

    pub fn get_local<T: From<Value>>(&self, depth: u32) -> Result<T, Error> {
        Ok(self.stack.peek(depth as usize)?.into())
    }

    pub fn set_local<T: Into<Value>>(&mut self, depth: u32, value: T) -> Result<(), Error> {
        Ok(*self.stack.pick(depth as usize)? = value.into())
    }

    // Globals

    pub fn get_global<T: From<Value>>(&self, depth: u32) -> Result<T, Error> {
        Ok(self.stack.get(depth as usize)?.into())
    }

    pub fn set_global<T: Into<Value>>(&mut self, depth: u32, value: T) -> Result<(), Error> {
        Ok(self.stack.set(depth as usize, value.into())?)
    }

    // Memory

    pub fn get_memory_u8(&self, addr: u32) -> Result<u8, Error> {
        let addr = addr as usize;
        if addr < self.memory.len() {
            Ok(self.memory[addr as usize])
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn get_memory_u16(&self, addr: u32) -> Result<u16, Error> {
        let addr = addr as usize;
        if addr + 1 < self.memory.len() {
            Ok(LittleEndian::read_u16(&self.memory[addr..]))
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn get_memory_u32(&self, addr: u32) -> Result<u32, Error> {
        let addr = addr as usize;
        if addr + 3 < self.memory.len() {
            Ok(LittleEndian::read_u32(&self.memory[addr..]))
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn set_memory_u8(&mut self, addr: u32, value: u8) -> Result<(), Error> {
        let addr = addr as usize;
        if addr < self.memory.len() {
            Ok(self.memory[addr as usize] = value)
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn set_memory_u16(&mut self, addr: u32, value: u16) -> Result<(), Error> {
        let addr = addr as usize;
        if addr + 1 < self.memory.len() {
            Ok(LittleEndian::write_u16(&mut self.memory[addr..], value))
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn set_memory_u32(&mut self, addr: u32, value: u32) -> Result<(), Error> {
        let addr = addr as usize;
        if addr + 3 < self.memory.len() {
            Ok(LittleEndian::write_u32(&mut self.memory[addr..], value))
        } else {
            Err(Error::OutOfBounds)
        }
    }    

    pub fn reset(&mut self) -> Result<(), Error> {
        self.stack.reset()?;
        for _ in self.globals.iter() {
            self.push(0)?;
        }
        for m in self.memory.iter_mut() {
            *m = 0;
        }
        Ok(())
    }

    pub fn run(&mut self, count: usize) -> Result<usize, Error> {
        let mut n = 0;
        while n < count && !self.done(){
            let pc = self.code.pos();
            let op = self.code.read_u8()?;
            println!("{:07x}: {:02x}", pc, op);
            match op {
                UNREACHABLE => return Err(Error::Unreachable),
                NOP => {},
                BR => {},
                BR_IF => {},
                BR_TABLE => {},
                RETURN => {},
                CALL => {},
                CALL_INDIRECT => {},
                DROP => {
                    let _: u32 = self.pop()?;
                },
                SELECT => {},
                GET_LOCAL | SET_LOCAL | TEE_LOCAL |
                GET_GLOBAL | SET_GLOBAL => {
                    let id = self.code.read_u32()?;
                    match op {
                        GET_LOCAL => {
                            let value: u32 = self.get_local(id)?;
                            self.push(value)?;
                        },
                        SET_LOCAL => {
                            let value: u32 = self.pop()?;
                            self.set_local(id, value)?;
                        },
                        TEE_LOCAL => {
                            let value: u32 = self.pop()?;
                            self.set_local(id, value)?;
                            self.push(value)?;
                        },
                        GET_GLOBAL => {
                            let value: u32 = self.get_global(id)?;
                            self.push(value)?;
                        },
                        SET_GLOBAL => {
                            let value: u32 = self.pop()?;
                            self.set_global(id, value)?;
                        },
                        _ => unimplemented!()
                    }
                },
                I32_CONST => {
                    let v = self.code.read_u32()?;
                    self.push(v)?;
                },
                // I32 load
                0x28 ... 0x30 => {
                    let _flags = self.code.read_u32()?;
                    let offset = self.code.read_u32()?;
                    let base: u32 = self.pop()?;
                    let addr = offset + base;

                    let res = match op {
                        I32_LOAD => {
                            self.get_memory_u32(addr)?
                        },
                        I32_LOAD8_S => {
                            self.get_memory_u8(addr)? as i8 as u32
                        },
                        I32_LOAD8_U => {
                            self.get_memory_u8(addr)? as i8 as u32
                        },
                        I32_LOAD16_S => {
                            self.get_memory_u16(addr)? as i16 as u32
                        },
                        I32_LOAD16_U => {
                            self.get_memory_u16(addr)? as u16 as u32
                        }                                               
                        _ => unimplemented!(),
                    };
                    self.push(res)?;
                },
                // I32 store
                0x36 | 0x38 | 0x3a | 0x3b => {
                    let _flags = self.code.read_u32()?;
                    let offset = self.code.read_u32()?;
                    let base: u32 = self.pop()?;
                    let value: u32 = self.pop()?;
                    let addr = offset + base;
                    match op {
                        I32_STORE => self.set_memory_u32(addr, value)?,
                        I32_STORE8 => self.set_memory_u8(addr, value as u8)?,
                        I32_STORE16 => self.set_memory_u16(addr, value as u16)?,
                        _ => unimplemented!(),
                    }
                },
                // I32 cmpops
                0x46 ... 0x50 => {
                    let (rhs, lhs): (u32, u32) = (self.pop()?, self.pop()?);
                    let res = match op {
                        I32_EQ => lhs == rhs,
                        I32_NE => lhs != rhs,
                        I32_LT_S => (lhs as i32) < (rhs as i32),
                        I32_LT_U => lhs < rhs,
                        I32_GT_S => (lhs as i32) > (rhs as i32),
                        I32_GT_U => lhs > rhs,
                        I32_LE_S => (lhs as i32) <= (rhs as i32),
                        I32_LE_U => lhs <= rhs,
                        I32_GE_S => (lhs as i32) >= (rhs as i32),
                        I32_GE_U => lhs >= rhs,                        
                        _ => return Err(Error::Unimplemented),
                    };
                    self.push(if res { 1 } else { 0 })?;
                }
                // I32 binops
                0x6a ... 0x79 => {
                    let (rhs, lhs): (u32, u32) = (self.pop()?, self.pop()?);
                    let res = match op {
                        I32_ADD => lhs.wrapping_add(rhs),
                        I32_SUB => lhs.wrapping_sub(rhs),
                        I32_MUL => lhs.wrapping_mul(rhs),
                        I32_DIV_S => ((lhs as i32) / (rhs as i32)) as u32,
                        I32_DIV_U => lhs / rhs,
                        I32_REM_S => ((lhs as i32) % (rhs as i32)) as u32,
                        I32_REM_U => lhs % rhs,
                        I32_AND => lhs & rhs,
                        I32_OR => lhs | rhs,
                        I32_XOR => lhs ^ rhs,
                        I32_SHL => lhs << rhs,
                        I32_SHR_S => ((lhs as i32) >> rhs) as u32,
                        I32_SHR_U => lhs >> rhs,
                        I32_ROTL => lhs.rotate_left(rhs),
                        I32_ROTR => lhs.rotate_right(rhs),
                        _ => unimplemented!()
                    };
                    self.push(res)?;
                },
                // I32 unops                
                0x45 | 0x67 ... 0x6a => {
                    let val: u32 = self.pop()?;
                    let res = match op {
                        I32_EQZ => if val == 0 { 1 } else { 0 },
                        I32_CLZ => val.leading_zeros(),
                        I32_CTZ => val.trailing_zeros(),
                        I32_POPCNT => val.count_zeros(),
                        _ => unimplemented!(),
                    };
                    self.push(res)?;
                },
                INTERP_ALLOCA => {

                },
                INTERP_BR_UNLESS => {

                },
                INTERP_DROP_KEEP => {
                    let (drop, keep) = (self.code.read_u32()?, self.code.read_u32()?);
                    self.stack.drop_keep(drop as usize, keep as usize)?;
                },
                _ => {
                    return Err(Error::Unimplemented)
                },
            }
            n += 1;
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use writer::Writer;
    use loader::WriteLoader;

    #[test]
    fn test_machine() {
        let f1_p = [I32, I32];
        let f1_r = [I32];

        let f2_p = [I32];
        let f2_r = [];

        let globals = [I32, I32];
        let functions = [0, 1];
        let signatures = [(&f1_p[..], &f1_r[..]), (&f2_p[..], &f2_r[..])];

        let mut memory = [0u8; 1024];

        let mut stack_buf = [Value::default(); 256];
        let mut stack = Stack::new(&mut stack_buf);

        let mut code_buf = [0u8; 256];
        let mut w = Writer::new(&mut code_buf);

        w.write_opcode(NOP).unwrap();
        w.write_opcode(I32_CONST).unwrap();
        w.write_u32(0x12).unwrap();
        w.write_opcode(I32_CONST).unwrap();
        w.write_u32(0x34).unwrap();
        w.write_opcode(I32_ADD).unwrap();
        w.write_opcode(I32_CONST).unwrap();
        w.write_i32(2).unwrap();
        w.write_opcode(I32_MUL).unwrap();
        w.write_opcode(I32_CONST).unwrap();
        w.write_u32(10).unwrap();
        w.write_opcode(I32_SUB).unwrap();
        

        let mut code: Reader = w.into();

        let mut m = Machine::new(
            &globals,
            &functions,
            &signatures,
            &mut memory,
            &mut stack,
            &mut code,
        );
        m.run(100).unwrap();
        println!("---");
        m.stack.dump();
    }

}