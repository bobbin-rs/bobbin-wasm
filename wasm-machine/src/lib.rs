#![allow(dead_code)]
//#![no_std]
#![feature(try_from)]

extern crate core;

extern crate byteorder;
extern crate wasm_leb128;

pub mod ops;
pub mod opcode;
pub mod reader;
pub mod writer;
pub mod stack;
pub mod scanner;
pub mod loader;

// use byteorder::{ByteOrder, LittleEndian};
// use wasm_leb128::{read_i32, read_u32};

#[derive(Debug, PartialEq)]
pub enum Error {
    Unreachable,
    End,
    Unimplemented,
    InvalidBlockType,
    ScopesFull,
    FixupsFull,
    InvalidIfSignature,
    InvalidReservedValue,
    InvalidBranchTableDefault { id: usize, len: usize},
    InvalidLocal { id: usize, len: usize },
    InvalidGlobal { id: usize, len: usize },
    InvalidFunction { id: usize, len: usize },
    InvalidSignature { id: usize, len: usize },
    UnexpectedStackDepth { wanted: usize, got: usize},
    UnexpectedType { wanted: TypeValue, got: TypeValue },
    UnexpectedReturnValue { wanted: TypeValue, got: TypeValue},
    UnexpectedReturnLength { got: usize },
    OpcodeError(opcode::Error),
    StackError(stack::Error),
    Leb128Error(wasm_leb128::Error),

}

impl From<opcode::Error> for Error {
    fn from(other: opcode::Error) -> Error {
        Error::OpcodeError(other)
    }
}

impl From<stack::Error> for Error {
    fn from(other: stack::Error) -> Error {
        Error::StackError(other)
    }
}

