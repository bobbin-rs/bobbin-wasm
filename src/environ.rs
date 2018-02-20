use PAGE_SIZE;
use error::Error;
use writer::Writer;
use small_vec::SmallVec;
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
    modules: SmallVec<'env, &'env ModuleInst<'env>>,
}

impl<'env> Environment<'env> {
    pub fn new(buf: &'env mut [u8]) -> (&'env mut [u8], Self) {   
        Environment::new_with_config(buf, Config::default())
    }

    pub fn new_with_config(buf: &'env mut [u8], cfg: Config) -> (&'env mut [u8], Self) {   
        let (mem_buf, buf) = buf.split_at_mut(cfg.memory_pages * PAGE_SIZE);
        let mem = MemoryInst::new(mem_buf, 1, None);
        let mut w = Writer::new(buf);
        let modules = w.alloc_smallvec(4);
        let buf = w.into_slice();
        (buf, Environment { cfg, mem, modules })
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'env> {
        &self.mem
    }

    pub fn load_module(&mut self, buf: &'env mut [u8], module_data: &[u8]) -> Result<(&'env mut [u8], &'env ModuleInst<'env>), Error> {
        let m = Module::from(module_data);
        let (buf, mi) = ModuleInst::new(buf, &self.mem, m)?;
        let mut w = Writer::new(buf);
        let mi = w.copy(mi)?;
        self.modules.push(mi);
        let buf = w.into_slice();        
        Ok((buf, mi))
    }
}
