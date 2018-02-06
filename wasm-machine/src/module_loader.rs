use {Error, Reader, Writer, TypeValue, Module};
use opcode::{FUNC};

pub const MAGIC_COOKIE: u32 = 0x6d736100;
pub const VERSION: u32 = 0x1;

pub const FIXUP: u32 = 0xffff_ffff;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SectionType {
    Custom = 0x0,
    Type = 0x1,
    Import = 0x2,
    Function = 0x3,
    Table = 0x4,
    Memory = 0x5,
    Global = 0x6,
    Export = 0x7,
    Start = 0x8,
    Element = 0x9,
    Code = 0x0a,
    Data = 0x0b,
}

impl SectionType {
    fn try_from_u32(other: u32) -> ModuleResult<Self> {
        use SectionType::*;
        Ok(
            match other {
                0x00 => Custom,
                0x01 => Type,
                0x02 => Import,
                0x03 => Function,
                0x04 => Table,
                0x05 => Memory,
                0x06 => Global,
                0x07 => Export,
                0x08 => Start,
                0x09 => Element,
                0x0a => Code,
                0x0b => Data,
                _ => return Err(Error::InvalidSection { id: other })                
            }
        )
    }
    fn try_from(other: u8) -> ModuleResult<Self> {
        SectionType::try_from_u32(other as u32)
    }
}

pub type ModuleResult<T> = Result<T, Error>;

pub struct ModuleLoader<'r, 'w> {
    r: Reader<'r>,
    w: Writer<'w>,
}

impl<'r, 'w> ModuleLoader<'r, 'w> {
    pub fn new(r: Reader<'r>, w: Writer<'w>) -> Self {
        ModuleLoader { r, w }
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

    fn apply_u32_fixup(&mut self, offset: u32) -> ModuleResult<()> {
        let pos = self.w.pos();
        self.w.write_u32_at(pos as u32, offset as usize)
    }

    pub fn load(mut self) -> ModuleResult<Module<'w>> {
        self.load_header()?;
        while !self.done() {
            self.load_section()?;
        }
        let r: Reader = self.w.into();
        Ok(Module::new(r))
    }

    pub fn load_header(&mut self) -> ModuleResult<()> {        
        Ok({
            self.read_u32_expecting(MAGIC_COOKIE, Error::InvalidHeader)?;
            self.read_u32_expecting(VERSION, Error::InvalidHeader)?;
        })
    }
    
    pub fn load_section(&mut self) -> ModuleResult<()> {
        Ok({
            let s = SectionType::try_from(self.read_var_u7()?)?;
            let s_len = self.read_var_u32()?;

            println!("load_section: {:?} len: {:02}", s, s_len);

            self.write_u32(s as u32)?;
            let fixup_len = self.write_u32_fixup()?;

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
                _ => self.r.advance(s_len as usize)
            }
            if self.r.pos() != pos + s_len as usize {
                return Err(Error::UnexpectedData)
            }


            self.apply_u32_fixup(fixup_len)?;            
        })
    }

    pub fn load_types(&mut self) -> ModuleResult<()> {
        println!("load_types");
        Ok({
            let len = self.copy_var_u32()?;

            for _ in 0..len {               
                // Read form 
                self.read_var_i7_expecting(FUNC as i8, Error::UnknownSignatureType)?;

                // Copy Parameters 
                let p_len = self.copy_var_u32()?;
                for _ in 0..p_len {
                    self.copy_var_i7()?;
                }

                // Copy Returns
                let s_len = self.copy_var_u32()?;                
                for _ in 0..s_len {
                    self.copy_var_i7()?;                    
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
        println!("load_imports");
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
            for _ in 0..len {
                // function index
                self.copy_var_u32()?;
            }
        })
    }

    pub fn load_tables(&mut self) -> ModuleResult<()> {
        println!("load_memory");
        Ok({
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                self.copy_table_description()?;
            }
        })
    }

    pub fn load_memory(&mut self) -> ModuleResult<()> {
        println!("load_memory");
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
        println!("load_globals");
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
        println!("load_exports");
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
        println!("load_start");
        // start index
        self.copy_var_u32()?;
        Ok(())
    }

    pub fn table_type(&self, _index: u32) -> ModuleResult<TypeValue> {
        Err(Error::Unimplemented)
    }

    pub fn load_elements(&mut self) -> ModuleResult<()> {        
        println!("load_elements");
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
            let len = self.copy_var_u32()?;
            for _ in 0..len {
                // body size
                let body_len = self.copy_var_u32()?;
                let body_end = self.r.pos() + body_len as usize;                
                println!("body_len: {}", body_len);
                // locals
                for _ in 0..self.copy_var_u32()? {
                    // count
                    self.copy_var_u32()?;
                    // type
                    self.copy_var_i7()?;
                }

                // body
                while self.r.pos() < body_end {
                    self.copy_u8()?;
                }
            }
        })
    }
}
