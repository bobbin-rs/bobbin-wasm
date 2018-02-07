use {Error, Reader, Writer, TypeValue, SectionType, Module};
use loader::{Label, Loader};
use stack::Stack;
use opcode::{FUNC, VOID};

pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;

pub const FIXUP: u32 = 0xffff_ffff;


pub type ModuleResult<T> = Result<T, Error>;

pub struct ModuleLoader<'r, 'w> {
    r: Reader<'r>,
    w: Writer<'w>,
    m: Module<'w>,
}

impl<'r, 'w> ModuleLoader<'r, 'w> {
    pub fn new(r: Reader<'r>, mut w: Writer<'w>) -> Self {
        let m = Module::new(w.split());
        ModuleLoader { r, w, m }
    }

    fn done(&self) -> bool {
        self.r.done()
    }

    fn read_u8(&mut self) -> ModuleResult<u8> {
        self.r.read_u8()
    }

    fn read_u32(&mut self) -> ModuleResult<u32> {
        self.r.read_u32()
    }

    fn read_var_i7(&mut self) -> ModuleResult<i8> {
        self.r.read_var_i7()
    }    

    fn read_var_u7(&mut self) -> ModuleResult<u8> {
        self.r.read_var_u7()
    }    

    fn read_var_u32(&mut self) -> ModuleResult<u32> {
        self.r.read_var_u32()
    }

    fn read_var_i7_expecting(&mut self, want: i8, err: Error) -> ModuleResult<i8> {
        if let Ok(got) = self.read_var_i7() {
            if got == want {
                return Ok(got)
            }
        }
        Err(err)
    }

    fn read_u32_expecting(&mut self, want: u32, err: Error) -> ModuleResult<u32> {
        if let Ok(got) = self.read_u32() {
            if got == want {
                return Ok(got)
            }
        }
        Err(err)
    }

    fn write_u8(&mut self, value: u8) -> ModuleResult<()> {
        self.w.write_u8(value)
    }

    fn write_i8(&mut self, value: i8) -> ModuleResult<()> {
        self.w.write_i8(value)
    }    

    fn write_u32(&mut self, value: u32) -> ModuleResult<()> {
        self.w.write_u32(value)
    }

    fn write_u32_at(&mut self, value: u32, offset: u32) -> ModuleResult<()> {
        self.w.write_u32_at(value, offset as usize)
    }

    fn write_u32_fixup(&mut self) -> ModuleResult<u32> {
        let pos = self.w.pos();
        self.w.write_u32(FIXUP)?;
        Ok(pos as u32)
    }

    fn copy_u8(&mut self) -> ModuleResult<u8> {
        let val = self.r.read_u8()?;
        self.w.write_u8(val)?;
        Ok(val)
    }    


    fn copy_var_u1(&mut self) -> ModuleResult<u8> {
        let val = self.r.read_var_u1()?;
        self.w.write_u8(val)?;
        Ok(val)
    }    

    fn copy_var_i7(&mut self) -> ModuleResult<i8> {
        let val = self.r.read_var_i7()?;
        self.w.write_i8(val)?;
        Ok(val)
    }    

    fn copy_var_u7(&mut self) -> ModuleResult<u8> {
        let val = self.r.read_var_u7()?;
        self.w.write_u8(val)?;
        Ok(val)
    }        

    fn copy_var_u32(&mut self) -> ModuleResult<u32> {
        let val = self.r.read_var_u32()?;
        self.w.write_u32(val)?;
        Ok(val)
    }

    fn copy_identifier(&mut self) -> ModuleResult<()> {
        let len = self.copy_var_u32()?;
        for _ in 0..len {
            let v = self.read_u8()?;
            self.write_u8(v)?;
        }
        Ok(())
    }

    fn apply_u32_fixup(&mut self, value: u32, offset: u32) -> ModuleResult<()> {
        // println!("apply_fixup: {:08x} @ {:08x}", value, offset);
        self.w.write_u32_at(value as u32, offset as usize)
    }

