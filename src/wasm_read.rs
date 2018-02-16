// use {SectionType, TypeValue};
// use cursor::*;
// use types::*;
// use module::*;

// pub trait WasmRead<'a> {
//     fn read_identifier(&mut self) -> Identifier<'a>;
//     fn read_initializer(&mut self) -> Initializer;
//     fn read_section_type(&mut self) -> SectionType;
//     fn read_type_value(&mut self) -> TypeValue;
//     fn read_type_values(&mut self) -> &'a [u8];
//     fn read_bytes(&mut self) -> &'a [u8];
//     fn read_global_type(&mut self) -> GlobalType;
//     fn read_limits(&mut self) -> Limits;
//     fn read_section(&mut self) -> Section<'a>;
//     fn read_type(&mut self) -> Type<'a>;
//     fn read_function(&mut self) -> Function;
//     fn read_table(&mut self) -> Table;
//     fn read_memory(&mut self) -> Memory;
//     fn read_import_desc(&mut self) -> ImportDesc;
//     fn read_export_desc(&mut self) -> ExportDesc;
//     fn read_data(&mut self) -> Data<'a>;
//     fn read_element(&mut self) -> Element<'a>;
//     fn read_global(&mut self) -> Global;
//     fn read_export(&mut self) -> Export<'a>;
//     fn read_import(&mut self) -> Import<'a>;
//     fn read_body(&mut self) -> Body<'a>;
// }

// impl<'a> WasmRead<'a> for Cursor<'a> {
//     fn read_identifier(&mut self) -> Identifier<'a> {
//         let len = self.read_u32();        
//         Identifier(self.slice(len as usize))
//     }

//     fn read_initializer(&mut self) -> Initializer {
//         let opcode = self.read_u8();
//         let immediate = self.read_i32();
//         let end = self.read_u8();
//         Initializer { opcode, immediate, end }
//     }

//     fn read_section_type(&mut self) -> SectionType {
//         SectionType::from(self.read_u8())
//     }

//     fn read_type_value(&mut self) -> TypeValue {
//         TypeValue::from(self.read_u8())
//     }

//     fn read_type_values(&mut self) -> &'a [u8] {
//         let data_len = self.read_u8();
//         self.slice(data_len as usize)
//     }

//     fn read_bytes(&mut self) -> &'a [u8] {
//         let data_len = self.read_u32();
//         self.slice(data_len as usize)
//     }

//     fn read_global_type(&mut self) -> GlobalType {
//         let type_value = self.read_type_value();
//         let mutability = self.read_u8();
//         GlobalType { type_value, mutability }
//     }

//     fn read_limits(&mut self) -> Limits {
//         let flags = self.read_u32();
//         let min = self.read_u32();
//         let max = match flags {
//             0 => None,
//             1 => Some(self.read_u32()),
//             _ => panic!("Unexpected Flags"),
//         };
//         Limits { flags, min, max }
//     }

//     fn read_section(&mut self) -> Section<'a> {
//         let section_type = self.read_section_type();
//         let buf = self.read_bytes();
//         Section { section_type, buf }
//     }    

//     fn read_type(&mut self) -> Type<'a> {
//         let parameters = self.read_type_values();
//         let returns = self.read_type_values();
//         Type { parameters, returns }
//     }

//     fn read_function(&mut self) -> Function {
//         let signature_type_index = self.read_u32();
//         Function { signature_type_index } 
//     }

//     fn read_table(&mut self) -> Table {
//         let element_type = self.read_type_value();
//         let limits = self.read_limits();
//         Table { element_type, limits }
//     }

//     fn read_memory(&mut self) -> Memory {
//         let limits = self.read_limits();
//         Memory { limits }
//     }

//     fn read_import_desc(&mut self) -> ImportDesc {
//         let kind = self.read_u8();
//         match kind {
//             0x00 => ImportDesc::Type(self.read_u32()),
//             0x01 => ImportDesc::Table(self.read_table()),
//             0x02 => ImportDesc::Memory(self.read_memory()),
//             0x03 => ImportDesc::Global(self.read_global_type()),
//             _ => panic!("Invalid import type: {:02x}", kind),
//         }        
//     }

//     fn read_export_desc(&mut self) -> ExportDesc {
//         let kind = self.read_u8();
//         let index = self.read_u32();

//         match kind {
//             0x00 => ExportDesc::Function(index),
//             0x01 => ExportDesc::Table(index),
//             0x02 => ExportDesc::Memory(index),
//             0x03 => ExportDesc::Global(index),
//             _ => panic!("Invalid export type: {:02x}", kind),
//         }
//     }

//     fn read_data(&mut self) -> Data<'a> {
//         let memory_index = self.read_u32();
//         let offset = self.read_initializer();
//         let data = self.read_bytes();
//         Data { memory_index, offset, data }
//     }

//     fn read_element(&mut self) -> Element<'a> {
//         let table_index = self.read_u32();
//         let offset = self.read_initializer();
//         let data = self.read_bytes();
//         Element { table_index, offset, data }
//     }

//     fn read_global(&mut self) -> Global {
//         let global_type = self.read_global_type();
//         let init = self.read_initializer();
//         Global { global_type, init }
//     }

//     fn read_export(&mut self) -> Export<'a> {
//         let identifier = self.read_identifier();
//         let export_desc = self.read_export_desc();
//         Export { identifier, export_desc }
//     }

//     fn read_import(&mut self) -> Import<'a> {
//         let module = self.read_identifier();
//         let export = self.read_identifier();
//         let desc = self.read_import_desc();
//         Import { module, export, desc }    
//     }    
//     fn read_body(&mut self) -> Body<'a> {
//         let buf = self.read_bytes();
//         Body { buf }
//     }
    
// }