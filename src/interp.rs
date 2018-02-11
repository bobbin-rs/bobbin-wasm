use {Error, Value};

use byteorder::{ByteOrder, LittleEndian};

use reader::Reader;
use writer::Writer;
use stack::Stack;
use opcode::*;

pub type InterpResult<T> = Result<T, Error>;

pub struct Config {
    mem_size: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            mem_size: 64,
        }
    }
}


pub struct Interp<'a, 'c> {
    cfg: Config,
    value_stack: Stack<'a, Value>,
    call_stack: Stack<'a, u32>,
    mem: &'a mut [u8],
    code: Reader<'c>,
    count: usize,
}

impl<'a, 'c> Interp<'a, 'c> {
    pub fn new(cfg: Config, code: &'c [u8], buf: &'a mut [u8]) -> Self {
        let mut w = Writer::new(buf);
        let value_stack = w.alloc_stack(64);
        let call_stack = w.alloc_stack(64);
        let mem = w.alloc_slice(cfg.mem_size);
        let code = Reader::new(code);
        let count = 0;
        Interp { cfg, value_stack, call_stack, mem, code, count }
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

    fn check_addr(&self, addr: u32) -> InterpResult<()> {
        if addr as usize <= self.mem.len() {
            Ok(())
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn get_mem_i8(&self, addr: u32) -> InterpResult<i8> {
        Ok(self.mem[addr as usize] as i8)
    }

    pub fn get_mem_u8(&self, addr: u32) -> InterpResult<u8> {
        Ok(self.mem[addr as usize])
    }

    pub fn get_mem_i16(&self, addr: u32) -> InterpResult<i16> {
        Ok(LittleEndian::read_i16(&self.mem[addr as usize..]))
    }

    pub fn get_mem_i32(&self, addr: u32) -> InterpResult<i32> {
        Ok(LittleEndian::read_i32(&self.mem[addr as usize..]))
    }
    
    pub fn get_mem_u16(&self, addr: u32) -> InterpResult<u16> {
        Ok(LittleEndian::read_u16(&self.mem[addr as usize..]))
    }

    pub fn get_mem_u32(&self, addr: u32) -> InterpResult<u32> {
        Ok(LittleEndian::read_u32(&self.mem[addr as usize..]))
    }

    pub fn set_mem_i8(&mut self, addr: u32, value: i8) -> InterpResult<()> {
        Ok(self.mem[addr as usize] = value as u8)
    }

    pub fn set_mem_i16(&mut self, addr: u32, value: i16) -> InterpResult<()>  {
        Ok(LittleEndian::write_i16(&mut self.mem[addr as usize..], value))
    }

    pub fn set_mem_i32(&mut self, addr: u32, value: i32) -> InterpResult<()>  {
        Ok(LittleEndian::write_i32(&mut self.mem[addr as usize..], value))
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
                BR_IF => {
                    let offset = self.code.read_u32()?;
                    let val = self.pop()?;
                    if val != 0 {             
                        self.code.set_pos(offset as usize);
                    }
                },                
                DROP => {
                    self.value_stack.pop()?;
                }
                I32_CONST => {
                    let value = Value(self.code.read_i32()?);
                    self.value_stack.push(value)?;
                },

                // I32 load
                0x28 ... 0x30 => {
                    let _flags = self.code.read_u32()?;
                    let offset = self.code.read_u32()?;
                    let base: u32 = self.pop()? as u32;
                    let addr = (offset + base) as u32;
                    self.check_addr(addr)?;

                    let res = match op {
                        I32_LOAD => {
                            self.get_mem_i32(addr)?
                        },
                        I32_LOAD8_S => {
                            self.get_mem_i8(addr)? as i8 as i32
                        },
                        I32_LOAD8_U => {
                            self.get_mem_u8(addr)? as u8 as i32
                        },
                        I32_LOAD16_S => {
                            self.get_mem_i16(addr)? as i16 as i32
                        },
                        I32_LOAD16_U => {
                            self.get_mem_u16(addr)? as u16 as i32
                        }                                               
                        _ => unimplemented!(),
                    };
                    self.push(res)?;
                },
                // I32 store
                0x36 | 0x38 | 0x3a | 0x3b => {
                    let _flags = self.code.read_u32()?;
                    let offset = self.code.read_u32()?;
                    let base: u32 = self.pop()? as u32;
                    let value: i32 = self.pop()?;
                    let addr = (offset + base) as u32;
                    self.check_addr(addr)?;

                    match op {
                        I32_STORE => self.set_mem_i32(addr, value)?,
                        I32_STORE8 => self.set_mem_i8(addr, value as i8)?,
                        I32_STORE16 => self.set_mem_i16(addr, value as i16)?,
                        _ => unimplemented!(),
                    }
                },
                // I32 cmpops
                0x46 ... 0x50 => {
                    let (rhs, lhs): (i32, i32) = (self.pop()?, self.pop()?);
                    let res = match op {
                        I32_EQ => lhs == rhs,
                        I32_NE => lhs != rhs,
                        I32_LT_U => lhs < rhs,
                        I32_LT_S => (lhs as u32) < (rhs as u32),
                        I32_GT_U => lhs > rhs,
                        I32_GT_S => (lhs as u32) > (rhs as u32),
                        I32_LE_U => lhs <= rhs,
                        I32_LE_S => (lhs as u32) <= (rhs as u32),
                        I32_GE_U => lhs >= rhs,                        
                        I32_GE_S => (lhs as u32) >= (rhs as u32),
                        _ => return Err(Error::Unimplemented),
                    };
                    self.push(if res { 1 } else { 0 })?;
                },
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
                INTERP_ALLOCA => {
                    let count = self.code.read_u32()?;
                    for _ in 0..count {
                        self.push(0)?;
                    }
                },
                INTERP_BR_UNLESS => {
                    let offset = self.code.read_u32()?;
                    let val = self.pop()?;
                    if val == 0 {             
                        self.code.set_pos(offset as usize);
                    }
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
        { $($name:ident: { $w:ident : $w_blk:block, $i:ident : $i_blk:block }),* }  => {
            $(
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
            )*
        }
    }

    macro_rules! test_i32_unop {
        { $($name:ident($op:expr, $val:expr, $ret:expr);)* } => {
            $(     
                #[test]
                fn $name() {                
                    with_writer(|mut w| {
                        w.write_opcode($op)?;
                        with_interp(w, |mut i| {
                            i.push($val)?;
                            i.run()?;
                            assert_eq!(i.pop()?, $ret);
                            Ok(())
                        })?;
                        Ok(())
                    }).unwrap();            
                }                
            )*
        }
    }

    macro_rules! test_i32_binop {
        { $($name:ident($op:expr, $lhs:expr, $rhs:expr, $ret:expr);)* } => {
            $(     
                #[test]
                fn $name() {                
                    with_writer(|mut w| {
                        w.write_opcode($op)?;
                        with_interp(w, |mut i| {
                            i.push($lhs)?;
                            i.push($rhs)?;
                            i.run()?;
                            assert_eq!(i.pop()?, $ret);
                            Ok(())
                        })?;
                        Ok(())
                    }).unwrap();            
                }                
            )*
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

    interp_test! {
        test_nop: {
            w : {
                w.write_opcode(NOP)?;
            }, 
            i: {
                i.run()?;
                assert!(i.pc() == 1);
            }
        },
        test_i32_const : {
            w : {
                w.write_opcode(I32_CONST)?;
                w.write_u32(0x1234)?;
            }, 
            i: {
                i.run()?;
                assert_eq!(i.pop()?, 0x1234);
            }
        },
        test_br : {
            w : {
                w.write_opcode(BR)?;
                w.write_u32(6)?;
                w.write_opcode(UNREACHABLE)?;
            }, 
            i: {
                i.run()?;
                assert_eq!(i.pc(), 6);
            }
        },
        test_br_if_0 : {
            w : {
                w.write_opcode(I32_CONST)?; // 0x0
                w.write_u32(0)?; // 0x1
                w.write_opcode(BR_IF)?; // 0x5
                w.write_u32(0x0f)?; // 0x6
                w.write_opcode(BR)?; // 0x0a
                w.write_u32(0x10)?; // 0x0b
                w.write_opcode(UNREACHABLE)?; // 0x0f
            }, 
            i: {                
                i.run()?;
            }
        },             
        test_br_if_1 : {
            w : {
                w.write_opcode(I32_CONST)?;
                w.write_u32(1)?;
                w.write_opcode(BR_IF)?;
                w.write_u32(11)?;
                w.write_opcode(UNREACHABLE)?;
            }, 
            i: {                
                i.run()?;
            }
        },
        test_br_unless_0 : {
            w : {
                w.write_opcode(I32_CONST)?;
                w.write_u32(0)?;
                w.write_opcode(INTERP_BR_UNLESS)?;
                w.write_u32(11)?;
                w.write_opcode(UNREACHABLE)?;
            }, 
            i: {                
                i.run()?;
            }
        },
        test_br_unless_1 : {
            w : {
                w.write_opcode(I32_CONST)?; // 0x0
                w.write_u32(1)?; // 0x1
                w.write_opcode(INTERP_BR_UNLESS)?; // 0x5
                w.write_u32(0x0f)?; // 0x6
                w.write_opcode(BR)?; // 0x0a
                w.write_u32(0x10)?; // 0x0b
                w.write_opcode(UNREACHABLE)?; // 0x0f
            }, 
            i: {                
                i.run()?;
            }
        },                        
        test_i32_load : {
            w : {
                w.write_opcode(I32_LOAD)?;
                w.write_u32(0x0)?;
                w.write_u32(0x1)?;
            },
            i: {
                i.set_mem_i32(0x11, 0x2345)?;
                i.push(0x10)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x2345);
            }
        },
        test_i32_store : {
            w : {
                w.write_opcode(I32_STORE)?;
                w.write_u32(0x0)?;
                w.write_u32(0x1)?;
            },
            i: {
                i.push(0x1234)?;
                i.push(0x10)?;
                i.run()?;
                assert_eq!(i.value_stack.len(), 0);
                assert_eq!(i.get_mem_u32(0x11)?, 0x1234);
            }
        },
        test_alloca : {
            w : {
                w.write_opcode(INTERP_ALLOCA)?;
                w.write_u32(0x2)?;
            },
            i: {
                i.run()?;
                assert_eq!(i.pop()?, 0);
                assert_eq!(i.pop()?, 0);
                assert_eq!(i.value_stack.len(), 0);
            }
        }               
    }

    test_i32_unop! {
        test_i32_eqz_0(I32_EQZ, 0, 1);
        test_i32_eqz_1(I32_EQZ, 1, 0);
    }

    test_i32_binop! {
        test_i32_eq_0_1(I32_EQ, 0, 1, 0);
        test_i32_eq_1_1(I32_EQ, 1, 1, 1);
        test_i32_add_1_2(I32_ADD, 1, 2, 3);
        test_i32_sub_3_2(I32_SUB, 3, 2, 1);
        test_i32_mul_3_2(I32_MUL, 3, 2, 6);
    }
}