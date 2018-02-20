use {Error, TypeValue};

use event::*;
use opcode::*;
use types::*;
use module::{Module, Section, Instr};

use core::convert::TryFrom;

pub type VisitorResult = Result<(), Error>;

pub trait Visitor {
    fn event(&mut self, evt: Event) -> VisitorResult;
}

pub fn visit<D: Visitor>(m: &Module, d: &mut D) -> Result<(), Error> {
    Ok({
        let name = "abc.wasm";
        let version = m.version();
        d.event(Event::Start { name, version })?;
        for s in m.sections() {
            let h = s.header();
            let s_type = h.section_type;
            let s_len = h.buf.len() as u32;
            let s_beg = h.buf.pos() as u32;
            let s_end = s_beg + s_len;
            let s_count = h.count();
            d.event(Event::SectionStart { s_type, s_beg, s_end, s_len, s_count })?;
            match s {
                Section::Type(ref t) => {
                    let c = h.count();
                    
                    d.event(Event::TypesStart { c })?;
                    for (n, sig) in t.iter().enumerate() {
                        let n = n as u32;
                        let form = sig.form();
                        d.event(Event::TypeStart { n , form })?;

                        {
                            let c = sig.parameters().iter().count() as u32;
                            d.event(Event::TypeParametersStart { c })?;
                            for (n, t) in sig.parameters().iter().enumerate() {
                                let n = n as u32;                                                                
                                d.event(Event::TypeParameter { n, t: *t })?;
                            }
                            d.event(Event::TypeParametersEnd {  })?;
                        }
                        {
                            let c = sig.returns().iter().count() as u32;
                            d.event(Event::TypeReturnsStart { c })?;
                            for (n, t) in sig.returns().iter().enumerate() {
                                let n = n as u32;
                                d.event(Event::TypeReturn { n, t: *t })?;
                            }
                            d.event(Event::TypeReturnsEnd {  })?;
                        }
                    }
                    d.event(Event::TypesEnd)?;
                },
                Section::Import(ref imps) => {
                    let c = h.count();
                    
                    d.event(Event::ImportsStart { c })?;
                    for (n, imp) in imps.iter().enumerate() {
                        let n = n as u32;
                        let module = imp.module;
                        let export = imp.export;
                        let desc = imp.desc;
                        d.event(Event::Import { n, module, export, desc })?;
                    }
                    d.event(Event::FunctionsEnd)?;
                },                
                Section::Function(ref f) => {
                    let c = h.count();
                    
                    d.event(Event::FunctionsStart { c })?;
                    for (n, func) in f.iter().enumerate() {
                        let n = n as u32;
                        let index = TypeIndex(func.signature_type_index);
                        d.event(Event::Function { n, index })?;
                    }
                    d.event(Event::FunctionsEnd)?;
                },
                Section::Table(ref t) => {
                    let c = h.count();
                    
                    d.event(Event::TablesStart { c })?;
                    for (n, table) in t.iter().enumerate() {
                        let n = n as u32;
                        let element_type = table.element_type;
                        let limits = table.limits;
                        d.event(Event::Table { n, element_type, limits })?;
                    }

                    d.event(Event::TablesEnd)?;
                },      
                Section::Memory(ref m) => {
                    let c = h.count();
                    
                    d.event(Event::MemsStart { c })?;
                    for (n, mem) in m.iter().enumerate() {
                        let n = n as u32;
                        let limits = mem.limits;
                        d.event(Event::Mem { n, limits })?;                    
                    }

                    d.event(Event::MemsEnd)?;
                },        
                Section::Global(ref g) => {
                    let c = h.count();
                    
                    d.event(Event::GlobalsStart { c })?;
                    for (n, global) in g.iter().enumerate() {
                        let n = n as u32;
                        let global_type = global.global_type;
                        let t = global_type.type_value;
                        let mutability = global_type.mutability;                        
                        let init = global.init;
                        d.event(Event::Global { n, t, mutability, init })?;
                    }

                    d.event(Event::MemsEnd)?;
                },
                Section::Export(ref e) => {
                    let c = h.count();
                    
                    d.event(Event::ExportsStart { c })?;
                    for (n, export) in e.iter().enumerate() {
                        let n = n as u32;
                        let id = export.identifier;
                        let desc = export.export_desc;
                        d.event(Event::Export { n, id, desc })?;
                    }
                    d.event(Event::ExportsEnd)?;
                },
                Section::Start(_) => {
                    let index = FuncIndex(h.count() as u32);
                    d.event(Event::StartFunction { index })?;
                },
                Section::Element(ref e) => {
                    let c = h.count();
                    
                    d.event(Event::ElementsStart { c })?;
                    for (n, element) in e.iter().enumerate() {
                        let n = n as u32;
                        let index = TableIndex(element.table_index);
                        let offset = element.offset;
                        let data = Some(element.data);
                        d.event(Event::Element { n, index, offset, data })?;
                    }
                    d.event(Event::ExportsEnd)?;
                },
                Section::Code(ref code_section) => {
                    let c = h.count();
                    d.event(Event::CodeStart { c })?;
                    for (n, code) in code_section.iter().enumerate() {
                        let n = n as u32;
                        let offset = code.range.start;
                        let size = code.range.end - code.range.start;
                        let locals = code.locals().count() as u32;
                        d.event(Event::Body { n, offset, size, locals })?;
                        for (i, local) in code.locals().enumerate() {
                            let i = i as u32;
                            let n = local.n;
                            let t = local.t;
                            d.event(Event::Local { i, n, t })?;
                        }
                        d.event(Event::InstructionsStart)?;                
                        for expr in code.iter() {
                            let Instr { range, opcode, imm } = expr;
                            let op = Opcode::try_from(opcode)?;
                            let data = &[];
                            let offset = range.start;
                            if !(range.end == code.range.end && opcode == END) {
                                d.event(Event::Instruction(Instruction { offset, data, op: &op, imm }))?;
                            }
                        }
                        d.event(Event::InstructionsEnd)?;
                        d.event(Event::BodyEnd)?;
                    }
                    // for (n, body) in code.iter().enumerate() {                        
                    //     let offset = body.pos()
                    //     let size = self.read_var_u32()?;
                    //     let body_beg = self.r.pos();
                    //     let body_end = body_beg + size as usize;     

                    //     let locals = self.read_count()?;
                    //     self.event(Event::Body { n, offset, size, locals })?;
                    //     for i in 0..locals {
                    //         let n = self.read_count()?;
                    //         let t = self.read_type()?;
                    //         self.event(Event::Local { i, n, t })?;
                    //     }
                    //     self.event(Event::InstructionsStart)?;                
                    //     while self.r.pos() < body_end {
                    //         self.read_instruction(body_end)?;
                    //     }
                    //     self.event(Event::InstructionsEnd)?;

                    //     // TODO: Check that function body ends with the END opcode

                    //     self.event(Event::BodyEnd)?;
                    // }
                    d.event(Event::CodeEnd)?;      
                },
                Section::Data(ref e) => {
                    let c = h.count();
                    d.event(Event::DataSegmentsStart { c })?;
                    for (n, data) in e.iter().enumerate() {
                        let n = n as u32;
                        let index = MemIndex(data.memory_index);
                        let offset = data.offset;
                        let data = data.data;
                        d.event(Event::DataSegment { n, index, offset, data } )?;
                    }
                    d.event(Event::DataSegmentsEnd)?;
                }                    
       
                _ => {}
            }
            d.event(Event::SectionEnd)?;
        }
        d.event(Event::End)?;
    })    
}

// pub fn visit_types<D: Visitor>(m: &Module, d: &mut D) -> Result<(), Error> {
// }