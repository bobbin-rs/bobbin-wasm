#![allow(dead_code)]
#![feature(wasm_import_module)]

mod host {
    #[wasm_import_module="host"]
    extern {
        pub fn write(ptr: *const u8, len: i32);
        pub fn led(id: i32);
        pub fn delay(ms: i32);
    }
}

fn write(buf: &[u8]) {
    unsafe { host::write(buf.as_ptr(), buf.len() as i32) };
}

fn write_str(buf: &str) {
    write(buf.as_bytes())
}

fn led(state: bool) {
    unsafe { host::led(if state { 1 } else { 0 }) }
}

fn delay(ms: u32) {
    unsafe { host::delay(ms as i32) }
}

#[no_mangle]
pub extern "C" fn main() {
    loop {
        write_str("Hello, World\n");
        led(true);
        delay(500);
        led(false);
        delay(500);
    }
}