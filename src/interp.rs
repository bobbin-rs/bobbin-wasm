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

    pub fn push(&mut self, value: i32) -> Result<(), Error> {
        Ok(self.value_stack.push(Value(value))?)
    }

    pub fn pop(&mut self) -> Result<i32, Error> {
        Ok(self.value_stack.pop()?.0)
    }

    pub fn run(&mut self) -> Result<(), Error> {   
        self.run_count(0)
    }

    pub fn run_count(&mut self, count: usize) -> Result<(), Error> {   
        self.count = 0;      
        while self.pc() < self.code.len() && (count == 0 || self.count < count) {
            info!("{:08x}", self.pc());
            let op = self.code.read_u8()?;
            match op {
                NOP => {},
                UNREACHABLE => return Err(Error::Unreachable),
                BR => {
                    let offset = self.code.read_u32()?;                    
                    self.code.set_pos(offset as usize);
                },
                DROP => {
                    self.value_stack.pop()?;
                }
                I32_CONST => {
                    let value = Value(self.code.read_i32()?);
                    self.value_stack.push(value)?;
                },

                // // I32 load
                // 0x28 ... 0x30 => {
                //     let _flags = self.code.read_u32()?;
                //     let offset = self.code.read_u32()?;
                //     let base: u32 = self.pop()?;
                //     let addr = offset + base;

                //     let res = match op {
                //         I32_LOAD => {
                //             self.get_memory_u32(addr)?
                //         },
                //         I32_LOAD8_S => {
                //             self.get_memory_u8(addr)? as i8 as u32
                //         },
                //         I32_LOAD8_U => {
                //             self.get_memory_u8(addr)? as i8 as u32
                //         },
                //         I32_LOAD16_S => {
                //             self.get_memory_u16(addr)? as i16 as u32
                //         },
                //         I32_LOAD16_U => {
                //             self.get_memory_u16(addr)? as u16 as u32
                //         }                                               
                //         _ => unimplemented!(),
                //     };
                //     self.push(res)?;
                // },
                // // I32 store
                // 0x36 | 0x38 | 0x3a | 0x3b => {
                //     let _flags = self.code.read_u32()?;
                //     let offset = self.code.read_u32()?;
                //     let base: u32 = self.pop()?;
                //     let value: u32 = self.pop()?;
                //     let addr = offset + base;
                //     match op {
                //         I32_STORE => self.set_memory_u32(addr, value)?,
                //         I32_STORE8 => self.set_memory_u8(addr, value as u8)?,
                //         I32_STORE16 => self.set_memory_u16(addr, value as u16)?,
                //         _ => unimplemented!(),
                //     }
                // },
                // // I32 cmpops
                // 0x46 ... 0x50 => {
                //     let (rhs, lhs): (u32, u32) = (self.pop()?, self.pop()?);
                //     let res = match op {
                //         I32_EQ => lhs == rhs,
                //         I32_NE => lhs != rhs,
                //         I32_LT_S => (lhs as i32) < (rhs as i32),
                //         I32_LT_U => lhs < rhs,
                //         I32_GT_S => (lhs as i32) > (rhs as i32),
                //         I32_GT_U => lhs > rhs,
                //         I32_LE_S => (lhs as i32) <= (rhs as i32),
                //         I32_LE_U => lhs <= rhs,
                //         I32_GE_S => (lhs as i32) >= (rhs as i32),
                //         I32_GE_U => lhs >= rhs,                        
                //         _ => return Err(Error::Unimplemented),
                //     };
                //     self.push(if res { 1 } else { 0 })?;
                // }
                // I32 binops
                0x6a ... 0x79 => {
                    let (rhs, lhs): (i32, i32) = (self.pop()?, self.pop()?);
                    info!("lhs: {} rhs: {}", lhs, rhs);
                    let res = match op {
                        I32_ADD => lhs.wrapping_add(rhs),
                        I32_SUB => lhs.wrapping_sub(rhs),
                        I32_MUL => lhs.wrapping_mul(rhs),
                        I32_DIV_S => lhs / rhs,
                        I32_DIV_U => ((lhs as u32) / (rhs as u32)) as i32,
                        I32_REM_S => lhs % rhs,
                        I32_REM_U => ((lhs as u32) % (rhs as u32)) as i32,
                        I32_AND => lhs & rhs,
                        I32_OR => lhs | rhs,
                        I32_XOR => lhs ^ rhs,
                        I32_SHL => lhs << rhs,
                        I32_SHR_S => lhs >> rhs,
                        I32_SHR_U => ((lhs as u32) >> rhs) as i32,
                        I32_ROTL => lhs.rotate_left(rhs as u32),
                        I32_ROTR => lhs.rotate_right(rhs as u32),
                        _ => unimplemented!()
                    };
                    info!("res: {}", res);
                    self.push(res)?;
                },
                // I32 unops                
                0x45 | 0x67 ... 0x6a => {
                    let val: i32 = self.pop()?;
                    let res = match op {
                        I32_EQZ => if val == 0 { 1 } else { 0 },
                        I32_CLZ => val.leading_zeros(),
                        I32_CTZ => val.trailing_zeros(),
                        I32_POPCNT => val.count_zeros(),
                        _ => unimplemented!(),
                    };
                    self.push(res as i32)?;
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

    macro_rules! interp_test {
        ($name:ident, $w:ident : $w_blk:block, $i:ident : $i_blk:block) => {
            #[test]
            fn $name() {                
                with_writer(|mut $w| {
                    $w_blk
                    with_interp($w, |mut $i| {
                        $i_blk
                        Ok(())
                    })?;
                    Ok(())
                }).unwrap();            
            }
        }
    }

    fn with_interp<T, F: FnOnce(Interp) -> Result<T, Error>>(mut w: Writer, f: F) -> Result<T, Error> {
        let cfg = Config::default();
        let mut buf = [0u8; 4096];
        f(Interp::new(cfg, w.split(), &mut buf[..]))
    }

    fn with_writer<T, F: FnOnce(Writer)-> Result<T, Error>>(f: F) -> Result<T, Error> {
        let mut buf = [0u8; 4096];
        f(Writer::new(&mut buf))
    }

    interp_test!(test_nop, w : {
        w.write_opcode(NOP)?;
    }, i: {
        i.run()?;
        assert!(i.pc() == 1);
    });



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
    fn test_i32_add() {
        with_writer(|mut w| {
            w.write_opcode(I32_CONST)?;
            w.write_u32(1)?;
            w.write_opcode(I32_CONST)?;
            w.write_u32(2)?;
            w.write_opcode(I32_ADD)?;
            with_interp(w, |mut interp| {                
                interp.run_count(0)?;
                let top = interp.value_stack.pop()?;
                assert_eq!(top, Value(3));
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