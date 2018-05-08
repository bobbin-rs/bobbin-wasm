#![allow(dead_code)]
#![feature(wasm_import_module)]

#[wasm_import_module="host"]
extern {
    fn write();
    fn led(id: i32);
    fn delay(ms: i32);
}

#[no_mangle]
pub extern "C" fn main() {
    loop {
        unsafe {
            led(1);
            delay(500);
            led(0);
            delay(500);
        }
    }
}