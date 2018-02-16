use opcode::*;
use module::*;
use cursor::*;
use types::*;

use core::convert::TryFrom;

pub trait WasmRead<'a> {
    fn read_identifier(&mut self) -> Identifier<'a>;
    fn read_initializer(&mut self) -> Initializer;
    fn read_section_type(&mut self) -> SectionType;
    fn read_signature(&mut self) -> Signature<'a>;
    fn read_type_value(&mut self) -> TypeValue;
    fn read_type_values(&mut self) -> &'a [u8];
    fn read_bytes(&mut self) -> &'a [u8];
    fn read_global_type(&mut self) -> GlobalType;
    fn read_limits(&mut self) -> Limits;
    fn read_section_header(&mut self) -> SectionHeader<'a>;
    fn read_function(&mut self) -> Function;
    fn read_table(&mut self) -> Table;
    fn read_memory(&mut self) -> Memory;
    fn read_import_desc(&mut self) -> ImportDesc;
    fn read_export_desc(&mut self) -> ExportDesc;
    fn read_data(&mut self) -> Data<'a>;
    fn read_element(&mut self) -> Element<'a>;
    fn read_global(&mut self) -> Global;
    fn read_export(&mut self) -> Export<'a>;
    fn read_import(&mut self) -> Import<'a>;
    fn read_body(&mut self) -> Body<'a>;
    fn read_local(&mut self) -> Local;
    fn read_depth(&mut self) -> u32;
    fn read_count(&mut self) -> u32;
    fn read_local_index(&mut self) -> LocalIndex;
    fn read_global_index(&mut self) -> GlobalIndex;
    fn read_func_index(&mut self) -> FuncIndex;
    fn read_type_index(&mut self) -> TypeIndex;
    fn read_instr(&mut self) -> Instr<'a>;
}

