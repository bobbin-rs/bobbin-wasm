extern crate wasm_reader;

use std::io::Read;
use std::path::Path;
use std::fs::File;

use wasm_reader::Reader;

pub fn main() {
    let path = Path::new("testdata/basic.wasm");
    let mut file = File::open(&path).unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let r = Reader::new(data.as_slice()).unwrap();

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

    let exports = r.export_section().unwrap().unwrap();
    let mut exports_iter = exports.iter().unwrap();
    while let Ok(Some(e)) = exports_iter.next() {
        println!("Name: {:?} Kind: {} Index: {}", 
            String::from_utf8_lossy(e.field),
            e.kind,
            e.index
        );        
    }
    

}