    pub fn load(mut self) -> ModuleResult<Module<'w>> {
        self.load_header()?;        
        while !self.done() {
            let _s = self.load_section()?;
            self.m.extend(self.w.split())
        }
        Ok(self.m)
    }

    pub fn load_header(&mut self) -> ModuleResult<()> {        
        Ok({
            self.read_u32_expecting(MAGIC_COOKIE, Error::InvalidHeader)?;
            self.read_u32_expecting(VERSION, Error::InvalidHeader)?;
        })
    }
    
    pub fn load_section(&mut self) -> ModuleResult<SectionType> {
        // ID(u8) LEN(u32) [LEN]
        Ok({
            let s = SectionType::try_from(self.read_var_u7()?)?;
            let s_len = self.read_var_u32()?;

            self.write_u8(s as u8)?;
            let fixup_len = self.write_u32_fixup()?;
            let s_beg = self.w.pos();
            let pos = self.r.pos();
            match s {
                SectionType::Type => self.load_types()?,
                SectionType::Import => self.load_imports()?,
                SectionType::Function => self.load_functions()?,
                SectionType::Table => self.load_tables()?,
                SectionType::Memory => self.load_memory()?,
                SectionType::Global => self.load_globals()?,
                SectionType::Export => self.load_exports()?,
                SectionType::Start => self.load_start()?,
                SectionType::Element => self.load_elements()?,
                SectionType::Code => self.load_code()?,
                SectionType::Data => self.load_data()?,
                _ => self.r.advance(s_len as usize)
            }
            if self.r.pos() != pos + s_len as usize {
                return Err(Error::UnexpectedData)
            }
            let s_end = self.w.pos();
            let s_len = s_end - s_beg;
            println!("{:08x} to {:08x} (len {:08x})", s_beg, s_end, s_len);
            self.apply_u32_fixup(s_len as u32, fixup_len)?;
            s
        })
    }

    pub fn load_types(&mut self) -> ModuleResult<()> {
        println!("load_types");
        Ok({
            let len = self.copy_var_u32()?;

            for i in 0..len {              
                println!("  {}:", i) ;
                // Read form 
                self.read_var_i7_expecting(FUNC as i8, Error::UnknownSignatureType)?;

                // Copy Parameters 
                let p_len = self.copy_var_u32()?;
                for _ in 0..p_len {
                    println!("    {:02x}", self.copy_var_i7()?);
                }

                // Copy Returns
                let s_len = self.copy_var_u32()?;                
                for _ in 0..s_len {
                    println!("    -> {:02x}", self.copy_var_i7()?);
                }                
            }            
        })
    }

    pub fn copy_resizable_limits(&mut self) -> ModuleResult<()> {    
        // flags
        let f = self.copy_var_u32()?;
        // minimum
        self.copy_var_u32()?;
        if f & 1 != 0 {
            // copy maximum
            self.copy_var_u32()?;
        }
        Ok(())
    }

    pub fn copy_table_description(&mut self) -> ModuleResult<()> {
        // table element type
        self.copy_var_i7()?;
        self.copy_resizable_limits()?;
        Ok(())        
    }

    pub fn copy_linear_memory_description(&mut self) -> ModuleResult<()> {
        self.copy_resizable_limits()?;
        Ok(())
    }

    pub fn copy_global_description(&mut self) -> ModuleResult<()> {
        // value type
        self.copy_var_i7()?;
        self.copy_var_u1()?;
        Ok(())        
    }

    pub fn load_imports(&mut self) -> ModuleResult<()> {
        // println!("load_imports");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                // copy module_name
                self.copy_identifier()?;
                // copy export_name
                self.copy_identifier()?;
                // copy external kind
                let kind = self.copy_var_u7()?;
                match kind {
                    // Function
                    0x00 => { self.copy_var_u32()?; },
                    // Table
                    0x01 => self.copy_table_description()?,
                    // Memory
                    0x02 => self.copy_linear_memory_description()?,                        
                    // Global
                    0x03 => self.copy_global_description()?,
                    _ => return Err(Error::UnknownExternalKind)
                }
            }
        })
    }

    pub fn load_functions(&mut self) -> ModuleResult<()> {
        println!("load_functions");
        Ok({
            let len = self.copy_var_u32()?;
            for i in 0..len {
                // function index
                println!("  {}: {:02x}", i, self.copy_var_u32()?);
            }
        })
    }

    pub fn load_tables(&mut self) -> ModuleResult<()> {
        // println!("load_memory");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                self.copy_table_description()?;
            }
        })
    }

    pub fn load_memory(&mut self) -> ModuleResult<()> {
        // println!("load_memory");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                self.copy_linear_memory_description()?;
            }
        })
    }

    pub fn copy_initializer(&mut self) -> ModuleResult<()> {
        // opcode
        self.copy_u8()?;
        // id
        self.copy_var_u32()?;
        Ok(())
    }

    pub fn load_globals(&mut self) -> ModuleResult<()> {
        // println!("load_globals");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                // value type
                self.copy_var_i7()?;
                // mutability
                self.copy_var_u1()?;
                // initializer
                self.copy_initializer()?;
            }
        })
    }

    pub fn load_exports(&mut self) -> ModuleResult<()> {
        // println!("load_exports");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                // id
                self.copy_identifier()?;
                // kind
                let _kind = self.copy_var_u7()?;
                // index
                self.copy_var_u32()?;
                // todo: validate index based on kind
            }
        })
    }

    pub fn load_start(&mut self) -> ModuleResult<()> {
        // println!("load_start");
        // start index
        self.copy_var_u32()?;
        Ok(())
    }

    pub fn table_type(&self, _index: u32) -> ModuleResult<TypeValue> {
        Err(Error::Unimplemented)
    }

    pub fn load_elements(&mut self) -> ModuleResult<()> {        
        // println!("load_elements");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                // index
                let id = self.copy_var_u32()?;
                // offset initializer
                self.copy_initializer()?;
                // if the table's element type is anyfunc:
                if self.table_type(id)? == TypeValue::AnyFunc {
                    // elems: array of varuint32
                    for _ in 0..self.copy_var_u32()? {
                        self.copy_var_u32()?;
                    }
                }
            }
        })
    }

    pub fn load_code(&mut self) -> ModuleResult<()> {
        Ok({
            println!("Getting info");
            println!("---");
            // for s in self.m.iter() {
            //     println!("{:>12} start=0x{:08x} end=0x{:08x} (size={:08x}) count: {}", 
            //         s.sid, s.off, s.off + s.len, s.len, s.cnt
            //     );
            // }
            println!("---");

            let len = self.copy_var_u32()?;
            for _ in 0..len {
                // body size
                let body_len = self.copy_var_u32()?;
                let body_beg = self.r.pos();
                let body_end = body_beg + body_len as usize;

                println!("body len: {}", body_len);
                // locals

                let mut locals = [TypeValue::default(); 16];
                let mut locals_count = 0;

                for i in 0..self.copy_var_u32()? {
                    println!("local {}", i);
                    let n = self.copy_var_u32()?;
                    let t = TypeValue::from(self.copy_var_i7()?);
                    for _ in 0..n {
                        locals[locals_count] = t;
                        locals_count += 1;
                    }
                }


                let mut labels_buf = [Label::default(); 256];
                let label_stack = Stack::new(&mut labels_buf);

                let mut type_buf = [TypeValue::default(); 256];
                let type_stack = Stack::new(&mut type_buf);
                let mut loader = Loader::new(label_stack, type_stack);

                let signature = VOID;
                let locals = &locals[..locals_count];
                let globals = [];
                let functions = [];
                let signatures = [];
                
                println!("locals: {:?}", locals);
                {
                    let body = &self.r.as_ref()[self.r.pos()..body_end];
                    // for b in body.iter() {
                    //     println!("{:02x}", b);
                    // }
                    let mut r = Reader::new(body);
                    loader.load(signature, &locals, &globals, &functions, &signatures, &mut r, &mut self.w).unwrap();
                }
                self.r.set_pos(body_end);
                println!("Done loading");


                // // body
                // while self.r.pos() < body_end {
                //     self.copy_u8()?;
                // }
            }
        })
    }

    pub fn load_data(&mut self) -> ModuleResult<()> {
        Ok({
           let len = self.copy_var_u32()?;
           for _ in 0..len {
               // index
               self.copy_var_u32()?;
               // offset
               self.copy_initializer()?;
               // data
               for _ in 0..self.copy_var_u32()? {
                   self.copy_u8()?;
               }
           }
        })
    }
}
