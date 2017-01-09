#![allow(dead_code)]
#![no_std]

extern crate byteorder;
extern crate wasm_leb128;

pub mod opcode;
pub mod writer;

use byteorder::{ByteOrder, LittleEndian};
use wasm_leb128::{read_i32, read_u32};

#[derive(Debug, PartialEq)]
pub enum Error {
    Unreachable,
    End,
    Unimplemented,
    Leb128Error(wasm_leb128::Error),
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
    locals: &'a mut [i32],
    globals: &'a mut [i32],
    sp: usize,
    pc: usize,
}

impl<'a> Machine<'a> {
    pub fn new(code: &'a [u8], stack: &'a mut[u8], memory: &'a mut[u8], locals: &'a mut[i32], globals: &'a mut[i32]) -> Self {
        let sp = stack.len();
        Machine { code: code, stack: stack, memory: memory, locals: locals, globals: globals, sp: sp, pc: 0 }
    }

    pub fn set_local(&mut self, index: usize, value: i32) {
        self.locals[index] = value;
    }

    pub fn get_local(&mut self, index: usize) -> i32 {
        self.locals[index]
    }

    pub fn set_global(&mut self, index: usize, value: i32) {
        self.globals[index] = value;
    }

    pub fn get_global(&mut self, index: usize) -> i32 {
        self.globals[index]
    }

    pub fn read_var_i32(&mut self) -> Result<i32, Error> {
        let (v, n) = try!(read_i32(&self.code[self.pc..]));
        self.pc += n;
        Ok(v)
    }

