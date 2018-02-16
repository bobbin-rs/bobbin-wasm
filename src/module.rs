use {Error, Cursor, FIXUP};
// use types::{Limits, Identifier, Initializer, GlobalType, };
use types::*;
// use memory_inst::MemoryInst;
use writer::Writer;
// use module_inst::ModuleInst;
// use wasm_read::WasmRead;
use opcode::*;


use core::{str, fmt};



// pub struct Module<'a> {
//     name: &'a str,
//     version: u32,
//     buf: &'a [u8],
// }

// impl<'a> Module<'a> {
//     pub fn new() -> Self {
//         let name = "";
//         let version = 0;
//         let buf = &[];
//         Module { name, version, buf }
//     }

//     pub fn name(&self) -> &str {
//         self.name
//     }

//     pub fn set_name(&mut self, name: &'a str) {
//         self.name = name;
//     }

//     pub fn set_version(&mut self, version: u32){
//         self.version = version;
//     }

//     pub fn instantiate<'b, 'mem>(&'a self, buf: &'b mut [u8], memory: &'mem MemoryInst<'mem>) -> Result<(ModuleInst<'a, 'b, 'mem>, &'b mut [u8]), Error> {
//         ModuleInst::new(self, buf, memory)
//     }

//     pub fn extend(&mut self, buf: &'a [u8]) {
//         if self.buf.len() == 0 {
//             self.buf = buf
//         } else {
//             let a_ptr = self.buf.as_ptr();
//             let a_len = self.buf.len();
//             let b_ptr = buf.as_ptr();
//             let b_len = buf.len();
//             unsafe {
//                 assert!(a_ptr.offset(a_len as isize) == b_ptr);
//                 self.buf = slice::from_raw_parts(a_ptr, a_len + b_len)
//             }
//         }
//     }

//     pub fn iter(&self) -> SectionIter {
//         SectionIter { index: 0, buf: Cursor::new(self.buf) }
//     }

//     pub fn section(&self, st: SectionType) -> Option<Section> {
//         self.iter().find(|s| s.section_type == st)
//     }

//     pub fn function_signature_type(&self, index: u32) -> Option<Type> {
//         // info!("function_signature_type: {}", index);

//         let mut i = 0;

//         for s in self.iter() {
//             match s.section_type {
//                 // SectionType::Type => {
//                 //     for t in s.types() {
//                 //         info!("{:?}", t);
//                 //     }
//                 // },
//                 SectionType::Import => {
//                     for import in s.imports() {                        
//                         // info!("checking import: {:?} {:?}", import.module.0, import.export.0);
//                         if let ImportDesc::Type(t) = import.desc {
//                             if i == index {
//                                 // info!("found type: {}", t);
//                                 return self.signature_type(t);
//                             }
//                             i += 1;
//                         }
//                     }
//                 },
//                 SectionType::Function => {
//                     for function in s.functions() {
//                         // info!("checking function");
//                         if i == index {
//                             // info!("found type: {}", function.signature_type_index);
//                             return self.signature_type(function.signature_type_index)
//                         }
//                         i += 1;
//                     }
//                 },
//                 _ => {},
//             }
//         }


//         self.function(index).and_then(|f| self.signature_type(f.signature_type_index))
//     }

//     pub fn with_function_signature_type<T, F: FnOnce(Option<Type>)->T>(&self, index: u32, f: F) -> T {
//         f(self.function_signature_type(index))
//     }

//     pub fn signature_type(&self, index: u32) -> Option<Type> {
//         self.section(SectionType::Type).unwrap().types().nth(index as usize)
//     }

//     pub fn function(&self, index: u32) -> Option<Function> {
//         self.section(SectionType::Function).unwrap().functions().nth(index as usize)
//     }

//     pub fn table(&self, index: u32) -> Option<Table> {
//         self.section(SectionType::Table).unwrap().tables().nth(index as usize)
//     }

//     pub fn linear_memory(&self, index: u32) -> Option<Memory> {
//         self.section(SectionType::Table).unwrap().linear_memories().nth(index as usize)
//     }

//     pub fn global(&self, index: u32) -> Option<Global> {
//         self.section(SectionType::Global).unwrap().globals().nth(index as usize)
//     }

