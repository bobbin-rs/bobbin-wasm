use PAGE_SIZE;
use error::Error;
use writer::Writer;
use small_vec::SmallVec;
use module::Module;
use memory_inst::MemoryInst;
use module_inst::{ModuleInst, FuncInst};
use types::{Value, ImportDesc, Identifier};
use interp::Interp;

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

pub trait HostHandler {
    fn import(&self, module: &Identifier, export: &Identifier, import_desc: &ImportDesc) -> Result<usize, Error>;
    fn dispatch(&self, interp: &mut Interp, type_index: usize, index: usize) -> Result<(), Error>;
}

pub struct Environment<'env, H: HostHandler> {
    cfg: Config,
    mem: MemoryInst<'env>,
    modules: SmallVec<'env, (&'env str, &'env ModuleInst<'env>)>,
    host_handler: H,
}

impl<'env, H: HostHandler> Environment<'env, H> {
    pub fn new(buf: &'env mut [u8], host_handler: H) -> (&'env mut [u8], Self) {   
        Environment::new_with_config(buf, host_handler, Config::default())
    }

    pub fn new_with_config(buf: &'env mut [u8], host_handler: H, cfg: Config) -> (&'env mut [u8], Self) {   
        let (mem_buf, buf) = buf.split_at_mut(cfg.memory_pages * PAGE_SIZE);
        let mem = MemoryInst::new(mem_buf, 1, None);
        let mut w = Writer::new(buf);
        let modules = w.alloc_smallvec(4);
        let buf = w.into_slice();
        (buf, Environment { cfg, mem, modules, host_handler })
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub fn mem(&self) -> &MemoryInst<'env> {
        &self.mem
    }

    pub fn load_module(&mut self, name: &'env str, buf: &'env mut [u8], module_data: &[u8]) -> Result<(&'env mut [u8], &'env ModuleInst<'env>), Error> {
        let m = Module::from(module_data);
        let (buf, mi) = ModuleInst::new(buf, &self, &self.mem, m)?;
        let mut w = Writer::new(buf);
        let mi = w.copy(mi)?;
        self.modules.push((name, mi));
        let buf = w.into_slice();        
        Ok((buf, mi))
    }

    pub fn import_host_function(&self, module: &Identifier, export: &Identifier, import_desc: &ImportDesc) -> Result<usize, Error> {
        self.host_handler.import(module, export, import_desc)
    }

    pub fn call_host_function(&self, interp: &mut Interp, type_index: usize, index: usize) -> Result<(), Error> {
        self.host_handler.dispatch(interp, type_index, index)
    }

    pub fn call_module_function(&self, interp: &mut Interp, module_index: usize, function_index: usize) -> Result<(), Error> {
        let &(name, mi) = &self.modules[module_index];
        let id = function_index;
        info!("calling {}:{}", name, function_index);

        match &mi.functions()[id] {
            &FuncInst::Host { type_index, module: _, name: _, host_index } => {
                self.call_host_function(interp, type_index, host_index)
            },
            &FuncInst::Import { type_index, ref module, ref name, module_index, import_index } => {
                info!("CALL IMPORT: type_index: {} module: {}, name: {}, module_index: {}, import_index: {}", type_index, module, name, module_index, import_index);
                self.call_module_function(interp, module_index, import_index)
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
