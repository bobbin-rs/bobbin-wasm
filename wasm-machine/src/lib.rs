#![allow(dead_code)]
#![no_std]

extern crate byteorder;
extern crate wasm_leb128;

pub mod opcode;

use byteorder::{ByteOrder, LittleEndian};
use wasm_leb128::{read_i32};

#[derive(Debug, PartialEq)]
pub enum Error {
    Unreachable(usize),
    End(usize),
    Leb128Error(wasm_leb128::Error),
    Unimplemented(usize),    
}

impl From<wasm_leb128::Error> for Error {
    fn from(other: wasm_leb128::Error) -> Error {
        Error::Leb128Error(other)
    }
}


pub struct Machine<'a> {
    code: &'a [u8],
    stack: &'a mut [u8],
    memory: &'a mut [u8],
    locals: &'a mut [u32],
    globals: &'a mut [u32],
    sp: usize,
    pc: usize,
}

impl<'a> Machine<'a> {
    pub fn new(code: &'a [u8], stack: &'a mut[u8], memory: &'a mut[u8], locals: &'a mut[u32], globals: &'a mut[u32]) -> Self {
        let sp = stack.len();
        Machine { code: code, stack: stack, memory: memory, locals: locals, globals: globals, sp: sp, pc: 0 }
    }

    pub fn read_var_i32(&mut self) -> Result<i32, Error> {
        let (v, n) = try!(read_i32(&self.code[self.pc..]));
        self.pc += n;
        Ok(v)
    }

    fn push_i32(&mut self, value: i32) {
        assert!(self.sp >= 4);
        self.sp -= 4; 
        LittleEndian::write_i32(&mut self.stack[self.sp..], value);
    }

    fn pop_i32(&mut self) -> i32 {
        assert!(self.sp <= self.stack.len() - 4);
        let v = LittleEndian::read_i32(&self.stack[self.sp..]);
        self.sp += 4;
        v
    }

    // Return error or the next PC
    pub fn step(&mut self) -> Result<usize, Error> {
        use self::opcode::*;
        let op = self.code[self.pc];        
        match op {
            UNREACHABLE => return Err(Error::Unreachable(self.pc)),
            NOP => self.pc += 1,
            BLOCK => self.pc += 1,
            LOOP => self.pc += 1,
            

            I32_CONST => {                
                self.pc += 1;
                let r = try!(self.read_var_i32());
                self.push_i32(r);
            },
            END => return Err(Error::End(self.pc)),
            _ => return Err(Error::Unimplemented(self.pc)),
        }
        Ok(self.pc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::opcode::*;

    #[test]
    fn test_unreachable() {
        let mut locals = [0u32; 16];
        let mut globals = [0u32; 16];
        let mut memory = [0u8; 16];
        let mut stack = [0u8; 16];
        let code: [u8;1] = [UNREACHABLE];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);
        assert_eq!(machine.step(), Err(Error::Unreachable(0)));
    }

    #[test]
    fn test_end() {
        let mut locals = [0u32; 16];
        let mut globals = [0u32; 16];
        let mut memory = [0u8; 16];
        let mut stack = [0u8; 16];
        let code: [u8;1] = [END];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);
        assert_eq!(machine.step(), Err(Error::End(0)));
    }

    #[test]
    fn test_nop() {
        let mut locals = [0u32; 16];
        let mut globals = [0u32; 16];
        let mut memory = [0u8; 16];
        let mut stack = [0u8; 16];
        let code: [u8; 3] = [NOP, NOP, END];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);
        assert_eq!(machine.step(), Ok(1));
        assert_eq!(machine.step(), Ok(2));
        assert_eq!(machine.step(), Err(Error::End(2)));
    }

    #[test]
    fn test_sequence() {
        let mut locals = [0u32; 16];
        let mut globals = [0u32; 16];
        let mut memory = [0u8; 16];
        let mut stack = [0u8; 16];        
        let code: [u8; 3] = [NOP, NOP, END];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);

    }
}