//     pub fn start(&self) -> Option<Function> {
//         self.section(SectionType::Start).and_then(|s| self.function(Cursor::new(s.buf).read_u32()))
//     }

//     pub fn elements(&self, index: u32) -> Option<Element> {
//         self.section(SectionType::Element).unwrap().elements().nth(index as usize)
//     }    

//     pub fn body(&self, index: u32) -> Option<Body> {
//         self.section(SectionType::Code).unwrap().bodies().nth(index as usize)
//     }
    
// }

// impl<'a> fmt::Debug for Module<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             writeln!(f, "<Module name={:?} version={}>", self.name(), self.version)?;
//             for s in self.iter() {
//                 s.fmt(f)?;
//             }
//             writeln!(f, "</Module>")?;
//         })
//     }
// }


// pub struct Section<'a> {
//     pub section_type: SectionType,
//     pub buf: &'a [u8],
// }

// impl<'a> Section<'a> {
//     pub fn new(section_type: SectionType, buf: &'a [u8]) -> Self {
//         Section { section_type, buf }
//     }

//     pub fn types(&self) -> TypeIter<'a> {
//         if let SectionType::Type = self.section_type {
//             TypeIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             TypeIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

//     pub fn imports(&self) -> ImportIter<'a> {
//         if let SectionType::Import = self.section_type {
//             ImportIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             ImportIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }    

//     pub fn functions(&self) -> FunctionIter<'a> {
//         if let SectionType::Function = self.section_type {
//             FunctionIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             FunctionIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

//     pub fn tables(&self) -> TableIter<'a> {
//         if let SectionType::Table = self.section_type {
//             TableIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             TableIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

//     pub fn linear_memories(&self) -> MemoryIter<'a> {
//         if let SectionType::Memory = self.section_type {
//             MemoryIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             MemoryIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }        

//     pub fn globals(&self) -> GlobalIter<'a> {
//         if let SectionType::Global = self.section_type {
//             GlobalIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             GlobalIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }    

//     pub fn exports(&self) -> ExportIter<'a> {
//         if let SectionType::Export = self.section_type {
//             ExportIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             ExportIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

//     pub fn start(&self) -> Start {
//         let function_index = Cursor::new(self.buf).read_u32();
//         Start { function_index }
//     }

//     pub fn elements(&self) -> ElementIter<'a> {
//         if let SectionType::Element = self.section_type {
//             ElementIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             ElementIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

//     pub fn bodies(&self) -> BodyIter<'a> {
//         if let SectionType::Code = self.section_type {
//             BodyIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             BodyIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

//     pub fn data(&self) -> DataIter<'a> {
//         if let SectionType::Data = self.section_type {
//             DataIter { index: 0, buf: Cursor::new(&self.buf[4..]) }
//         } else {
//             DataIter { index: 0, buf: Cursor::new(&[]) }
//         }
//     }

// }

// impl<'a> fmt::Debug for Section<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok({
//             let indent = "  ";
//             writeln!(f, "{}<Section type={:?} size={}>", indent, self.section_type, self.buf.len())?;
//             if self.buf.len() > 0 {
//                 let indent = "    ";
//                 write!(f, "{}", indent)?;
//                 for (_i, b) in self.buf.iter().enumerate() {
//                     write!(f, "{:02x} ", *b)?;
//                 }
//                 writeln!(f, "")?;
//             }
//             match self.section_type {
//                 SectionType::Type => {
//                     for t in self.types() {
//                         t.fmt(f)?;
//                     }
//                 },
//                 SectionType::Import => {
//                     for i in self.imports() {
//                         i.fmt(f)?;
//                     }
//                 },                
//                 SectionType::Function => {
//                     for func in self.functions() {
//                         func.fmt(f)?;
//                     }
//                 },
//                 SectionType::Table => {
//                     for t in self.tables() {
//                         t.fmt(f)?;
//                     }
//                 },                
//                 SectionType::Memory => {
//                     for m in self.linear_memories() {
//                         m.fmt(f)?;
//                     }
//                 },      
//                 SectionType::Global => {
//                     for g in self.globals() {
//                         g.fmt(f)?;
//                     }
//                 },                          
//                 SectionType::Export => {
//                     for e in self.exports() {
//                         e.fmt(f)?;
//                     }
//                 },
//                 SectionType::Start => {
//                     self.start().fmt(f)?;
//                 }                
//                 SectionType::Element => {
//                     for e in self.elements() {
//                         e.fmt(f)?;
//                     }
//                 }                
//                 SectionType::Code => {
//                     for b in self.bodies() {
//                         b.fmt(f)?;
//                     }
//                 },
//                 SectionType::Data => {
//                     for d in self.data() {
//                         d.fmt(f)?;
//                     }
//                 },

