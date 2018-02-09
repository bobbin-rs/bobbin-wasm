use types::*;
use opcode::{Opcode, Immediate};
use {SectionType, TypeValue};

pub enum Event<'a> {
    Start,
    End,

    SectionStart { s_type: SectionType, s_beg: u32, s_end: u32, s_len: u32},
    SectionEnd,

    TypesStart { c: u32 },
        TypeStart { n: u32, form: TypeValue },
            TypeParametersStart { c: u32 },
                TypeParameter { n: u32, t: TypeValue},
            TypeParametersEnd,
            TypeReturnsStart { c: u32 },
                TypeReturn { n: u32, t: TypeValue },
            TypeReturnsEnd,
        TypeEnd,
    TypesEnd,

    FunctionsStart { c: u32 },
        Function { n: u32, index: TypeIndex },
    FunctionsEnd,

    TablesStart { c: u32 },
        Table { n: u32, element_type: TypeValue, limits: ResizableLimits },
    TablesEnd,

    MemsStart { c: u32 },
        Mem { n: u32, limits: ResizableLimits },
    MemsEnd,

    GlobalsStart { c: u32 },
        Global { n:u32, t: TypeValue, mutability: u8, init: Initializer },
    GlobalsEnd,

    ExportsStart { c: u32 },
        Export { n: u32, id: Identifier<'a>, index: ExternalIndex },
    ExportsEnd,

    StartFunction { index: FuncIndex },

    CodeStart { c: u32 },
        Body { n: u32 , offset: u32, size: u32, locals: u32},
            Local { i: u32, n: u32, t: TypeValue },
            Instruction{ n: u32, offset: u32, data: &'a [u8], op: &'a Opcode, imm: Immediate },
    CodeEnd,

    DataSegmentsStart { c: u32 },
        DataSegment { n: u32, index: MemIndex, offset: Initializer, data: &'a [u8] },
    DataSegmentsEnd,

    ImportsStart { c: u32 },
        Import { n: u32, module: Identifier<'a>, export: Identifier<'a>, index: ExternalIndex },
    ImportsEnd,
    
    ElementsStart { c: u32 },
        Element { n: u32, index: TableIndex, offset: Initializer, data: Option<&'a [u8]> },
    ElementsEnd,
}