    pub fn read_var_u32(&mut self) -> Result<u32, Error> {
        let (v, n) = try!(read_u32(&self.code[self.pc..]));
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

    fn top_i32(&self) -> i32 {
        LittleEndian::read_i32(&self.stack[self.sp..])
    }

    // Return error or the next PC
    pub fn step(&mut self) -> Result<(), Error> {
        use self::opcode::*;
        let op = self.code[self.pc];        
        match op {
            // Control Flow Operators
            UNREACHABLE => return Err(Error::Unreachable),
            NOP => self.pc += 1,
            BLOCK => self.pc += 1,
            LOOP => self.pc += 1,
            IF => unimplemented!(),
            ELSE => unimplemented!(),
            END => return Err(Error::End),
            BR => unimplemented!(),
            BR_IF => unimplemented!(),
            BR_TABLE => unimplemented!(),
            RETURN => unimplemented!(),

            // Call Operators
            CALL => unimplemented!(),
            CALL_INDIRECT => unimplemented!(),

            // Parametric Operators
            DROP => unimplemented!(),
            SELECT => unimplemented!(),

            // Variable Access
            GET_LOCAL => {
                self.pc += 1;
                let index = try!(self.read_var_u32());
                let value = self.locals[index as usize];
                self.push_i32(value);
            },
            SET_LOCAL => {
                self.pc += 1;
                let index = try!(self.read_var_u32());
                let value = self.pop_i32();
                self.locals[index as usize] = value;                
            },  
            TEE_LOCAL => {
                self.pc += 1;
                let index = try!(self.read_var_u32());                
                self.locals[index as usize] = self.top_i32();                
            },  
            GET_GLOBAL => {
                self.pc += 1;
                let index = try!(self.read_var_u32());
                let value = self.globals[index as usize];
                self.push_i32(value);
            },
            SET_GLOBAL => {
                self.pc += 1;
                let index = try!(self.read_var_u32());
                let value = self.pop_i32();
                self.globals[index as usize] = value;                
            },  

            // Memory Operators
            I32_LOAD => unimplemented!(),
            I32_LOAD8_S => unimplemented!(),
            I32_LOAD8_U => unimplemented!(),
            I32_LOAD16_S => unimplemented!(),
            I32_LOAD16_U => unimplemented!(),
            I32_STORE => unimplemented!(),
            I32_STORE8 => unimplemented!(),
            I32_STORE16 => unimplemented!(),

            // Constants
            I32_CONST => {       
                self.pc += 1;         
                let r = try!(self.read_var_i32());
                self.push_i32(r);
            },

            // Comparison Operators
            I32_EQZ => unimplemented!(),
            I32_EQ => unimplemented!(),
            I32_NE => unimplemented!(),
            I32_LT_S => unimplemented!(),
            I32_LT_U => unimplemented!(),
            I32_GT_S => unimplemented!(),
            I32_GT_U => unimplemented!(),
            I32_LE_S => unimplemented!(),
            I32_LE_U => unimplemented!(),
            I32_GE_S => unimplemented!(),
            I32_GE_U => unimplemented!(),

            // Numeric Operators
            I32_CLZ => unimplemented!(),
            I32_CTZ => unimplemented!(),
            I32_POPCNT => unimplemented!(),
            I32_ADD => unimplemented!(),
            I32_SUB => unimplemented!(),
            I32_MUL => unimplemented!(),
            I32_DIV_S => unimplemented!(),
            I32_DIV_U => unimplemented!(),
            I32_REM_S => unimplemented!(),
            I32_REM_U => unimplemented!(),
            I32_AND => unimplemented!(),
            I32_OR => unimplemented!(),
            I32_XOR => unimplemented!(),
            I32_SHL => unimplemented!(),
            I32_SHR_S => unimplemented!(),
            I32_SHR_U => unimplemented!(),
            I32_ROTL => unimplemented!(),
            I32_ROTR => unimplemented!(),

            _ => return Err(Error::Unimplemented),
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.step() {
                Ok(()) => {},
                Err(Error::End) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use super::opcode::*;
    use writer::*;

    fn with_machine<F: FnOnce(&mut Machine)>(code: &[u8], f: F) {
        let mut locals = [0i32; 16];
        let mut globals = [0i32; 16];
        let mut memory = [0u8; 16];
        let mut stack = [0u8; 16];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);
        f(&mut machine)
    }

    #[test]
    fn test_unreachable() {
        let code: [u8;1] = [UNREACHABLE];
        with_machine(&code[..], |m| {
            assert_eq!(m.step(), Err(Error::Unreachable));
        });
    }


    #[test]
    fn test_end() {
        let code: [u8;1] = [END];
        with_machine(&code[..], |m| {
            assert_eq!(m.step(), Err(Error::End));
        });
    }


    #[test]
    fn test_nop() {
        let code: [u8; 3] = [NOP, NOP, END];
        with_machine(&code[..], |m| {
            assert_eq!(m.step(), Ok(()));
            assert_eq!(m.step(), Ok(()));
            assert_eq!(m.step(), Err(Error::End));
        });        
    }

    #[test]
    fn test_i32_const() {
        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);
        w.write_u8(I32_CONST);
        w.write_var_i32(0x12345678);
        w.write_u8(END);
        with_machine(&w, |m| {            
            assert_eq!(m.run(), Ok(()));            
            assert_eq!(m.pop_i32(), 0x12345678);
        });        
    }    

    #[test]
    fn test_get_global() {
        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);
        w.write_u8(GET_GLOBAL);
        w.write_var_u32(0);
        w.write_u8(END);
        with_machine(&w, |m| {            
            m.set_global(0, 0x12345678);
            assert_eq!(m.run(), Ok(()));            
            assert_eq!(m.pop_i32(), 0x12345678);
        });        
    }        

    #[test]
    fn test_set_global() {
        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);
        w.write_u8(SET_GLOBAL);
        w.write_var_u32(0);
        w.write_u8(END);
        with_machine(&w, |m| {
            m.push_i32(0x12345678);            
            assert_eq!(m.run(), Ok(()));            
            assert_eq!(m.get_global(0), 0x12345678);
        });        
    }        


    #[test]
    fn test_get_local() {
        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);
        w.write_u8(GET_LOCAL);
        w.write_var_u32(0);
        w.write_u8(END);
        with_machine(&w, |m| {            
            m.set_local(0, 0x12345678);
            assert_eq!(m.run(), Ok(()));            
            assert_eq!(m.pop_i32(), 0x12345678);
        });        
    }        

    #[test]
    fn test_set_local() {
        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);
        w.write_u8(SET_LOCAL);
        w.write_var_u32(0);
        w.write_u8(END);
        with_machine(&w, |m| {
            m.push_i32(0x12345678);            
            assert_eq!(m.run(), Ok(()));            
            assert_eq!(m.get_local(0), 0x12345678);
        });        
    }        

    #[test]
    fn test_tee_local() {
        let mut buf = [0u8; 64];
        let mut w = Writer::new(&mut buf);
        w.write_u8(TEE_LOCAL);
        w.write_var_u32(0);
        w.write_u8(END);
        with_machine(&w, |m| {
            m.push_i32(0x12345678);            
            assert_eq!(m.run(), Ok(()));            
            assert_eq!(m.get_local(0), 0x12345678);
            assert_eq!(m.pop_i32(), 0x12345678);
        });        
    }   
}
