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
    pub fn new(buf: &'env mut [u8]) -> (&'env mut [u8], Self) {   
        Environment::new_with_config(buf, Config::default())
    }

    pub fn new_with_config(buf: &'env mut [u8], cfg: Config) -> (&'env mut [u8], Self) {   
        let (mem_buf, buf) = buf.split_at_mut(cfg.memory_pages * PAGE_SIZE);
        let mem = MemoryInst::new(mem_buf, 1, None);
        (buf, Environment { cfg, mem })
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'env> {
        &self.mem
    }

    pub fn load_module<'buf>(&mut self, buf: &'env mut [u8], module_data: &[u8]) -> Result<(&'env mut [u8], ModuleInst<'env>), Error> {
        let m = Module::from(module_data);
        ModuleInst::new(buf, self, m)
    }
}
