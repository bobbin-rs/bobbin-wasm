use core::ops::{Index, IndexMut};

pub const SMALLVEC_SIZE: usize = 16;

pub struct SmallVec<T> {
    buf: [Option<T>; SMALLVEC_SIZE],
    len: usize,
}

impl<T> SmallVec<T> {
    pub fn new() -> Self {
        SmallVec { buf: [
            None, None, None, None,
            None, None, None, None,
            None, None, None, None,
            None, None, None, None,
            ], len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, v: T) {
        self.buf[self.len] = Some(v);
        self.len += 1;
    }
}

impl<T> Index<usize> for SmallVec<T> {
    type Output = Option<T>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.buf[i]
    }
}

impl<T> IndexMut<usize> for SmallVec<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.buf[i]
    }
}