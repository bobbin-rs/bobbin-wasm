#![feature(wasm_import_module)]

#[wasm_import_module="host"]
extern {
    fn hello();
}

#[no_mangle]
pub extern "C" fn run_hello() {
    unsafe { hello(); }
}