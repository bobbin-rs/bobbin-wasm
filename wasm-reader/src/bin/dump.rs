extern crate wasm_reader;

use std::io::Read;
use std::path::Path;
use std::fs::File;

use wasm_reader::{type_name, Reader};
use wasm_reader::section::TypeSectionItem;

pub fn main() {
    let path = Path::new("testdata/basic.wasm");
    let mut file = File::open(&path).unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let r = Reader::new(data.as_slice()).unwrap();

    println!("Sections:");

    if let Ok(Some(section)) = r.type_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.end() - section.start(),
            section.count().unwrap(),
        );
    }

    if let Ok(Some(section)) = r.function_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.end() - section.start(),
            section.count().unwrap(),
        );
    }

    if let Ok(Some(section)) = r.memory_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.end() - section.start(),
            section.count().unwrap(),
        );
    }

    if let Ok(Some(section)) = r.export_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.end() - section.start(),
            section.count().unwrap(),
        );
    }


    if let Ok(Some(section)) = r.code_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.end() - section.start(),
            section.count().unwrap(),
        );
    }
    println!("");

    println!("Exports:");

    let types = r.type_section().unwrap().unwrap();
    let functions = r.function_section().unwrap().unwrap();

    let exports = r.export_section().unwrap().unwrap();
    let mut exports_iter = exports.iter().unwrap();
    while let Ok(Some(e)) = exports_iter.next() {
        let func = functions.get(e.index).unwrap().unwrap();                 
        print!("{:>8}: {}", 
            e.index,
            String::from_utf8_lossy(e.field),
        );   
        let mut types_iter = types.iter().unwrap();
        while let Ok(Some((n, item))) = types_iter.next() {
            if n != func { continue; }
            match item {
                TypeSectionItem::Form(f) => {
                    print!(" {}", type_name(f));
                }
                TypeSectionItem::ParamType(p) => {
                    print!(" {}", type_name(p));
                }
                TypeSectionItem::ReturnType(r) => {
                    print!(" -> {}", type_name(r));
                }
            }
        }


        println!("");
    }
    println!("");
    

}