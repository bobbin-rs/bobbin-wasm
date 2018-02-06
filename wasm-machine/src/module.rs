use {Error, Reader};

pub struct Module<'a> {
    r: Reader<'a>,
}

impl<'a> Module<'a> {
    pub fn new(r: Reader<'a>) -> Self {
        Module { r }
    }
}