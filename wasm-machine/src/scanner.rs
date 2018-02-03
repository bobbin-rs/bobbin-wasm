use super::opcode::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Entry {
    pc: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Scanner {
    pc: usize,
    stack: [Entry; 16],
    stack_pos: usize,
    array: [Entry; 16],
    array_pos: usize,
}

impl Scanner {
    fn new() -> Self {
        Scanner { 
            pc: 0,
            stack: [Entry::default(); 16], 
            stack_pos: 0,
            array: [Entry::default(); 16],
            array_pos: 0,
        }
    }

    fn push(&mut self) {
        self.stack[self.stack_pos] = Entry { pc: self.pc };
        self.stack_pos += 1;
    }

    fn pop(&mut self) -> Entry {
        self.stack_pos -= 1;
        self.stack[self.stack_pos]
    }

    fn top(&self) -> &Entry {
        &self.stack[self.stack_pos - 1]
    }

    fn indent(&self) {
        print!("0x{:02x}: ", self.pc);
        for _ in 0..self.stack_pos*2 { print!(" ") }
    }

    fn scan(&mut self, body: &mut [u8]) {
        let mut depth = 0;
        for (i, op) in body.iter().enumerate() {
            self.pc = i;
            // println!("0x{:02x}", op);
            match *op {
                NOP => {
                    self.indent();
                    println!("nop");
                },
                BLOCK => {
                    self.indent();
                    println!("(block");
                    self.push();
                },
                LOOP => {
                    self.indent();
                    println!("(loop");
                    self.push();
                },
                IF => {
                    self.indent();
                    println!("(if");
                    self.push();
                },                
                ELSE => {
                    let e = self.top();
                    self.indent();
                    println!("; else 0x{:02x}", e.pc);
                },                
                END => {
                    let e = self.pop();
                    self.indent();
                    println!("); 0x{:02x}", e.pc);
                },
                BR => {
                    self.indent();
                    println!("br");
                },
                BR_IF => {
                    self.indent();
                    println!("br_if");
                },
                BR_TABLE => {
                    self.indent();
                    println!("br_table");
                },
                RETURN => {
                    self.indent();
                    println!("return");
                }
                _ => {}
            }
        }
    assert!(depth == 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::opcode::*;

    #[test]
    fn test_scanner() {
        let mut ops = [NOP, BLOCK, LOOP, BR, IF, BR_IF, END, IF, NOP, IF, NOP, END, ELSE, NOP, IF, NOP, END, END, END, END, NOP];

        let mut s = Scanner::new();
        s.scan(&mut ops);
    }
}