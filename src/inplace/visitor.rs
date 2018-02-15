use Error;

use Delegate;
use event::*;
use super::*;

pub fn visit<D: Delegate>(m: &Module, d: &mut D) -> Result<(), Error> {
    Ok({
        let name = "abc.wasm";
        let version = m.version;
        d.dispatch(Event::Start { name, version })?;
        for s in m.sections() {
            let h = s.header();
            let s_type = h.section_type;
            let s_len = h.buf.len() as u32;
            let s_beg = h.buf.pos() as u32;
            let s_end = s_beg + s_len;
            let s_count = h.count();
            d.dispatch(Event::SectionStart { s_type, s_beg, s_end, s_len, s_count })?;
            
            match s {
                Section::Type(ref t) => {
                    let c = h.count();
                    
                    d.dispatch(Event::TypesStart { c })?;
                    for (n, sig) in t.iter().enumerate() {
                        let n = n as u32;
                        let form = sig.form;
                        d.dispatch(Event::TypeStart { n , form })?;

                        {
                            let c = sig.parameters.len() as u32;
                            d.dispatch(Event::TypeParametersStart { c })?;
                            for (n, t) in sig.parameters.iter().enumerate() {
                                let n = n as u32;
                                let t = TypeValue::from(*t);
                                d.dispatch(Event::TypeParameter { n, t })?;
                            }
                            d.dispatch(Event::TypeParametersEnd {  })?;
                        }
                        {
                            let c = sig.returns.len() as u32;
                            d.dispatch(Event::TypeReturnsStart { c })?;
                            for (n, t) in sig.returns.iter().enumerate() {
                                let n = n as u32;
                                let t = TypeValue::from(*t);
                                d.dispatch(Event::TypeReturn { n, t })?;
                            }
                            d.dispatch(Event::TypeReturnsEnd {  })?;
                        }
                    }
                    d.dispatch(Event::TypesEnd)?;
                },
                Section::Import(ref imps) => {
                    let c = h.count();
                    
                    d.dispatch(Event::ImportsStart { c })?;
                    for (n, imp) in imps.iter().enumerate() {
                        let n = n as u32;
                        let module = imp.module;
                        let export = imp.export;
                        let desc = imp.desc;
                        d.dispatch(Event::Import { n, module, export, desc })?;
                    }
                    d.dispatch(Event::FunctionsEnd)?;
                },                
                Section::Function(ref f) => {
                    let c = h.count();
                    
                    d.dispatch(Event::FunctionsStart { c })?;
                    for (n, func) in f.iter().enumerate() {
                        let n = n as u32;
                        let index = TypeIndex(func.signature_type_index);
                        d.dispatch(Event::Function { n, index })?;
                    }
                    d.dispatch(Event::FunctionsEnd)?;
                },
                Section::Table(ref t) => {
                    let c = h.count();
                    
                    d.dispatch(Event::TablesStart { c })?;
                    for (n, table) in t.iter().enumerate() {
                        let n = n as u32;
                        let element_type = table.element_type;
                        let limits = table.limits;
                        d.dispatch(Event::Table { n, element_type, limits })?;
                    }

                    d.dispatch(Event::TablesEnd)?;
                },      
                Section::Memory(ref m) => {
                    let c = h.count();
                    
                    d.dispatch(Event::MemsStart { c })?;
                    for (n, mem) in m.iter().enumerate() {
                        let n = n as u32;
                        let limits = mem.limits;
                        d.dispatch(Event::Mem { n, limits })?;                    
                    }

                    d.dispatch(Event::MemsEnd)?;
                },        
                Section::Global(ref g) => {
                    let c = h.count();
                    
                    d.dispatch(Event::GlobalsStart { c })?;
                    for (n, global) in g.iter().enumerate() {
                        let n = n as u32;
                        let global_type = global.global_type;
                        let t = global_type.type_value;
                        let mutability = global_type.mutability;                        
                        let init = global.init;
                        d.dispatch(Event::Global { n, t, mutability, init })?;
                    }

                    d.dispatch(Event::MemsEnd)?;
                },
                Section::Export(ref e) => {
                    let c = h.count();
                    
                    d.dispatch(Event::ExportsStart { c })?;
                    for (n, export) in e.iter().enumerate() {
                        let n = n as u32;
                        let id = export.identifier;
                        let desc = export.export_desc;
                        d.dispatch(Event::Export { n, id, desc })?;
                    }
                    d.dispatch(Event::ExportsEnd)?;
                },
                Section::Start(_) => {
                    let index = FuncIndex(h.count() as u32);
                    d.dispatch(Event::StartFunction { index })?;
                },
                Section::Element(ref e) => {
                    let c = h.count();
                    
                    d.dispatch(Event::ElementsStart { c })?;
                    for (n, element) in e.iter().enumerate() {
                        let n = n as u32;
                        let index = TableIndex(element.table_index);
                        let offset = element.offset;
                        let data = Some(element.data);
                        d.dispatch(Event::Element { n, index, offset, data })?;
                    }
                    d.dispatch(Event::ExportsEnd)?;
                },
                Section::Code(ref _code) => {
                    let c = h.count();
                    d.dispatch(Event::CodeStart { c })?;
                    // for (n, body) in code.iter().enumerate() {                        
                    //     let offset = body.pos()
                    //     let size = self.read_var_u32()?;
                    //     let body_beg = self.r.pos();
                    //     let body_end = body_beg + size as usize;     

                    //     let locals = self.read_count()?;
                    //     self.dispatch(Event::Body { n, offset, size, locals })?;
                    //     for i in 0..locals {
                    //         let n = self.read_count()?;
                    //         let t = self.read_type()?;
                    //         self.dispatch(Event::Local { i, n, t })?;
                    //     }
                    //     self.dispatch(Event::InstructionsStart)?;                
                    //     while self.r.pos() < body_end {
                    //         self.read_instruction(body_end)?;
                    //     }
                    //     self.dispatch(Event::InstructionsEnd)?;

                    //     // TODO: Check that function body ends with the END opcode

                    //     self.dispatch(Event::BodyEnd)?;
                    // }
                    d.dispatch(Event::CodeEnd)?;                    
                },
                Section::Data(ref e) => {
                    let c = h.count();
                    d.dispatch(Event::DataSegmentsStart { c })?;
                    for (n, data) in e.iter().enumerate() {
                        let n = n as u32;
                        let index = MemIndex(data.memory_index);
                        let offset = data.offset;
                        let data = data.data;
                        d.dispatch(Event::DataSegment { n, index, offset, data } )?;
                    }
                    d.dispatch(Event::DataSegmentsEnd)?;
                }                    
       
                _ => {}
            }
            d.dispatch(Event::SectionEnd)?;
        }
        d.dispatch(Event::End)?;
    })    
}

// pub fn visit_types<D: Delegate>(m: &Module, d: &mut D) -> Result<(), Error> {
// }