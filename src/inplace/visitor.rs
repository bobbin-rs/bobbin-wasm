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