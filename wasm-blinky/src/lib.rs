#![allow(dead_code)]
#![feature(wasm_import_module)]

const DIGITS: &[u8; 16] = b"0123456789abcdef";    

mod host {
    #[wasm_import_module="host"]
    extern {
        pub fn write(ptr: *const u8, len: i32);
        pub fn led(id: i32);
        pub fn delay(ms: i32);
    }
}

fn u8_to_hex(c: u8) -> [u8; 2] {
    [DIGITS[((c >> 4) & 0xf) as usize], DIGITS[(c & 0xf) as usize]]
}

fn u16_to_hex(c: u16) -> [u8; 4] {
    let (a, b) = (u8_to_hex((c >> 8) as u8), u8_to_hex(c as u8));
    [a[0], a[1], b[0], b[1]]
}

fn u32_to_hex(c: u32) -> [u8; 8] {
    let (a, b) = (u16_to_hex((c >> 16) as u16), u16_to_hex(c as u16));
    [a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]]
}

fn write(buf: &[u8]) {
    unsafe { host::write(buf.as_ptr(), buf.len() as i32) };
}

fn write_str(buf: &str) {
    write(buf.as_bytes())
}

fn write_u8(value: u8) {
    write(u8_to_hex(value).as_ref())
}

fn write_hex(value: u32) {
    let buf = u32_to_hex(value);
    write(buf.as_ref());
}

fn led(state: bool) {
    unsafe { host::led(if state { 1 } else { 0 }) }
}

fn delay(ms: u32) {
    unsafe { host::delay(ms as i32) }
}

#[no_mangle]
pub extern "C" fn main() {
    let mut i = 0x10u8;
    loop {
        write_u8(i);
        write_str("\n");
        led(true);
        delay(500);
        led(false);
        delay(500);
        // i = i.wrapping_add(1);
    }
}