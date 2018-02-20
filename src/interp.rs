use {Error, Value};

// use module_inst::{ FuncInst};
use environ::Environment;
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
    #[allow(dead_code)]
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

    pub fn call(&mut self, env: &Environment, mi: &ModuleInst, func_index: usize) -> Result<Option<Value>, Error> {
        let code_buf = mi.code().as_ref();

        info!("code section len: {:08x}", code_buf.len());

        let body_range = mi.code().body_range(func_index);        
        info!("body: {:08x} to {:08x}", body_range.start, body_range.end);

        let mut code = Reader::new(code_buf);
        code.set_pos(body_range.start);

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
                    match &mi.functions()[id as usize] {
                        &FuncInst::Import { type_index, ref module, ref name, module_index, import_index } => {
                            info!("CALL IMPORT: type_index: {} module: {}, name: {}, module_index: {}, import_index: {}", type_index, module, name, module_index, import_index);
                            if module.0 == b"host" {
                                env.call_host_function(self, import_index)?;
                            } else {
                                env.call_module_function(self, module_index, import_index)?;
                            }                            
                        },
                        &FuncInst::Local { type_index: _, function_index } => {
                            let body_range = mi.code().body_range(function_index);
                            let offset = body_range.start;
                            let pos = code.pos();
                            info!("CALL: {:08x} to {:08x}", pos, offset);

                            self.call_stack.push(pos as u32)?;
                            code.set_pos(offset as usize);                            
                        }
                    }
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
                        &FuncInst::Import { type_index: _, module: _, name: _, module_index: _, import_index: _ } => {
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

                            let body_range = mi.code().body_range(function_index);
                            let offset = body_range.start;

                            // let body = m.body(function_index as u32).unwrap();
                            // let offset = code_buf.as_ptr().offset_to(body.buf.as_ptr()).unwrap() as usize;
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
                    let ret = env.mem().grow_memory(pages);
                    info!("  => {}", ret);
                    self.push(ret)?;
                },
                MEM_SIZE => {
                    let size = env.mem().num_pages();
                    self.push(size as i32)?;
                }
                // I32 load
                0x28 ... 0x30 => {
                    let _flags = code.read_u32()?;
                    let offset = code.read_u32()?;
                    let base: u32 = self.pop()? as u32;
                    let addr = (offset + base) as usize;
                    let mem = env.mem();

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
                    let mem = env.mem();

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

        match self.stack_len() {
            0 => Ok(None),
            1 => Ok(Some(Value(self.pop()?))),
            _ => Err(Error::UnexpectedReturnLength { got: self.stack_len() as u32 }),
        }
    }

}
