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
    mem: MemoryInst<'env>,
}

impl<'env> Environment<'env> {
    pub fn new(buf: &'env mut [u8]) -> (Self, &'env mut [u8]) {   
        Environment::new_with_config(Config::default(), buf)
    }

    pub fn new_with_config(cfg: Config, buf: &'env mut [u8]) -> (Self, &'env mut [u8]) {   
        let (mem_buf, buf) = buf.split_at_mut(cfg.memory_pages * PAGE_SIZE);
        let mem = MemoryInst::new(mem_buf, 1, None);
        (Environment { cfg, mem }, buf)
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'env> {
        &self.mem
    }

    pub fn load_module(&'env mut self, buf: &'env mut [u8], module_data: &[u8]) -> Result<(ModuleInst<'env>, &'env mut [u8]), Error> {
        let m = Module::from(module_data);
        ModuleInst::new(self, buf, m)
    }
}
