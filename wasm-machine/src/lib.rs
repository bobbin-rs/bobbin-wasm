#![allow(dead_code)]
#![no_std]

pub mod opcode;

#[derive(Debug, PartialEq)]
pub enum Error {
    Unimplemented(usize),
    End(usize),
}

pub struct Machine<'a> {
    code: &'a [u8],
    stack: &'a mut [u32],
    memory: &'a mut [u32],
    locals: &'a mut [u32],
    globals: &'a mut [u32],
    sp: usize,
    pc: usize,
}

impl<'a> Machine<'a> {
    pub fn new(code: &'a [u8], stack: &'a mut[u32], memory: &'a mut[u32], locals: &'a mut[u32], globals: &'a mut[u32]) -> Self {
        Machine { code: code, stack: stack, memory: memory, locals: locals, globals: globals, sp: 0, pc: 0 }
    }

    // Return error or the next PC
    pub fn step(&mut self) -> Result<usize, Error> {
        use self::opcode::*;
        let op = self.code[self.pc];        
        match op {
            NOP => { self.pc += 1; Ok(self.pc) } ,
            END => Err(Error::End(self.pc)),
            _ => Err(Error::Unimplemented(self.pc)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::opcode::*;

    #[test]
    fn test_end() {
        let mut locals = [0u32; 16];
        let mut globals = [0u32; 16];
        let mut memory = [0u32; 16];
        let mut stack = [0u32; 16];
        let code: [u8;1] = [END];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);
        assert_eq!(machine.step(), Err(Error::End(0)));
    }

    #[test]
    fn test_nop() {
        let mut locals = [0u32; 16];
        let mut globals = [0u32; 16];
        let mut memory = [0u32; 16];
        let mut stack = [0u32; 16];
        let code: [u8; 3] = [NOP, NOP, END];
        let mut machine = Machine::new(&code, &mut stack, &mut memory, &mut locals, &mut globals);
        assert_eq!(machine.step(), Ok(1));
        assert_eq!(machine.step(), Ok(2));
        assert_eq!(machine.step(), Err(Error::End(2)));
    }
}
