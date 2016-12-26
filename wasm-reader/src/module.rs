use Error;
use Reader;
use section::{self, Section};
use SmallVec; 

pub struct Type {
    params: SmallVec<i8>,
    return_type: Option<i8>,
}
pub struct Function {
    types: SmallVec<u32>,
}
pub struct Memory {
    entries: SmallVec<MemoryType>,
}

pub struct MemoryType {
    flags: u32,
    initial: u32,
    maximum: Option<u32>,
}

pub struct Export {
    entries: SmallVec<ExportEntry>,
}

pub struct ExportEntry {
    //field: &'a [u8],
    kind: u8,
    index: u32,
}

pub struct Code {
    locals: SmallVec<LocalEntry>,
    //code: &'a [u8],
}

pub struct LocalEntry {
    count: u32,
    entry_type: i8,
}

pub struct Module {
    types: SmallVec<Type>,
    functions: SmallVec<Function>,
    memories: SmallVec<Memory>,
    exports: SmallVec<Export>,
    codes: SmallVec<Code>,
}



impl Module {
    pub fn new() -> Self {
        Module {
            types: SmallVec::new(),
            functions: SmallVec::new(),
            memories: SmallVec::new(),
            exports: SmallVec::new(),
            codes: SmallVec::new(),
        }
    }
    pub fn read(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut r = Reader::new(buf);
        r.read_module_header().unwrap();
        while let Ok(s) = r.read_section() {
            match s {
                Section::Code(code_sect) => {
                    let mut s_iter = code_sect.iter().unwrap();
                    let mut locals = SmallVec::new();
                    while let Some(item) = s_iter.next().unwrap() {
                        match item {
                            section::CodeItem::Local(count, local_type) => {
                                locals.push(LocalEntry{count: count, entry_type: local_type});
                            },
                            section::CodeItem::Body(_body) => {
                                let c = Code {
                                    locals: SmallVec::new(),
                                };
                                self.codes.push(c);
                                locals = SmallVec::new();
                            }
                        }
                    }
                }
                _ => unimplemented!()
            }
        }
        Ok(())
    }
}

