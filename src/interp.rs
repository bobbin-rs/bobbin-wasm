use {Error, Value};

use reader::Reader;
use writer::Writer;
use stack::Stack;
use opcode::*;

pub struct Config {
}

impl Default for Config {
    fn default() -> Config {
        Config {}
    }
}


pub struct Interp<'a, 'c> {
    cfg: Config,
    value_stack: Stack<'a, Value>,
    call_stack: Stack<'a, u32>,
    code: Reader<'c>,
    count: usize,
}

impl<'a, 'c> Interp<'a, 'c> {
    pub fn new(cfg: Config, code: &'c [u8], buf: &'a mut [u8]) -> Self {
        let mut w = Writer::new(buf);
        let value_stack = w.alloc_stack(64);
        let call_stack = w.alloc_stack(64);
        let code = Reader::new(code);
        let count = 0;
        Interp { cfg, value_stack, call_stack, code, count }
    }

    pub fn pc(&self) -> usize {
        self.code.pos()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn jump(&mut self, offset: u32) {
        self.code.set_pos(offset as usize);
    }

    pub fn pop_value(&mut self) -> Result<Value, Error> {
        Ok(self.value_stack.pop()?)
    }

    pub fn run_count(&mut self, count: usize) -> Result<(), Error> {   
        self.count = 0;      
        while self.pc() < self.code.len() && self.count < count {
            info!("{:08x}", self.pc());
            let op = self.code.read_u8()?;
            match op {
                NOP => {},
                UNREACHABLE => return Err(Error::Unreachable),
                BR => {
                    let offset = self.code.read_u32()?;                    
                    self.code.set_pos(offset as usize);
                },

                I32_CONST => {
                    let value = Value(self.code.read_i32()?);
                    self.value_stack.push(value)?;
                },

                _ => return Err(Error::Unimplemented),
            }
            self.count += 1;
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use loader::LoaderWrite;

    fn with_interp<T, F: FnOnce(Interp) -> Result<T, Error>>(mut w: Writer, f: F) -> Result<T, Error> {
        let cfg = Config::default();
        let mut buf = [0u8; 4096];
        f(Interp::new(cfg, w.split(), &mut buf[..]))
    }

    fn with_writer<T, F: FnOnce(Writer)-> Result<T, Error>>(f: F) -> Result<T, Error> {
        let mut buf = [0u8; 4096];
        f(Writer::new(&mut buf))
    }

    #[test]
    fn test_nop() {
        with_writer(|mut w| {
            w.write_opcode(NOP)?;
            with_interp(w, |mut interp| {                
                interp.run_count(1)?;
                assert_eq!(interp.pc(), 1);
                assert_eq!(interp.count(), 1);

                Ok(())
            })?;
            Ok(())
        }).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_unreachable() {
        with_writer(|mut w| {
            w.write_opcode(UNREACHABLE)?;
            with_interp(w, |mut interp| {                
                interp.run_count(1)?;
                assert_eq!(interp.pc(), 1);
                assert_eq!(interp.count(), 1);

                Ok(())
            })?;
            Ok(())
        }).unwrap();
    }


    #[test]
    fn test_i32_const() {
        with_writer(|mut w| {
            w.write_opcode(I32_CONST)?;
            w.write_u32(0x1234)?;
            with_interp(w, |mut interp| {                
                interp.run_count(1)?;
                let top = interp.value_stack.pop()?;
                assert_eq!(top, Value(0x1234));
                Ok(())
            })?;
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_br() {
        with_writer(|mut w| {
            w.write_opcode(BR)?;
            w.write_u32(0x4)?;
            w.write_opcode(NOP)?;
            w.write_opcode(NOP)?;
            with_interp(w, |mut interp| {                
                interp.run_count(1)?;
                assert_eq!(interp.pc(), 0x4);
                Ok(())
            })?;
            Ok(())
        }).unwrap();
    }    

}