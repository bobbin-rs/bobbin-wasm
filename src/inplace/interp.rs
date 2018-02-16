use {Error, Value, SectionType};

use module_inst::{ModuleInst, FuncInst};
use reader::Reader;
use writer::Writer;
use stack::Stack;
use opcode::*;

use core::convert::TryFrom;

pub type InterpResult<T> = Result<T, Error>;

pub struct Config {
    value_stack_size: usize,
    call_stack_size: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            value_stack_size: 64,
            call_stack_size: 64,
        }
    }
}


pub struct Interp<'a> {
    cfg: Config,
    value_stack: Stack<'a, Value>,
    call_stack: Stack<'a, u32>,
}

impl<'a> Interp<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self::new_with_config(Config::default(), buf)
    }

    pub fn new_with_config(cfg: Config, buf: &'a mut [u8]) -> Self {
        let mut w = Writer::new(buf);
        let value_stack = w.alloc_stack(cfg.value_stack_size);
        let call_stack = w.alloc_stack(cfg.call_stack_size);
        Interp { cfg, value_stack, call_stack }
    }

    // Value Stack

    pub fn push(&mut self, value: i32) -> Result<(), Error> {
        Ok(self.value_stack.push(Value(value))?)
    }

    pub fn pop(&mut self) -> Result<i32, Error> {
        Ok(self.value_stack.pop()?.0)
    }

    pub fn stack_len(&self) -> usize {
        self.value_stack.len()
    }

    pub fn run(&mut self, mi: &ModuleInst, func_index: usize) -> Result<(), Error> {
        // let body = mi.module().body(func_index as u32);
        let m = mi.module();
        // let func = m.function(func_index as u32).unwrap();
        // let func_type = m.signature_type(func.signature_type_index).unwrap();
        let code_section = m.section(SectionType::Code).unwrap();
        let code_buf = &code_section.buf;
        let code_buf = unsafe {
            use core::slice;
            let new_len = code_buf.len() + 5;
            let new_ptr = code_buf.as_ptr().offset(-5);
            slice::from_raw_parts(new_ptr, new_len)
        };
        let mut code = Reader::new(&code_buf);

        info!("code section len: {:08x}", code_buf.len());


        let body = m.body(func_index as u32).unwrap();

        // info!("body: size={}", body.buf.len());

        let body_pos = code_buf.as_ptr().offset_to(body.buf.as_ptr()).unwrap() as usize;
        // info!("body pos: {:08x}", body_pos);
        code.set_pos(body_pos);

        // for (i, b) in code_buf.iter().enumerate() {
        //     let here = if i == code.pos() { " <=" } else { "" };
        //     info!("{:04x}: {:02x}{}", i, b, here);
        // }

        let mut _count = 0;

        // Set up locals

        loop {
            if code.pos() >= code.len() {
                info!("V: {} 0x{:08x}: CODE END {:08x}", self.value_stack.len(), code.pos(), code.len());
                break;
            }
            let pos = code.pos();
            let opc = code.read_u8()?;
            let op = Opcode::try_from(opc).unwrap();
            info!("V: {} 0x{:08x}: {}", self.value_stack.len(), pos, op.text);
            match opc {
                NOP => {},
                UNREACHABLE => return Err(Error::Unreachable),
                BR => {
                    let offset = code.read_u32()?;
                    info!("  => {:08x}", offset);
                    code.set_pos(offset as usize);
                },
                BR_IF => {
                    let offset = code.read_u32()?;
                    let val = self.pop()?;
                    if val != 0 {             
                        info!("  => {:08x}", offset);
                        code.set_pos(offset as usize);
                    }
                },
                BR_TABLE => {
                    // BR_TABLE COUNT:u32 TABLE_OFFSET:u32
                    // INTERP_DATA SIZE:u32
                    // [OFFSET:u32 DROP:u32 KEEP:u32]
                    // OFFSET:u32 DROP:u32 KEEP:u32

                    let count = code.read_u32()?;
                    let table_offset = code.read_u32()?;
                    info!("  => count: {} table_offset: {:08x}", count, table_offset);
                    let mut val = self.pop()?;
                    info!("  => value: {}", val);

                    let index = if val < 0 || val > count as i32 {
                        count
                    } else {
                        val as u32
                    };
                    info!("  => index: {}", index);
                    let entry_offset = table_offset + (index * BR_TABLE_ENTRY_SIZE);
                    info!("  => entry_offset: {:08x}", entry_offset);
                    code.set_pos(entry_offset as usize);
                    let dst = code.read_u32()?;
                    let drop = code.read_u32()?;
                    let keep = code.read_u32()?;
                    info!("  => dst: {:08x} drop: {} keep: {}", dst, drop, keep);
                    self.value_stack.drop_keep(drop as usize, keep as usize)?;
                    code.set_pos(dst as usize);
                    info!("  => done");
                },                             
                DROP => {
                    self.value_stack.pop()?;
                },
                CALL => {
                    let id = code.read_u32()?;

                    // Lookup Function
                    // info!("Function index: {}", id);
                    let f = &mi.functions()[id as usize];
                    // info!("FuncInst: {:?}", f);
                    let offset = match f {
                        &FuncInst::Local { type_index: _, function_index } => {
                            // info!("Function Index {}", function_index);
                            // lookup body offset
                            let body = m.body(function_index as u32).unwrap();

                            // info!("body: size={}", body.buf.len());

                            let body_pos = code_buf.as_ptr().offset_to(body.buf.as_ptr()).unwrap() as usize;
                            // info!("body_pos: {:08x}", body_pos);
                            body_pos
                        },
                        _ => {
                            return Err(Error::Unimplemented)
                        }
                    };

                    let pos = code.pos();
                    info!("CALL: {:08x} to {:08x}", pos, offset);

                    self.call_stack.push(pos as u32)?;
                    code.set_pos(offset as usize);
                },
                CALL_INDIRECT => {
                    // func_sig = function_type(func_index)
                    // sig_type = type(sig)
                    // func_type = type(func_sig)
                    // check sig_type == func_type
                    // get body offset
                    // jump

                    let sig = code.read_u32()?;
                    info!("CALL_INDIRECT {}", sig);
                    let sig_type = &mi.types()[sig as usize];
                    info!("   sig_type: {:?}", sig_type);
                    let table_index = self.value_stack.pop()?;
                    info!("   table_index: {:?}", table_index);

                    let table = mi.table(0);
                    if table_index.0 as usize >= table.len() {
                        return Err(Error::UndefinedTableIndex { id: table_index.0 })
                    }

                    let func_index = mi.indirect_function_id(table_index.0 as usize);
                    let func_inst = &mi.functions()[func_index as usize];
                    info!("   func_inst: {:?}", func_inst);
                    match func_inst {
                        &FuncInst::Import { type_index: _, import_index: _ } => {
                            unimplemented!()
                        }
                        &FuncInst::Local { type_index, function_index } => {
                            let func_type = &mi.types()[type_index];
                            // info!("Local Function: {}", function_index);
                            // info!("Sig: {:?} Func: {:?}", sig_type, func_type);

                            if sig_type.parameters != func_type.parameters {
                                return Err(Error::SignatureMismatch)
                            }
                            if sig_type.returns != func_type.returns {
                                return Err(Error::SignatureMismatch)
                            }

                            let body = m.body(function_index as u32).unwrap();
                            let offset = code_buf.as_ptr().offset_to(body.buf.as_ptr()).unwrap() as usize;
                            let pos = code.pos();
                            info!("  => {:08x} to {:08x}", pos, offset);

                            self.call_stack.push(pos as u32)?;
                            code.set_pos(offset as usize);
                        }
                    }
                }
                RETURN => {
                    if self.call_stack.len() == 0 {
                        info!("RETURN");
                        break;
                    }

                    let offset = self.call_stack.pop()?;
                    info!("RETURN: to {:08x}", offset);
                    code.set_pos(offset as usize);
                },
                END => {
                    panic!("unexpected END");
                }
                SELECT => {
                    let cond: i32 = self.pop()?;
                    let _false: i32 = self.pop()?;
                    let _true: i32 = self.pop()?;
                    self.push(if cond != 0 { _true } else { _false })?;
                },                
                I32_CONST => {
                    let value = Value(code.read_i32()?);
                    self.value_stack.push(value)?;
                },
                GET_LOCAL => {
                    let depth: u32 = code.read_u32()?;
                    info!("GET_LOCAL: {} ", depth);
                    let value: i32 = self.value_stack.peek((depth - 1) as usize)?.0;
                    info!("   => {}", value);
                    self.push(value)?;
                },
                SET_LOCAL => {
                    // check: should depth be relative to top of stack at beginning of operation?
                    let depth: u32 = code.read_u32()?;
                    info!("SET_LOCAL: {} ", depth);
                    let value: i32 = self.pop()?;
                    info!("   <= {}", value);
                    self.value_stack.pick((depth - 1) as usize)?.0 = value;
                },
                TEE_LOCAL => {
                    let depth: u32 = code.read_u32()?;
                    let value: i32 = self.pop()?;
                    self.value_stack.peek((depth - 1) as usize)?.0 = value;
                    self.push(value)?;
                },                
                GET_GLOBAL => {
                    let index = code.read_u32()?;
                    let value: i32 = mi.get_global(index)?;
                    self.push(value)?;
                },
                SET_GLOBAL => {
                    let index = code.read_u32()?;
                    let value: i32 = self.pop()?;
                    mi.set_global(index, value)?;
                },
                MEM_GROW => {
                    let pages = self.pop()?;
                    info!("MEM_GROW: {}", pages);
                    let ret = mi.memory_inst().grow_memory(pages);
                    info!("  => {}", ret);
                    self.push(ret)?;
                },
                MEM_SIZE => {
                    let size = mi.memory_inst().num_pages();
                    self.push(size as i32)?;
                }
                // I32 load
                0x28 ... 0x30 => {
                    let _flags = code.read_u32()?;
                    let offset = code.read_u32()?;
                    let base: u32 = self.pop()? as u32;
                    let addr = (offset + base) as usize;
                    let mem = mi.memory_inst();

                    let res = match opc {
                        I32_LOAD => {
                            mem.load(addr)?
                        },
                        I32_LOAD8_S => {
                            mem.load8_s(addr)?
                        },
                        I32_LOAD8_U => {
                            mem.load8_u(addr)?
                        },
                        I32_LOAD16_S => {
                            mem.load16_s(addr)?
                        },
                        I32_LOAD16_U => {
                            mem.load16_u(addr)?
                        }                                               
                        _ => unimplemented!(),
                    };
                    self.push(res)?;
                },
                // I32 store
                0x36 | 0x38 | 0x3a | 0x3b => {
                    let _flags = code.read_u32()?;
                    let offset = code.read_u32()?;
                    let base: u32 = self.pop()? as u32;
                    let value: i32 = self.pop()?;
                    let addr = (offset + base) as usize;
                    let mem = mi.memory_inst();

                    match opc {
                        I32_STORE => mem.store(addr, value)?,
                        I32_STORE8 => mem.store8(addr, value)?,
                        I32_STORE16 => mem.store16(addr, value)?,
                        _ => unimplemented!(),
                    }
                },
                // I32 cmpops
                0x46 ... 0x50 => {
                    let (rhs, lhs): (i32, i32) = (self.pop()?, self.pop()?);
                    info!("lhs: {} rhs: {}", lhs, rhs);
                    let res = match opc {
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
                    info!("res: {}", res);
                    self.push(if res { 1 } else { 0 })?;
                },
                // I32 binops
                0x6a ... 0x79 => {
                    let (rhs, lhs): (i32, i32) = (self.pop()?, self.pop()?);
                    info!("lhs: {} rhs: {}", lhs, rhs);
                    let res = match opc {
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
                    info!("val: {}", val);
                    let res = match opc {
                        I32_EQZ => if val == 0 { 1 } else { 0 },
                        I32_CLZ => val.leading_zeros(),
                        I32_CTZ => val.trailing_zeros(),
                        I32_POPCNT => val.count_ones(),
                        _ => unimplemented!(),
                    };
                    info!("res: {}", res);                    
                    self.push(res as i32)?;
                },
                INTERP_ALLOCA => {
                    let count = code.read_u32()?;
                    info!("INTERP_ALLOCA: {}", count);
                    for _ in 0..count {
                        self.push(0)?;
                    }
                },
                INTERP_BR_UNLESS => {
                    let offset = code.read_u32()?;
                    info!("BR_UNLESS: {:08x}", offset);
                    let val = self.pop()?;
                    if val == 0 {             
                        info!("  => {:08x}", offset);
                        code.set_pos(offset as usize);
                    }
                },                
                INTERP_DROP_KEEP => {
                    let drop = code.read_u32()?;
                    let keep = code.read_u32()?;
                    info!("INTERP_DROP_KEEP {} {}", drop, keep);
                    let val = if keep > 0 {
                        Some(self.pop()?)
                    } else {
                        None
                    };
                    info!("keeping {:?}", val);
                    for _ in 0..drop {
                        let v = self.pop()?;
                        info!("popped {:?}", v);
                    }
                    if let Some(val) = val {
                        info!("pushed {:?}", val);
                        self.push(val)?;
                    }
                    info!("V: {}", self.value_stack.len());
                },
                _ => return Err(Error::Unimplemented),
            }
            // info!("{:08x}: END INST {:08x}", code.pos(), code.len());
            _count += 1;
        }
        Ok(())
    }

}

#[cfg(test_disabled)]
mod tests {
    use super::*;
    use module::ModuleWrite;

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
        test_drop : {
            w : {
                w.write_opcode(I32_CONST)?;
                w.write_u32(0x1234)?;
                w.write_opcode(DROP)?
            }, 
            i: {
                i.run()?;
                assert_eq!(i.value_stack.len(), 0);
            }
        },
        test_select_0 : {
            w : {
                w.write_opcode(SELECT)?;
            }, 
            i: {
                i.push(0x10)?; // true
                i.push(0x20)?; // false 
                i.push(0x0)?;  // cond
                i.run()?;
                assert_eq!(i.pop()?, 0x20);
            }
        },        
        test_select_1 : {
            w : {
                w.write_opcode(SELECT)?;
            }, 
            i: {
                i.push(0x10)?; // true
                i.push(0x20)?; // false 
                i.push(0x1)?;  // cond
                i.run()?;
                assert_eq!(i.pop()?, 0x10);
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
        test_set_local : {
            w : {
                w.write_opcode(SET_LOCAL)?;
                w.write_u32(0x1)?;
            },
            i: {
                i.push(0x0)?;
                i.push(0x1234)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x1234);
            }
        },   
        test_get_local : {
            w : {
                w.write_opcode(GET_LOCAL)?;
                w.write_u32(0x1)?;
            },
            i: {
                i.push(0x1234)?;
                i.push(0x0000)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x1234);
            }
        }, 
        test_tee_local : {
            w : {
                w.write_opcode(TEE_LOCAL)?;
                w.write_u32(0x0)?;
            },
            i: {
                i.push(0x1234)?; // Local 0
                i.push(0x0010)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x0010);
            }
        },                                                    
        test_set_global : {
            w : {
                w.write_opcode(SET_GLOBAL)?;
                w.write_u32(0x10)?;
            },
            i: {
                i.push(0x1234)?;
                i.run()?;
                assert_eq!(i.value_stack.len(), 0);
                assert_eq!(i.get_global(0x10)?, 0x1234);
            }
        },                       
        test_get_global : {
            w : {
                w.write_opcode(GET_GLOBAL)?;
                w.write_u32(0x10)?;
            },
            i: {
                i.set_global(0x10, 0x1234)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x1234);
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
        },
        test_drop_keep_0_0 : {
            w : {
                w.write_opcode(INTERP_DROP_KEEP)?;
                w.write_u32(0x0)?;
                w.write_u32(0x0)?;
            },
            i: {
                i.push(0x10)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x10);
                assert_eq!(i.value_stack.len(), 0);
            }
        },                
        test_drop_keep_0_1 : {
            w : {
                w.write_opcode(INTERP_DROP_KEEP)?;
                w.write_u32(0x0)?;
                w.write_u32(0x1)?;
            },
            i: {
                i.push(0x10)?;
                i.push(0x20)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x20);
                assert_eq!(i.value_stack.len(), 1);
            }
        },
        test_drop_keep_1_0 : {
            w : {
                w.write_opcode(INTERP_DROP_KEEP)?;
                w.write_u32(0x1)?;
                w.write_u32(0x0)?;
            },
            i: {
                i.push(0x10)?;
                i.run()?;
                assert_eq!(i.value_stack.len(), 0);
            }
        },
        test_drop_keep_1_1 : {
            w : {
                w.write_opcode(INTERP_DROP_KEEP)?;
                w.write_u32(0x1)?;
                w.write_u32(0x1)?;
            },
            i: {
                i.push(0x10)?;
                i.push(0x20)?;
                i.run()?;
                assert_eq!(i.pop()?, 0x20);                
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
        test_i32_ne_0_1(I32_NE, 0, 1, 1);
        test_i32_ne_1_1(I32_NE, 1, 1, 0);

        test_i32_ltu_0_1(I32_LT_U, 0, 1, 1);
        test_i32_ltu_1_1(I32_LT_U, 1, 1, 0);

        test_i32_lts_0_1(I32_LT_S, 0, 1, 1);
        test_i32_lts_1_1(I32_LT_S, 1, 1, 0);

        test_i32_gtu_1_0(I32_GT_U, 1, 0, 1);
        test_i32_gtu_1_1(I32_GT_U, 1, 1, 0);

        test_i32_gts_1_0(I32_GT_S, 1, 0, 1);
        test_i32_gts_1_1(I32_GT_S, 1, 1, 0);

        test_i32_leu_0_1(I32_LE_U, 0, 1, 1);
        test_i32_leu_1_1(I32_LE_U, 1, 1, 1);
        test_i32_leu_2_1(I32_LE_U, 2, 1, 0);

        test_i32_les_0_1(I32_LE_S, 0, 1, 1);
        test_i32_les_1_1(I32_LE_S, 1, 1, 1);
        test_i32_les_2_1(I32_LE_S, 2, 1, 0);

        test_i32_geu_0_1(I32_GE_U, 0, 1, 0);
        test_i32_geu_1_1(I32_GE_U, 1, 1, 1);
        test_i32_geu_2_1(I32_GE_U, 2, 1, 1);

        test_i32_ges_0_1(I32_GE_S, 0, 1, 0);
        test_i32_ges_1_1(I32_GE_S, 1, 1, 1);
        test_i32_ges_2_1(I32_GE_S, 2, 1, 1);

        test_i32_add_1_2(I32_ADD, 1, 2, 3);

        test_i32_sub_3_2(I32_SUB, 3, 2, 1);

        test_i32_mul_3_2(I32_MUL, 3, 2, 6);

        test_i32_divs_12_3(I32_DIV_S, -12, 3, -4);
        test_i32_divu_12_3(I32_DIV_U, 12, 3, 4);
    }
}