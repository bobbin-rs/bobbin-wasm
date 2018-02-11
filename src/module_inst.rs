use {Error};
use module::*;
use writer::Writer;

pub struct ModuleInst<'a> {
    name: &'a str,
}

impl<'a> ModuleInst<'a> {
    pub fn new(m: &Module, buf: &'a mut [u8]) -> Result<Self, Error> {
        let mut w = Writer::new(buf);
        let name = w.copy_str(m.name());

        Ok(ModuleInst { name })
    }

    pub fn name(&self) -> &str {
        self.name
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_inst() {
        let mut buf = [0u8; 1024];

        let mut m = Module::new(&[]);
        m.set_name("hello.wasm", &[]);

        let mi = ModuleInst::new(&m, &mut buf).unwrap();
        assert_eq!(mi.name(), "hello.wasm");
    }
}