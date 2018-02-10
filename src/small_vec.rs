use core::ops::{Index, IndexMut};

pub struct SmallVec<'a, T: 'a> {
    buf: &'a mut [T],
    pos: usize,
}

impl<'a, T: 'a> SmallVec<'a, T> {
    pub fn new(buf: &'a mut [T]) -> Self {
        SmallVec { buf, pos: 0}
    }

    pub fn into_buf(self) -> &'a mut [T] {
        self.buf
    }

    pub fn cap(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.pos
    }

    pub fn rem(&self) -> usize {
        self.cap() - self.len()
    }

    pub fn push(&mut self, value: T) {
        self.buf[self.pos] = value;
        self.pos += 1;
    }
}

impl<'a, T: 'a + Copy> SmallVec<'a, T> {
    pub fn pop(&mut self) -> Option<T> {
        if self.pos > 0 {            
            self.pos -= 1;
            Some(self.buf[self.pos])
        } else {
            None
        }
    }
}

impl<'a, T: 'a> AsRef<[T]> for SmallVec<'a, T> {
    fn as_ref(&self) -> &[T] {
        &self.buf[..self.pos]
    }
}

impl<'a, T: 'a> AsMut<[T]> for SmallVec<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.buf[..self.pos]
    }
}


impl<'a, T: 'a> Index<usize> for SmallVec<'a, T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.buf[i]
    }
}

impl<'a, T: 'a> IndexMut<usize> for SmallVec<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smallvec() {
        let mut buf = [0u8; 16];
        let mut v = SmallVec::new(&mut buf);
        for i in 0..16 {
            v.push(i as u8);
        }
        assert_eq!(v.cap(), 16);
        assert_eq!(v.len(), 16);
        assert_eq!(v.rem(), 0);

        for i in 0..16 {
            assert_eq!(v[i], i as u8);
            v[i] = v[i] * 2;
            assert_eq!(v[i], (i * 2) as u8);
        }

        for i in 0..16 {
            assert_eq!(v.pop(), Some((16 - i - 1) * 2));
        }

        assert_eq!(v.len(), 0);

    }
}