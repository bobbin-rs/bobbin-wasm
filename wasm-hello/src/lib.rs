extern {
    fn hello();
}

// #[no_mangle]
// extern "C" fn add_one(x: i32) -> i32 {
//     x + 1
// }

#[no_mangle]
pub extern "C" fn run_hello() {
    unsafe { hello(); }
}