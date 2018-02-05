use ::opcode::*;
use ::core::convert::TryFrom;
use Error;
use wasm_leb128::{read_i32, read_u32, read_u1, read_i7};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Clone, Copy, Debug, Default)]
pub struct Entry {
    pc: usize,
    end: usize,
    mid: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Scanner<'a> {
    code: &'a [u8],
    pc: usize,
    stack: [usize; 16],
    stack_pos: usize,
    array: [Entry; 16],
    array_pos: usize,
}

impl<'a> Scanner<'a> {
    fn new(code: &'a [u8]) -> Self {
        Scanner { 
            code: code,
            pc: 0,
            stack: [0; 16], 
            stack_pos: 0,
            array: [Entry::default(); 16],
            array_pos: 0,
        }
    }

    fn push(&mut self) -> usize {
        let id = self.array_pos;
        self.array[self.array_pos] = Entry { pc: self.pc, end: 0, mid: 0 };
        self.array_pos += 1;
        self.stack[self.stack_pos] = id;
        self.stack_pos += 1;
        id
    }

    fn pop(&mut self) -> usize {
        self.stack_pos -= 1;
        self.stack[self.stack_pos]
    }

    fn top(&self) -> usize {
        self.stack[self.stack_pos - 1]
    }

    pub fn read_var_u1(&mut self) -> Result<bool, Error> {
        let (v, n) = try!(read_u1(&self.code[self.pc..]));
        self.pc += n;
        Ok(v)
    }

