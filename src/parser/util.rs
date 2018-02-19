use core::mem;
use core::slice;

pub fn from_byte_slice<T>(buf: &[u8]) -> &[T] {
    let size = mem::size_of::<T>();
    assert!(buf.len() % size == 0);
    let t_len = buf.len() / size;
    unsafe { slice::from_raw_parts(buf.as_ptr() as *const T, t_len) }
}

// pub fn into_byte_slice<T>(buf: &[T]) -> &[u8] {
//     debug_assert!(mem::size_of::<T>() == 1);
//     unsafe { slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) }
// }