impl<'a> WasmRead<'a> for Cursor<'a> {
    fn read_identifier(&mut self) -> Identifier<'a> {
        let len = self.read_var_u32();        
        Identifier(self.slice(len as usize))
    }

    fn read_initializer(&mut self) -> Initializer {
        use opcode::*;
        let opcode = self.read_u8();
        if opcode == I64_CONST || opcode == F64_CONST {
            panic!("64 bit initializers not supported");
        }
        let immediate = self.read_var_i32();
        let end = self.read_u8();
        Initializer { opcode, immediate, end }
    }

    fn read_section_type(&mut self) -> SectionType {
        SectionType::from(self.read_var_u7())
    }

    fn read_signature(&mut self) -> Signature<'a> {
        let form = self.read_type_value();
        let p_len = self.read_var_u32();
        let parameters = self.slice(p_len as usize);
        let r_len = self.read_var_u32();
        let returns = self.slice(r_len as usize);
        Signature { form, parameters, returns }
    }

    fn read_type_value(&mut self) -> TypeValue {
        TypeValue::from(self.read_var_u7())
    }

    fn read_type_values(&mut self) -> &'a [u8] {
        let data_len = self.read_var_u7();
        self.slice(data_len as usize)
    }

    fn read_bytes(&mut self) -> &'a [u8] {
        let data_len = self.read_var_u32();
        self.slice(data_len as usize)
    }

    fn read_global_type(&mut self) -> GlobalType {
        let type_value = self.read_type_value();
        let mutability = self.read_var_u7();
        GlobalType { type_value, mutability }
    }

    fn read_limits(&mut self) -> Limits {
        let flags = self.read_var_u32();
        let min = self.read_var_u32();
        let max = match flags {
            0 => None,
            1 => Some(self.read_var_u32()),
            _ => panic!("Unexpected Flags"),
        };
        Limits { flags, min, max }
    }

    fn read_section_header(&mut self) -> SectionHeader<'a> {
        let section_type = SectionType::from(self.read_var_u7());
        let size = self.read_var_u32();            
        let buf = self.split(size as usize);            
        SectionHeader { section_type, buf }        
    }

    fn read_function(&mut self) -> Function {
        let signature_type_index = self.read_var_u32();
        Function { signature_type_index } 
    }

    fn read_table(&mut self) -> Table {
        let element_type = self.read_type_value();
        let limits = self.read_limits();
        Table { element_type, limits }
    }

    fn read_memory(&mut self) -> Memory {
        let limits = self.read_limits();
        Memory { limits }
    }

    fn read_import_desc(&mut self) -> ImportDesc {
        let kind = self.read_var_u7();
        match kind {
            0x00 => ImportDesc::Type(self.read_var_u32()),
            0x01 => ImportDesc::Table(self.read_table()),
            0x02 => ImportDesc::Memory(self.read_memory()),
            0x03 => ImportDesc::Global(self.read_global_type()),
            _ => panic!("Invalid import type: {:02x}", kind),
        }        
    }

    fn read_export_desc(&mut self) -> ExportDesc {
        let kind = self.read_var_u7();
        let index = self.read_var_u32();

        match kind {
            0x00 => ExportDesc::Function(index),
            0x01 => ExportDesc::Table(index),
            0x02 => ExportDesc::Memory(index),
            0x03 => ExportDesc::Global(index),
            _ => panic!("Invalid export type: {:02x}", kind),
        }
    }

    fn read_data(&mut self) -> Data<'a> {
        let memory_index = self.read_var_u32();
        let offset = self.read_initializer();
        let data = self.read_bytes();
        Data { memory_index, offset, data }
    }

    fn read_element(&mut self) -> Element<'a> {
        let table_index = self.read_var_u32();
        let offset = self.read_initializer();
        let data = self.read_bytes();
        Element { table_index, offset, data }
    }

    fn read_global(&mut self) -> Global {
        let global_type = self.read_global_type();
        let init = self.read_initializer();
        Global { global_type, init }
    }

    fn read_export(&mut self) -> Export<'a> {
        let identifier = self.read_identifier();
        let export_desc = self.read_export_desc();
        Export { identifier, export_desc }
    }

    fn read_import(&mut self) -> Import<'a> {
        let module = self.read_identifier();
        let export = self.read_identifier();
        let desc = self.read_import_desc();
        Import { module, export, desc }    
    }    

    fn read_body(&mut self) -> Body<'a> {
        let size = self.read_var_u32();
        let pos = self.pos();
        let locals_count = self.read_var_u32() as usize;
        let locals = self.split(locals_count * 2);
        let locals_len = self.pos() - pos;
        let expr = self.split((size as usize) - locals_len);

        let range = pos as u32 .. pos as u32 + size;

        Body { range, locals, expr }
    }    

    fn read_local(&mut self) -> Local {
        let n = self.read_var_u32();
        let t = self.read_type_value();
        Local { n, t }
    }

    fn read_depth(&mut self) -> u32 {
        self.read_var_u32()
    }
    fn read_count(&mut self) -> u32 {
        self.read_var_u32()
    }
    fn read_local_index(&mut self) -> LocalIndex {
        LocalIndex(self.read_var_u32())
    }
    fn read_global_index(&mut self) -> GlobalIndex {
        GlobalIndex(self.read_var_u32())
    }
    fn read_func_index(&mut self) -> FuncIndex {
        FuncIndex(self.read_var_u32())
    }
    fn read_type_index(&mut self) -> TypeIndex {
        TypeIndex(self.read_var_u32())
    }

    fn read_instr(&mut self) -> Instr<'a> {
        use self::ImmediateType::*;

        let offset = self.pos() as u32;
        let op = Opcode::try_from(self.read_u8()).unwrap();
        let imm = match op.immediate_type() {
            None => Immediate::None,
            BlockSignature => {
                let signature = self.read_type_value();
                Immediate::Block { signature }
            },
            BranchDepth => {
                let depth = self.read_depth() as u8;
                Immediate::Branch { depth }
            },
            BranchTable => {
                let count = self.read_count() as usize;
                let table = self.slice(count + 1);
                Immediate::BranchTable { table }
            },
            Local => {                
                let index = self.read_local_index();
                Immediate::Local { index }
            },
            Global => {
                let index = self.read_global_index();
                Immediate::Global { index }
            },
            Call => {
                let index = self.read_func_index();
                Immediate::Call { index }
            },
            CallIndirect => {
                let index = self.read_type_index();
                let reserved = self.read_var_u32();
                Immediate::CallIndirect { index, reserved }
            },
            I32 => {
                let value = self.read_var_i32();
                Immediate::I32Const { value }
            },
            F32 => {
                let value = self.read_f32();
                Immediate::F32Const { value }
            },
            I64 => {
                let value = self.read_var_i64();
                Immediate::I64Const { value }
            },
            F64 => {
                let value = self.read_f64();
                Immediate::F64Const { value }
            },
            LoadStore=> {
                let align = self.read_var_u32();
                let offset = self.read_var_u32();
                Immediate::LoadStore { align, offset }
            },
            Memory => {
                let reserved = self.read_var_u1();
                Immediate::Memory { reserved }
            },                
        };
        let end = self.pos() as u32;
        let range = offset..end;
        Instr { range, opcode: op.code, imm }
    }   
}