impl From<wasm_leb128::Error> for Error {
    fn from(other: wasm_leb128::Error) -> Error {
        Error::Leb128Error(other)
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
    Void = -0x40,
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

impl From<TypeValue> for i8 {
    fn from(other: TypeValue) -> Self {
        other as i8
    }
}


// pub struct Machine<'a> {
//     code: &'a [u8],
//     stack: &'a mut [u8],
//     memory: &'a mut [u8],
//     locals: &'a mut [i32],
//     globals: &'a mut [i32],
//     functions: &'a mut [i32],
//     sp: usize,
//     pc: usize,
// }

// impl<'a> Machine<'a> {
//     pub fn new(code: &'a [u8], stack: &'a mut[u8], memory: &'a mut[u8], locals: &'a mut[i32], globals: &'a mut[i32], functions: &'a mut[i32]) -> Self {
//         let sp = stack.len();
//         Machine { code: code, stack: stack, memory: memory, locals: locals, globals: globals, functions: functions, sp: sp, pc: 0 }
//     }    
//     pub fn get_function(&self, i: usize) -> i32 {
//         self.functions[i as usize]
//     }

//     pub fn set_function(&mut self, i: usize, addr: i32) {
//         self.functions[i as usize] = addr;
//     }

//     pub fn get_memory_i32(&self, offset: usize) -> i32 {
//         LittleEndian::read_i32(&self.memory[offset..])
//     }

//     pub fn set_memory_i32(&mut self, offset: usize, value: i32) {
//         LittleEndian::write_i32(&mut self.memory[offset..], value);
//     }

//     pub fn get_memory_i16(&self, offset: usize) -> i16 {
//         LittleEndian::read_i16(&self.memory[offset..])
//     }

//     pub fn set_memory_i16(&mut self, offset: usize, value: i16) {
//         LittleEndian::write_i16(&mut self.memory[offset..], value);
//     }

//     pub fn get_memory_i8(&self, offset: usize) -> i8 {
//         self.memory[offset] as i8
//     }

//     pub fn set_memory_i8(&mut self, offset: usize, value: i8) {
//         self.memory[offset] = value as u8;
//     }

//     pub fn get_local(&self, index: usize) -> i32 {
//         self.locals[index]
//     }

//     pub fn set_local(&mut self, index: usize, value: i32) {
//         self.locals[index] = value;
//     }

//     pub fn get_global(&self, index: usize) -> i32 {
//         self.globals[index]
//     }

//     pub fn set_global(&mut self, index: usize, value: i32) {
//         self.globals[index] = value;
//     }

//     pub fn read_var_i32(&mut self) -> Result<i32, Error> {
//         let (v, n) = try!(read_i32(&self.code[self.pc..]));
//         self.pc += n;
//         Ok(v)
//     }

//     pub fn read_var_u32(&mut self) -> Result<u32, Error> {
//         let (v, n) = try!(read_u32(&self.code[self.pc..]));
//         self.pc += n;
//         Ok(v)
//     }

//     fn push_i32(&mut self, value: i32) {
//         assert!(self.sp >= 4);
//         self.sp -= 4; 
//         LittleEndian::write_i32(&mut self.stack[self.sp..], value);
//     }

//     fn pop_i32(&mut self) -> i32 {
//         assert!(self.sp <= self.stack.len() - 4);
//         let v = LittleEndian::read_i32(&self.stack[self.sp..]);
//         self.sp += 4;
//         v
//     }

//     fn top_i32(&self) -> i32 {
//         LittleEndian::read_i32(&self.stack[self.sp..])
//     }

//     // Return error or the next PC
//     pub fn step(&mut self) -> Result<(), Error> {
//         use self::opcode::*;
//         let op = self.code[self.pc];
//         self.pc += 1;
//         match op {
//             // Control Flow Operators
//             UNREACHABLE => return Err(Error::Unreachable),
//             NOP => {},
//             BLOCK => {},
//             LOOP => {},
//             // IF => BR_UNLESS ELSE + 1
//             IF => {},
//             // IF => END + 1
//             ELSE => {},
//             // 
//             END => return Err(Error::End),
//             BR => {},
//             BR_IF => {},
//             BR_TABLE => unimplemented!(),
//             RETURN => {                
//                 self.pc = self.pop_i32() as usize;
//             },

//             // Call Operators
//             CALL => {
//                 let i = try!(self.read_var_u32());
//                 let dst = self.get_function(i as usize);
//                 let pc = self.pc as i32;
//                 self.push_i32(pc);
//                 self.pc = dst as usize;                
//             },
//             CALL_INDIRECT => unimplemented!(),

//             // Parametric Operators
//             DROP => {
//                 self.pop_i32();
//             },            
//             SELECT => {
//                 let cond = self.pop_i32();
//                 let f = self.pop_i32();
//                 let t = self.pop_i32();
//                 self.push_i32(if cond != 0 { t } else { f });
//             },

//             // Variable Access
//             GET_LOCAL => {
//                 let index = try!(self.read_var_u32());
//                 let value = self.locals[index as usize];
//                 self.push_i32(value);
//             },
//             SET_LOCAL => {
//                 let index = try!(self.read_var_u32());
//                 let value = self.pop_i32();
//                 self.locals[index as usize] = value;                
//             },  
//             TEE_LOCAL => {
//                 let index = try!(self.read_var_u32());                
//                 self.locals[index as usize] = self.top_i32();                
//             },  
//             GET_GLOBAL => {
//                 let index = try!(self.read_var_u32());
//                 let value = self.globals[index as usize];
//                 self.push_i32(value);
//             },
//             SET_GLOBAL => {
//                 let index = try!(self.read_var_u32());
//                 let value = self.pop_i32();
//                 self.globals[index as usize] = value;                
//             },  

//             // Memory Operators
//             I32_LOAD => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value = self.get_memory_i32(offset);
//                 self.push_i32(value);
//             },
//             I32_LOAD8_S => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value: i32 = self.get_memory_i8(offset) as i32;
//                 self.push_i32(value);
//             },
//             I32_LOAD8_U => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value: u32 = self.get_memory_i8(offset) as u32;
//                 self.push_i32(value as i32);
//             },
//             I32_LOAD16_S => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value: i32 = self.get_memory_i16(offset) as i32;
//                 self.push_i32(value);
//             },
//             I32_LOAD16_U => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value: u32 = self.get_memory_i16(offset) as u32;
//                 self.push_i32(value as i32);
//             },
//             I32_STORE => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value = self.pop_i32();
//                 self.set_memory_i32(offset, value);
//             },
//             I32_STORE8 => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value = self.pop_i32();
//                 self.set_memory_i8(offset, value as i8);
//             },
//             I32_STORE16 => {
//                 let _flags = try!(self.read_var_u32());
//                 let offset = self.pop_i32() as usize + try!(self.read_var_u32()) as usize;
//                 let value = self.pop_i32();
//                 self.set_memory_i16(offset, value as i16);
//             },            
//             // Constants
//             I32_CONST => {       
//                 let r = try!(self.read_var_i32());
//                 self.push_i32(r);
//             },

//             // Comparison Operators
//             I32_EQZ => {
//                 let v = self.pop_i32();
//                 self.push_i32(if v == 1 { 1 } else { 0 });
//             },
//             I32_EQ => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(if l == r { 1 } else { 0 });
//             },
//             I32_NE => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(if l != r { 1 } else { 0 });
//             },
//             I32_LT_S => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(if l < r { 1 } else { 0 });
//             },
//             I32_LT_U => {
//                 let r = self.pop_i32() as u32;
//                 let l = self.pop_i32() as u32;
//                 self.push_i32(if l < r { 1 } else { 0 });
//             },
//             I32_GT_S => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(if l >r { 1 } else { 0 });
//             },
//             I32_GT_U => {
//                 let r = self.pop_i32() as u32;
//                 let l = self.pop_i32() as u32;
//                 self.push_i32(if l >r { 1 } else { 0 });
//             },               
//             I32_LE_S => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(if l <= r { 1 } else { 0 });
//             },
//             I32_LE_U => {
//                 let r = self.pop_i32() as u32;
//                 let l = self.pop_i32() as u32;
//                 self.push_i32(if l <= r { 1 } else { 0 });
//             },
//             I32_GE_S => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(if l >= r { 1 } else { 0 });
//             },
//             I32_GE_U => {
//                 let r = self.pop_i32() as u32;
//                 let l = self.pop_i32() as u32;
//                 self.push_i32(if l >= r { 1 } else { 0 });
//             },               

//             // Numeric Operators
//             I32_CLZ => {
//                 let v = self.pop_i32();
//                 self.push_i32(v.leading_zeros() as i32);
//             },
//             I32_CTZ => {
//                 let v = self.pop_i32();
//                 self.push_i32(v.trailing_zeros() as i32);
//             },
//             I32_POPCNT => {
//                 let v = self.pop_i32();
//                 self.push_i32(v.count_ones() as i32);
//             },
//             I32_ADD => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l + r);        
//             },
//             I32_SUB => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l - r);
//             },                        
//             I32_MUL => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l * r);
//             },
//             I32_DIV_S => unimplemented!(),
//             I32_DIV_U => unimplemented!(),
//             I32_REM_S => unimplemented!(),
//             I32_REM_U => unimplemented!(),
//             I32_AND => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l & r);
//             },
//             I32_OR => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l | r);
//             },
//             I32_XOR => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l ^ r);
//             },                        
//             I32_SHL => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l << r);                
//             },
//             I32_SHR_S => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l >> r);                
//             },
//             I32_SHR_U => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32() as u32;
//                 self.push_i32((l >> r) as i32);
//             },
//             I32_ROTL => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l.rotate_left(r as u32));
//             },
//             I32_ROTR => {
//                 let r = self.pop_i32();
//                 let l = self.pop_i32();
//                 self.push_i32(l.rotate_right(r as u32));
//             }

//             _ => return Err(Error::Unimplemented),
//         }
//         Ok(())
//     }

//     pub fn run(&mut self) -> Result<(), Error> {
//         loop {
//             match self.step() {
//                 Ok(()) => {},
//                 Err(Error::End) => return Ok(()),
//                 Err(e) => return Err(e),
//             }
//         }
//     }

// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use super::opcode::*;
//     use writer::*;

//     fn with_machine<F: FnOnce(&mut Machine)>(code: &[u8], f: F) {
//         let mut stack = [0u8; 16];    
//         let mut memory = [0u8; 16];
//         let mut locals = [0i32; 16];
//         let mut globals = [0i32; 16];
//         let mut functions = [0i32; 16];
//         let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals, &mut functions);
//         f(&mut machine)
//     }

//     #[test]
//     fn test_unreachable() {
//         let code: [u8;1] = [UNREACHABLE];
//         with_machine(&code[..], |m| {
//             assert_eq!(m.step(), Err(Error::Unreachable));
//         });
//     }


//     #[test]
//     fn test_end() {
//         let code: [u8;1] = [END];
//         with_machine(&code[..], |m| {
//             assert_eq!(m.step(), Err(Error::End));
//         });
//     }


//     #[test]
//     fn test_nop() {
//         let code: [u8; 3] = [NOP, NOP, END];
//         with_machine(&code[..], |m| {
//             assert_eq!(m.step(), Ok(()));
//             assert_eq!(m.step(), Ok(()));
//             assert_eq!(m.step(), Err(Error::End));
//         });        
//     }

//     #[test]
//     fn test_i32_const() {
//         let mut buf = [0u8; 64];
//         let mut w = Writer::new(&mut buf);
//         w.write_u8(I32_CONST).unwrap();
//         w.write_var_i32(0x12345678).unwrap();
//         w.write_u8(END).unwrap();
//         with_machine(&w, |m| {            
//             assert_eq!(m.run(), Ok(()));            
//             assert_eq!(m.pop_i32(), 0x12345678);
//         });        
//     }    

//     #[test]
//     fn test_get_global() {
//         let mut buf = [0u8; 64];
//         let mut w = Writer::new(&mut buf);
//         w.write_u8(GET_GLOBAL).unwrap();
//         w.write_var_u32(0).unwrap();
//         w.write_u8(END).unwrap();
//         with_machine(&w, |m| {            
//             m.set_global(0, 0x12345678);
//             assert_eq!(m.run(), Ok(()));            
//             assert_eq!(m.pop_i32(), 0x12345678);
//         });        
//     }        

//     #[test]
//     fn test_set_global() {
//         let mut buf = [0u8; 64];
//         let mut w = Writer::new(&mut buf);
//         w.write_u8(SET_GLOBAL).unwrap();
//         w.write_var_u32(0).unwrap();
//         w.write_u8(END).unwrap();
//         with_machine(&w, |m| {
//             m.push_i32(0x12345678);            
//             assert_eq!(m.run(), Ok(()));            
//             assert_eq!(m.get_global(0), 0x12345678);
//         });        
//     }        


//     #[test]
//     fn test_get_local() {
//         let mut buf = [0u8; 64];
//         let mut w = Writer::new(&mut buf);
//         w.write_u8(GET_LOCAL).unwrap();
//         w.write_var_u32(0).unwrap();
//         w.write_u8(END).unwrap();
//         with_machine(&w, |m| {            
//             m.set_local(0, 0x12345678);
//             assert_eq!(m.run(), Ok(()));            
//             assert_eq!(m.pop_i32(), 0x12345678);
//         });        
//     }        

//     #[test]
//     fn test_set_local() {
//         let mut buf = [0u8; 64];
//         let mut w = Writer::new(&mut buf);
//         w.write_u8(SET_LOCAL).unwrap();
//         w.write_var_u32(0).unwrap();
//         w.write_u8(END).unwrap();
//         with_machine(&w, |m| {
//             m.push_i32(0x12345678);            
//             assert_eq!(m.run(), Ok(()));            
//             assert_eq!(m.get_local(0), 0x12345678);
//         });        
//     }        

//     #[test]
//     fn test_tee_local() {
//         let mut buf = [0u8; 64];
//         let mut w = Writer::new(&mut buf);
//         w.write_u8(TEE_LOCAL).unwrap();
//         w.write_var_u32(0).unwrap();
//         w.write_u8(END).unwrap();
//         with_machine(&w, |m| {
//             m.push_i32(0x12345678);            
//             assert_eq!(m.run(), Ok(()));            
//             assert_eq!(m.get_local(0), 0x12345678);
//             assert_eq!(m.pop_i32(), 0x12345678);
//         });        
//     }   
// }
