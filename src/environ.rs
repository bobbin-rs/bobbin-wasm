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

pub struct Environment<'buf> {
    cfg: Config,
    mem: MemoryInst<'buf>,
}

impl<'buf> Environment<'buf> {
    pub fn new(buf: &'buf mut [u8]) -> (&'buf mut [u8], Self) {   
        Environment::new_with_config(buf, Config::default())
    }

    pub fn new_with_config(buf: &'buf mut [u8], cfg: Config) -> (&'buf mut [u8], Self) {   
        let (mem_buf, buf) = buf.split_at_mut(cfg.memory_pages * PAGE_SIZE);
        let mem = MemoryInst::new(mem_buf, 1, None);
        (buf, Environment { cfg, mem })
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'buf> {
        &self.mem
    }

    pub fn load_module<'m>(&'buf mut self, buf: &'m mut [u8], module_data: &[u8]) -> Result<(&'m mut [u8], ModuleInst<'m, 'buf>), Error> {
        let m = Module::from(module_data);
        ModuleInst::new(buf, self, m)
    }
}
