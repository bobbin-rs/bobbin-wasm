pub type StackResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Overflow,
    Underflow,
}

pub struct Stack<'a, T: 'a + Copy> {
    buf: &'a mut [T],
    pos: usize,
}

impl<'a, T: 'a + Copy> Stack<'a, T> {
    pub fn new(buf: &'a mut [T]) -> Self {
        Stack { buf: buf, pos: 0 }
    }

    #[inline]
    pub fn cap(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.pos == 0
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.pos == self.buf.len()
    }    

    #[inline]
    pub fn reset(&mut self) -> StackResult<()> {
        Ok(self.pos = 0)
    }

    #[inline]
    fn pre_incr(&mut self) -> StackResult<usize> {
        let pos = self.pos;
        if self.full() { 
            Err(Error::Overflow)
        } else {
            self.pos += 1;
            Ok(pos)
        }
    }

    #[inline]
    fn post_decr(&mut self) -> StackResult<usize> {
        let mut pos = self.pos;
        if self.empty() { 
            Err(Error::Underflow)
        } else {
            pos -= 1;
            self.pos = pos;
            Ok(pos)
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) -> StackResult<()> {
        Ok(self.buf[self.pre_incr()?] = value)
    }

    #[inline]
    pub fn pop(&mut self) -> StackResult<T> {
        Ok(self.buf[self.post_decr()?])
    }

    /// Returns a copy of the item at the top of the stack.
    #[inline]
    pub fn top(&mut self) -> StackResult<T> {
        if self.empty() {
            Err(Error::Underflow)            
        } else {
            Ok(self.buf[self.pos - 1])
        }
    }

    /// Returns a copy of the item `depth` items from the top.
    #[inline]
    pub fn peek(&self, depth: usize) -> StackResult<T> {
        if depth >= self.pos {
            Err(Error::Underflow)
        } else {
            Ok(self.buf[self.pos - depth - 1])
        }
    }

    /// Returns a mutable reference to the item `depth` items from the top.
    #[inline]
    pub fn pick(&mut self, depth: usize) -> StackResult<&mut T> {
        if depth >= self.pos {
            Err(Error::Underflow)
        } else {
            Ok(&mut self.buf[self.pos - depth - 1])
        }        
    }

    /// Drops `drop_count` items. If `keep_count` is 0, then items [0..`drop_count`) are
    /// deleted. If `keep_count` is 1, then items [1..`drop_count`+1) are deleted.
    #[inline]
    pub fn drop_keep(&mut self, drop_count: usize, keep_count: usize) -> StackResult<()> {
        assert!(keep_count <= 1, "keep_count must be 0 or 1");
        if keep_count == 1 {
            *self.pick(drop_count)? = self.top()?;
        }
        Ok(self.pos -= drop_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut buf = [0u8; 8];
        let mut s = Stack::new(&mut buf);
        assert!(s.empty());
        assert!(!s.full());
        assert_eq!(s.cap(), 8);
        assert_eq!(s.len(), 0);

        for i in 0..8 {
            s.push(i as u8).unwrap();            
        }

        assert_eq!(s.push(8), Err(Error::Overflow));
        assert_eq!(s.len(), 8);

        assert!(!s.empty());
        assert!(s.full());

        assert_eq!(s.top().unwrap(), 7);
        for i in 0..8 {
            assert_eq!(s.peek(i).unwrap(), (7-i) as u8);
        }
        assert_eq!(s.peek(8), Err(Error::Underflow));

        for i in 0..8 {
            *s.pick(i).unwrap() = i as u8;
        }

        for i in 0..8 {
            assert_eq!(s.pop().unwrap(), i as u8);
        }
    }

    #[test]
    fn test_pick() {
        let mut buf = [0u8; 8];
        let mut s = Stack::new(&mut buf);
        s.push(1).unwrap();
        assert_eq!(*s.pick(0).unwrap(), 1);
    }

    #[test]
    fn test_drop_keep() {
        let mut buf = [0u8; 8];
        let mut s = Stack::new(&mut buf);
        for i in 0..8 {
            s.push(i as u8).unwrap();
        }
        s.drop_keep(8, 0).unwrap();
        assert_eq!(s.len(), 0);

        for i in 0..8 {
            s.push(i as u8).unwrap();
        }
        assert_eq!(s.len(), 8);
        s.drop_keep(7, 1).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s.top().unwrap(), 7);
    }
}