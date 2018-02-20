use PAGE_SIZE;
use error::Error;
use module::Module;
use memory_inst::MemoryInst;
use module_inst::ModuleInst;

pub struct Config {
    memory_pages: usize
}

impl Default for Config {
    fn default() -> Config {
        Config {
            memory_pages: 4,
        }
    }
}

pub struct Environment<'env> {
    cfg: Config,
    buf: Option<&'env mut [u8]>,
    mem: MemoryInst<'env>,
}

impl<'env> Environment<'env> {
    pub fn new(buf: &'env mut [u8]) -> Self {   
        Environment::new_with_config(Config::default(), buf)
    }

    pub fn new_with_config(cfg: Config, buf: &'env mut [u8]) -> Self {   
        let (mem_buf, buf) = buf.split_at_mut(cfg.memory_pages * PAGE_SIZE);
        let mem = MemoryInst::new(mem_buf, 1, None);
        Environment { cfg, buf: Some(buf), mem }
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'env> {
        &self.mem
    }

    pub fn load_module(&'env mut self, module_data: &[u8]) -> Result<ModuleInst<'env>, Error> {
        let m = Module::from(module_data);
        let buf = self.buf.take().unwrap();
        let (mi, _buf) = ModuleInst::new(self, buf, m)?;
        // return remaining buffer to environ
        Ok(mi)
    }
}