    pub fn read_var_i7(&mut self) -> Result<i8, Error> {
        let (v, n) = try!(read_i7(&self.code[self.pc..]));
        self.pc += n;
        Ok(v)
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

    pub fn read_f32(&mut self) -> Result<f32, Error> {
        let v = LittleEndian::read_f32(&self.code[self.pc..]);
        self.pc += 4;
        Ok(v)
    }
    

    fn indent(&self) {
        print!("0x{:02x}: ", self.pc);
        for _ in 0..self.stack_pos*2 { print!(" ") }
    }

    fn scan_var_u1(&mut self) -> Result<(), Error> {
        print!(" {}", if self.read_var_u1()? { 1 } else { 0 });
        Ok(())
    }

    fn scan_var_i7(&mut self) -> Result<(), Error> {
        print!(" 0x{}", self.read_var_i7()?);
        Ok(())
    }

    fn scan_var_i32(&mut self) -> Result<(), Error> {
        print!(" 0x{:08x}", self.read_var_i32()?);
        Ok(())
    }

    fn scan_var_u32(&mut self) -> Result<(), Error> {
        print!(" 0x{:08x}", self.read_var_u32()?);
        Ok(())
    }

    fn scan_f32(&mut self) -> Result<(), Error> {
        print!(" 0x{:08x}", self.read_f32()? as u32);
        Ok(())
    }

    fn scan_block_signature_type(&mut self) -> Result<(), Error> {
        print!(" 0x{:02x}", self.read_var_i7()?);
        Ok(())
    }

    fn scan_depth(&mut self) -> Result<(), Error> {
        print!(" {}", self.read_var_u32()?);
        Ok(())
    }

    fn scan_table(&mut self) -> Result<(), Error> {
        let size = self.read_var_u32()?;
        print!(" [");
        for i in 0..size {
            if i > 0 { print!(" ") }
            print!("{}", self.read_var_u32()?);
        }
        print!("] {}", self.read_var_u32()?);
        Ok(())
    }    

    fn scan_load_store(&mut self) -> Result<(), Error> {
        print!(" 0x{:08x}", self.read_var_u32()?);
        print!(" 0x{:08x}", self.read_var_u32()?);        
        Ok(())
        
    }

    fn scan(&mut self) -> Result<(), Error> {
        while self.pc < self.code.len() {
            let op = self.code[self.pc];
            self.pc += 1;

            // println!("0x{:02x}", op);
            match op {
                NOP => {
                    self.indent();
                    println!("nop");
                },
                BLOCK => {
                    self.indent();
                    print!("(block");
                    let n = self.push();
                    self.scan_block_signature_type()?;
                    println!(" #{}", n);
                },
                LOOP => {
                    self.indent();
                    print!("(loop");
                    let n = self.push();
                    self.scan_block_signature_type()?;
                    println!(" #{}", n);
                },
                IF => {
                    self.indent();
                    print!("(if");
                    let n = self.push();
                    self.scan_block_signature_type()?;
                    println!(" #{}", n);
                },                
                ELSE => {
                    let e = self.top();
                    self.array[e].mid = self.pc;
                    self.indent();
                    println!("; else #{}", e);
                },                
                END => {
                    let e = self.pop();
                    self.array[e].end = self.pc;
                    self.indent();
                    println!("); #{}", e);
                },
                BR => {
                    self.indent();
                    print!("br");
                    self.scan_depth()?;
                    println!("");
                },
                BR_IF => {
                    self.indent();
                    print!("br_if");
                    self.scan_depth()?;
                    println!("");
                },
                BR_TABLE => {
                    self.indent();
                    print!("br_table");
                    self.scan_table()?;
                    println!("");
                },
                RETURN => {
                    self.indent();
                    println!("return");
                }
                _ => {
                    if let Ok(op) = Opcode::try_from(op) {
                        print!("0x{:02x}: {}", self.pc, op.text);
                        match op.code {
                            I32_CONST => self.scan_var_i32()?,
                            F32_CONST => self.scan_f32()?,
                            GET_LOCAL | SET_LOCAL | TEE_LOCAL => self.scan_var_u32()?,
                            GET_GLOBAL | SET_GLOBAL => self.scan_var_u32()?,
                            CALL => self.scan_var_u32()?,
                            CALL_INDIRECT => {
                                self.scan_var_u32()?;
                                self.scan_var_u1()?;
                            },
                            I32_LOAD | F32_LOAD | I32_STORE | F32_STORE | 
                            I32_LOAD8_S | I32_LOAD8_U | I32_LOAD16_S | I32_LOAD16_U |
                            I32_STORE8 | I32_STORE16 => self.scan_load_store()?,
                            MEM_GROW | MEM_SIZE => self.scan_var_u1()?,
                            _ => {},
                        }
                        println!("");
                    } else {
                        panic!("Invalid opcode: 0x{:02x}", op);
                    }
                },
            };
        }
        assert!(self.stack_pos == 0);

        for i in 0..self.array_pos {
            let e = self.array[i];
            print!("#{}: pc: 0x{:02x} end: 0x{:02x}", i, e.pc, e.end);
            if e.mid > 0 {
                print!(" mid: 0x{:02x}", e.mid);
            }
            println!("");
        }
        Ok(())

    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_scanner() {
    //     let code = [
    //         NOP, 
    //         MEM_SIZE, 0x00,
    //         I32_LOAD, 0x02, 0x00,
    //         I32_ADD,
    //         I32_SUB,
    //         CALL, 0x01,
    //         CALL_INDIRECT, 0x01, 0x00,
    //         BLOCK, 0x00,
    //             BR_TABLE, 0x02, 0x00, 0x00, 0x00,
    //             LOOP, 0x00,
    //                 BR, 0x01,
    //                 IF, 0x00,
    //                     BR_IF, 0x02,
    //                 END, 
    //                 IF, 0x00,
    //                     NOP, 
    //                     IF, 0x00,
    //                         NOP, 
    //                     END, 
    //                 ELSE, 
    //                     NOP, 
    //                     IF, 
    //                         NOP, 
    //                     END, 
    //                 END, 
    //             END, 
    //         END, 
    //         NOP,
    //     ];

    //     // let mut s = Scanner::new(&code);
    //     // s.scan().unwrap();
    // }
}