use PAGE_SIZE;
use error::Error;
use writer::Writer;
use small_vec::SmallVec;
use module::Module;
use memory_inst::MemoryInst;
use module_inst::{ModuleInst, FuncInst};
use types::Value;
use interp::Interp;

pub type HostFn = fn(interp: &mut Interp, index: usize) -> Result<(), Error>;

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
    modules: SmallVec<'env, (&'env str, &'env ModuleInst<'env>)>,
    host_fn: Option<HostFn>,
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
        let host_fn = None;
        (buf, Environment { cfg, mem, modules, host_fn })
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'env> {
        &self.mem
    }

    pub fn register_host_function(&mut self, host_fn: HostFn) -> Result<(), Error> {
        Ok({
            self.host_fn = Some(host_fn);
        })
    }

    pub fn load_module(&mut self, name: &'env str, buf: &'env mut [u8], module_data: &[u8]) -> Result<(&'env mut [u8], &'env ModuleInst<'env>), Error> {
        let m = Module::from(module_data);
        let (buf, mi) = ModuleInst::new(buf, &self.mem, m)?;
        let mut w = Writer::new(buf);
        let mi = w.copy(mi)?;
        self.modules.push((name, mi));
        let buf = w.into_slice();        
        Ok((buf, mi))
    }

    pub fn call_host_function(&self, interp: &mut Interp, index: usize) -> Result<(), Error> {
        if let Some(host_fn) = self.host_fn { 
            host_fn(interp, index)
        } else {
            Err(Error::NoHostFunction)
        }
    }
    pub fn call_module_function(&self, interp: &mut Interp, module_index: usize, function_index: usize) -> Result<(), Error> {
        let &(name, mi) = &self.modules[module_index];
        let id = function_index;
        info!("calling {}:{}", name, function_index);

        match &mi.functions()[id] {
            &FuncInst::Import { type_index, ref module, ref name, module_index, import_index } => {
                info!("CALL IMPORT: type_index: {} module: {}, name: {}, module_index: {}, import_index: {}", type_index, module, name, module_index, import_index);
                if module.0 == b"host" {
                    self.call_host_function(interp, import_index)
                } else {
                    self.call_module_function(interp, module_index, import_index)
                }                            
            },
            &FuncInst::Local { type_index: _, function_index } => {
                if let Some(Value(v)) = interp.call(self, mi, function_index)? {
                    Ok(interp.push(v)?)
                } else {
                    Ok(())
                }
            }
        }
    }        
}
