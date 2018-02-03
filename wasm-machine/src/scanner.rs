use super::opcode::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Entry {
    pc: usize,
    end: usize,
    mid: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Scanner {
    pc: usize,
    stack: [usize; 16],
    stack_pos: usize,
    array: [Entry; 16],
    array_pos: usize,
}

impl Scanner {
    fn new() -> Self {
        Scanner { 
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

    fn indent(&self) {
        print!("0x{:02x}: ", self.pc);
        for _ in 0..self.stack_pos*2 { print!(" ") }
    }

    fn scan(&mut self, body: &mut [u8]) {
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
                    print!("(block");
                    let n = self.push();
                    println!(" #{}", n);

                },
                LOOP => {
                    self.indent();
                    print!("(loop");
                    let n = self.push();
                    println!(" #{}", n);
                },
                IF => {
                    self.indent();
                    print!("(if");
                    let n = self.push();
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
    assert!(self.stack_pos == 0);

    for i in 0..self.array_pos {
        let e = self.array[i];
        print!("#{}: pc: 0x{:02x} end: 0x{:02x}", i, e.pc, e.end);
        if e.mid > 0 {
            print!(" mid: 0x{:02x}", e.mid);
        }
        println!("");
    }

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