extern crate wasm_reader;

use std::io::Read;
use std::path::Path;
use std::fs::File;

use wasm_reader::{type_name, Reader};
use wasm_reader::section::{TypeSectionItem, CodeItem};

pub fn main() {
    let path = Path::new("testdata/basic.wasm");
    let mut file = File::open(&path).unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let r = Reader::new(data.as_slice()).unwrap();

    println!("Sections:");

    if let Some(section) = r.type_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.len(),
            section.count(),
        );
    }

    if let Some(section) = r.function_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.len(),
            section.count(),
        );
    }

    if let Some(section) = r.memory_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.len(),
            section.count(),
        );
    }

    if let Some(section) = r.export_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.len(),
            section.count(),
        );
    }


    if let Some(section) = r.code_section() {
        println!("{:>09} start=0x{:08x} end=0x{:08x} (size=0x{:08x}) count: {}", 
            section.name(),
            section.start(),
            section.end(),
            section.len(),
            section.count(),
        );
    }
    println!("");

    println!("Exports:");

    let types = r.type_section().unwrap();
    let functions = r.function_section().unwrap();

    let exports = r.export_section().unwrap();
    let mut exports_iter = exports.iter();
    while let Some(e) = exports_iter.next() {
        let func = functions.get(e.index).unwrap();                 
        print!("{:>8}: {}", 
            e.index,
            String::from_utf8_lossy(e.field),
        );   
        let mut types_iter = types.iter();
        while let Some((n, item)) = types_iter.next() {
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
    println!("Code Disassembly:");
    let code_section = r.code_section().unwrap();
    let mut code_iter = code_section.iter();
    let mut i = 0;
    while let Some(item) = code_iter.next() {
        println!("func {}", i);
        match item {
            CodeItem::Body(body) => {
                for b in body.bytes() {
                    println!(" {:02x}", b.unwrap());
                }
            }
            _ => unimplemented!()
        }
        i += 1;
    }
}