//                 _ => {},
//             }
//             writeln!(f, "{}</Section>", indent)?;
//         })
//     }
// }

// pub struct SectionIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for SectionIter<'a> {
//     type Item = Section<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_section())
//         } else {
//             None
//         }
//     }
// }

// pub struct TypeIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for TypeIter<'a> {
//     type Item = Type<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {

//             self.index += 1;
//             Some(self.buf.read_type())
//         } else {
//             None
//         }
//     }
// }


// pub struct ImportIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for ImportIter<'a> {
//     type Item = Import<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_import())
//         } else {
//             None
//         }
//     }
// }


// pub struct FunctionIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for FunctionIter<'a> {
//     type Item = Function;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_function())
//         } else {
//             None
//         }
//     }
// }

// pub struct TableIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for TableIter<'a> {
//     type Item = Table;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_table())
//         } else {
//             None
//         }
//     }
// }


// pub struct MemoryIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for MemoryIter<'a> {
//     type Item = Memory;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_memory())
//         } else {
//             None
//         }
//     }
// }

// pub struct GlobalIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for GlobalIter<'a> {
//     type Item = Global;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_global())
//         } else {
//             None
//         }
//     }
// }


// pub struct ExportIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for ExportIter<'a> {
//     type Item = Export<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_export())
//         } else {
//             None
//         }
//     }
// }


// pub struct ElementIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for ElementIter<'a> {
//     type Item = Element<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_element())
//         } else {
//             None
//         }
//     }
// }

// pub struct BodyIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for BodyIter<'a> {
//     type Item = Body<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;            
//             Some(self.buf.read_body())
//         } else {
//             None
//         }
//     }
// }

// pub struct DataIter<'a> {
//     index: u32,
//     buf: Cursor<'a>,
// }

// impl<'a> Iterator for DataIter<'a> {
//     type Item = Data<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buf.len() > 0 {
//             self.index += 1;
//             Some(self.buf.read_data())
//         } else {
//             None
//         }
//     }
// }


pub trait ModuleWrite {
    fn write_section_type(&mut self, st: SectionType) -> Result<(), Error>;
    fn write_section_start(&mut self, st: SectionType) -> Result<usize, Error>;
    fn write_section_end(&mut self, fixup: usize) -> Result<(), Error>;
    fn write_type(&mut self, t: TypeValue) -> Result<(), Error>;
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn write_identifier(&mut self, id: Identifier) -> Result<(), Error>;
    fn write_initializer(&mut self, init: Initializer) -> Result<(), Error>;
    fn write_opcode(&mut self, op: u8) -> Result<(), Error>;
    fn write_limits(&mut self, limits: Limits) -> Result<(), Error>;
    fn write_table(&mut self, table: Table) -> Result<(), Error>;
    fn write_memory(&mut self, memory: Memory) -> Result<(), Error>;
    fn write_global_type(&mut self, global_type: GlobalType) -> Result<(), Error>;    
    fn write_import_desc(&mut self, desc: ImportDesc) -> Result<(), Error>;
    fn write_import(&mut self, import: Import) -> Result<(), Error>;
    fn write_code_start(&mut self) -> Result<usize, Error>;
    fn write_code_end(&mut self, fixup: usize) -> Result<(), Error>;

    // Code

    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error>;
    fn write_alloca(&mut self, count: u32) -> Result<(), Error>;
    
}

impl<'a> ModuleWrite for Writer<'a> {
    fn write_section_start(&mut self, st: SectionType) -> Result<usize, Error> {
        self.write_section_type(st)?;
        let pos = self.pos();
        self.write_u32(FIXUP)?;
        Ok(pos)
    }    

    fn write_section_end(&mut self, fixup: usize) -> Result<(), Error> {
        Ok({
            let len = self.pos() - (fixup + 4);
            self.write_u32_at(len as u32, fixup)?;
        })
    }

    fn write_section_type(&mut self, st: SectionType) -> Result<(), Error> {
        self.write_u8(st as u8)
    }    
    fn write_type(&mut self, t: TypeValue) -> Result<(), Error> {
        self.write_u8(t as u8)
    }
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Error> {
        Ok({
            self.write_u32(buf.len() as u32)?;
            for b in buf {
                self.write_u8(*b)?;
            }
        })
    }

    fn write_identifier(&mut self, id: Identifier) -> Result<(), Error> {
        self.write_bytes(id.0)
    }

    fn write_initializer(&mut self, init: Initializer) -> Result<(), Error> {
        Ok({
            self.write_opcode(init.opcode)?;
            self.write_i32(init.immediate)?;
            self.write_opcode(init.end)?;
        })
    }

    fn write_opcode(&mut self, op: u8) -> Result<(), Error> {
        self.write_u8(op)
    }

    fn write_limits(&mut self, limits: Limits) -> Result<(), Error> {
        Ok({
            if let Some(max) = limits.max {
                self.write_u32(1)?;
                self.write_u32(limits.min)?;
                self.write_u32(max)?;
            } else {
                self.write_u32(0)?;
                self.write_u32(limits.min)?;            
            }
        })
    }    

    fn write_table(&mut self, table: Table) -> Result<(), Error> {
        Ok({
            self.write_i8(table.element_type as i8)?;
            self.write_limits(table.limits)?;
        })
    }

    fn write_memory(&mut self, memory: Memory) -> Result<(), Error> {
        Ok({
            self.write_limits(memory.limits)?;
        })
    }

    fn write_global_type(&mut self, global_type: GlobalType) -> Result<(), Error> {
        Ok({
            self.write_i8(global_type.type_value as i8)?;
            self.write_u8(global_type.mutability)?;
            
        })
    }

    fn write_import_desc(&mut self, desc: ImportDesc) -> Result<(), Error> {
        Ok({
            match desc {
                ImportDesc::Type(t) => {
                    self.write_u8(0x00)?;
                    self.write_u32(t)?;
                },
                ImportDesc::Table(t) => {
                    self.write_u8(0x01)?;
                    self.write_table(t)?;
                },
                ImportDesc::Memory(m) => {
                    self.write_u8(0x02)?;
                    self.write_memory(m)?;
                },
                ImportDesc::Global(g) => {
                    self.write_u8(0x03)?;                    
                    self.write_global_type(g)?;
                }
            }
        })
    }
    fn write_import(&mut self, import: Import) -> Result<(), Error> {
        Ok({
            self.write_identifier(import.module)?;        
            self.write_identifier(import.export)?;
            self.write_import_desc(import.desc)?;
        })
    }
    
    fn write_code_start(&mut self) -> Result<usize, Error> {
        Ok({
            let pos = self.pos();
            self.write_u32(FIXUP)?;
            pos
        })
    }

    fn write_code_end(&mut self, fixup: usize) -> Result<(), Error> {
        Ok({
            let len = self.pos() - (fixup + 4);
            // info!("code_end pos: {:08x}", self.pos());
            // info!("code_end len: {:08x}", len);
            self.write_u32_at(len as u32, fixup)?;
        })
    }

    // Code

    fn write_drop_keep(&mut self, drop_count: u32, keep_count: u32) -> Result<(), Error> {
        // info!("drop_keep {}, {}", drop_count, keep_count);
        if drop_count == 1 && keep_count == 0 {
            self.write_opcode(DROP)?;            
        } else if drop_count > 0 {
            self.write_opcode(INTERP_DROP_KEEP)?;
            self.write_u32(drop_count as u32)?;
            self.write_u32(keep_count as u32)?;
        }
        Ok(())
    }

    // fn write_end(&mut self) -> Result<(), Error> { self.w.write_opcode(END) }    
    fn write_alloca(&mut self, count: u32) -> Result<(), Error> {
        Ok(
            if count > 0 {
                self.write_opcode(INTERP_ALLOCA)?;
                self.write_u32(count as u32)?;
            }
        )
    }        
}
