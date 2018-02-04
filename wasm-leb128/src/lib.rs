#![allow(unused_imports, unused_variables)]
#![feature(core_intrinsics)]
#![no_std]

use core::intrinsics::ctlz;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidEncoding,
    BufferSize,
    Overflow,
}

#[inline]
pub fn read_u1(buf: &[u8]) -> Result<(bool, usize), Error> {
    assert!(buf[0] & !0x1 == 0, "Invalid Encoding: 0x{:2x}", buf[0]);
    if !(buf[0] & !0x1 == 0) { return Err(Error::InvalidEncoding) }
    Ok((buf[0] != 0, 1))
}

#[inline]
pub fn read_u7(buf: &[u8]) -> Result<(u8, usize), Error> {
    if !(buf[0] & 1 << 7 == 0) { return Err(Error::Overflow) }
    Ok((buf[0] & !(1 << 7), 1))
}

#[inline]
pub fn read_i7(buf: &[u8]) -> Result<(i8, usize), Error> {
    if !(buf[0] & 1 << 7 == 0) { return Err(Error::Overflow) }
    let mut result = buf[0] & 0b0111_1111;
    if result & 0b0100_0000 != 0 {        
        unsafe {
            let shift = 8 - ctlz(buf[0] & !0b0100_000);
            result |= ((1 << shift) as u8).wrapping_neg();
        }
    }
    Ok((result as i8, 1))
}

#[inline]
pub fn read_u32(buf: &[u8]) -> Result<(u32, usize), Error> {
    let mut i = 0;
    let mut result = 0;    
    let mut shift = 0;
    loop {
        let b = buf[i];
        result |= ((b & 0b0111_1111) as u32) << shift;
        shift += 7;
        i += 1;
        if b & 0b1000_0000 == 0 {
            break;
        }
    }
    unsafe {
        let size = shift + 1 - ctlz(buf[i-1]);
        if !(size <= 32) { return Err(Error::Overflow) }
    }
    Ok((result, i))
}

#[inline]
pub fn read_i32(buf: &[u8]) -> Result<(i32, usize), Error> {
    const SIGN_BIT: u8 = 0b0100_0000;

    let mut i = 0;
    let mut result: i32 = 0;    
    let mut shift = 0;
    loop {
        let b = buf[i];
        result |= ((b & 0b0111_1111) as i32) << shift;
        shift += 7;
        i += 1;
        if b & 0b1000_0000 == 0 {
            break;
        }
    }
    let last_byte = buf[i-1];    
    unsafe {
        let size = if (last_byte & SIGN_BIT) == 0 {
            shift + 1 - ctlz(last_byte) as usize
        } else {
            shift + 2 - ctlz(!(last_byte | 0b1000_0000)) as usize
        };
        if !(size <= 32) { return Err(Error::Overflow) }
    }   
    if shift < 32 && (last_byte & 0b0100_0000) != 0 {
        result |= ((1 << shift) as i32).wrapping_neg();
    }
    Ok((result, i))
}

#[inline]
pub fn write_u1(buf: &mut [u8], value: bool) -> Result<usize, Error> {
    if buf.len() < 1 { return Err(Error::BufferSize) }
    buf[0] = if value { 1 } else { 0 };
    Ok(1)
}

#[inline]
pub fn write_u7(buf: &mut [u8], value: u8) -> Result<usize, Error> {
    if buf.len() < 1 { return Err(Error::BufferSize) }
    if !(value & !0b0111_1111 == 0) { return Err(Error::Overflow) }
    buf[0] = value;
    Ok(1)
}

#[inline]
pub fn write_i7(buf: &mut [u8], value: i8) -> Result<usize, Error> {
    if buf.len() < 1 { return Err(Error::BufferSize) }
    if !(if value >= 0 { value < 64 } else { value >= -64 }) { return Err(Error::Overflow) }
    buf[0] = value as u8 & 0b0111_1111;
    Ok(1)
}

#[inline]
pub fn write_u32(buf: &mut [u8], mut value: u32) -> Result<usize, Error> {
    let mut i = 0;
    loop {
        let mut b = value as u8 & 0b0111_1111;
        value >>= 7;
        if value != 0 {
            b |= 0b1000_0000;
        }
        if buf.len() == i { return Err(Error::BufferSize) }
        buf[i] = b;
        i += 1;
        if value == 0 {
            return Ok(i)
        }
    }
}

#[inline]
pub fn write_i32(buf: &mut [u8], mut value: i32) -> Result<usize, Error> {
    const SIGN_BIT: u8 = 0b0100_0000;
    let mut i = 0;
    let mut more = true;
    loop {
        let mut b = value as u8 & 0b0111_1111;
        value >>= 7;
        if (value == 0 && b & SIGN_BIT == 0) ||
            (value == -1 && b & SIGN_BIT != 0) {
                more = false;
        } else {
            b |= 0b1000_0000;
        }
        if buf.len() == i { return Err(Error::BufferSize) }
        buf[i] = b;
        i += 1;
        if !more {
            return Ok(i)
        }
    }    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u1() {
        assert_eq!(read_u1(&[0b0000_0000]).unwrap(), (false, 1));
        assert_eq!(read_u1(&[0b0000_0001]).unwrap(), (true, 1));
    }

    #[test]
    fn test_read_u7() {
        for i in 0u8..128 {
            assert_eq!(read_u7(&[i]).unwrap(), (i, 1));            
        }
    }

    #[test]
    fn test_read_i7() {
        for i in 0u8..64 {
            assert_eq!(read_i7(&[i]).unwrap(), (i as i8, 1));            
        }
        assert_eq!(read_i7(&[0b0111_1111]).unwrap(), (-1, 1));            
        assert_eq!(read_i7(&[0b0111_1110]).unwrap(), (-2, 1));            
        assert_eq!(read_i7(&[0b0111_1100]).unwrap(), (-4, 1));
        assert_eq!(read_i7(&[0b0111_1000]).unwrap(), (-8, 1));
        assert_eq!(read_i7(&[0b0111_0000]).unwrap(), (-16, 1));
        assert_eq!(read_i7(&[0b0110_0000]).unwrap(), (-32, 1));
        assert_eq!(read_i7(&[0b0100_0000]).unwrap(), (-64, 1));
    }    

    #[test]
    fn test_write_u1() {
        let mut buf = [0u8; 8];
        assert_eq!(write_u1(&mut buf, false).unwrap(), 1);
        assert_eq!(buf[0], 0);
        assert_eq!(write_u1(&mut buf, true).unwrap(), 1);
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn test_write_u7() {
        for i in 0..128 {
            let mut buf = [0u8; 8];            
            assert_eq!(write_u7(&mut buf, i).unwrap(), 1);
            assert_eq!(buf[0], i);
        }        
    }

    #[test]
    fn test_write_i7() {
        let mut buf = [0u8; 8];            
        for i in 0..64 {
            assert_eq!(write_i7(&mut buf, i).unwrap(), 1);
            assert_eq!(buf[0], i as u8);
        }       
        assert_eq!(write_i7(&mut buf, -1).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1111);
        assert_eq!(write_i7(&mut buf, -2).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1110);
        assert_eq!(write_i7(&mut buf, -4).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1100);
        assert_eq!(write_i7(&mut buf, -8).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_1000);
        assert_eq!(write_i7(&mut buf, -16).unwrap(), 1);
        assert_eq!(buf[0], 0b0111_0000);
        assert_eq!(write_i7(&mut buf, -32).unwrap(), 1);
        assert_eq!(buf[0], 0b0110_0000);
        assert_eq!(write_i7(&mut buf, -64).unwrap(), 1);
        assert_eq!(buf[0], 0b0100_0000);
    }

    #[test]
    fn test_write_u32() {
        let mut buf = [0xffu8; 8];

        // 0-7 bits
        assert_eq!(write_u32(&mut buf, 0b000000).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0000]);
        assert_eq!(write_u32(&mut buf, 0b000001).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1111111).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0111_1111]);

        // 8-14 bits
        assert_eq!(write_u32(&mut buf, 0b1_0000000).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0000_0001]);

        // 15-21 bits
        assert_eq!(write_u32(&mut buf, 0b1_0000000_0000000).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_0000000_1111111).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1111_1111, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111_0000000).unwrap(), 3);
        assert_eq!(&buf[..3], &[0b1000_0000, 0b1111_1111, 0b0000_0001]);

        // 22-28 bits
        assert_eq!(write_u32(&mut buf, 0b1_0000000_0000000_0000000).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_0000100_0000010_0000001).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111_1111111_1111111).unwrap(), 4);
        assert_eq!(&buf[..4], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001]);

        // 29-32 bits    
        assert_eq!(write_u32(&mut buf, 0b1_0000000_0000000_0000000_0000000).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_0001000_0000100_0000010_0000001).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b1000_1000, 0b0000_0001]);
        assert_eq!(write_u32(&mut buf, 0b1_1111111_1111111_1111111_1111111).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_0001]);

        // 32 bits    
        assert_eq!(write_u32(&mut buf, 0b1000_0000000_0000000_0000000_0000000).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_1000]);
        assert_eq!(write_u32(&mut buf, 0b1000_0001000_0000100_0000010_0000001).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1000_0001, 0b1000_0010, 0b1000_0100, 0b1000_1000, 0b0000_1000]);
        assert_eq!(write_u32(&mut buf, 0b1111_1111111_1111111_1111111_1111111).unwrap(), 5);
        assert_eq!(&buf[..5], &[0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111]);
    }

    #[test]
    fn test_read_u32() {
        // 0-7 bits
        assert_eq!(read_u32(&[0b0000000]).unwrap(), (0b0000000, 1));
        assert_eq!(read_u32(&[0b1111111]).unwrap(), (0b1111111, 1));

        // 8-14 bits
        assert_eq!(read_u32(&[0b10000000, 0b0000001]).unwrap(), (0b1_0000000, 2));
        assert_eq!(read_u32(&[0b10000001, 0b0000001]).unwrap(), (0b1_0000001, 2));
        assert_eq!(read_u32(&[0b11111111, 0b0000001]).unwrap(), (0b1_1111111, 2));

        // 15-21 bits
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b0000001]).unwrap(), (0b1_0000000_0000000, 3));
        assert_eq!(read_u32(&[0b10000001, 0b10000000, 0b0000001]).unwrap(), (0b1_0000000_0000001, 3));
        assert_eq!(read_u32(&[0b10000000, 0b10000001, 0b0000001]).unwrap(), (0b1_0000001_0000000, 3));
        assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b0000001]).unwrap(), (0b1_1111111_1111111, 3));

        // 22-28 bits
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b10000000, 0b0000001]).unwrap(), (0b1_0000000_0000000_0000000, 4));
        assert_eq!(read_u32(&[0b10000001, 0b10000000, 0b10000000, 0b0000001]).unwrap(), (0b1_0000000_0000000_0000001, 4));
        assert_eq!(read_u32(&[0b10000000, 0b10000001, 0b10000000, 0b0000001]).unwrap(), (0b1_0000000_0000001_0000000, 4));
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b10000001, 0b0000001]).unwrap(), (0b1_0000001_0000000_0000000, 4));
        assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b11111111, 0b0000001]).unwrap(), (0b1_1111111_1111111_1111111, 4));

        // 29-32 bits
        assert_eq!(read_u32(&[0b10000000, 0b10000000, 0b10000000, 0b10000000, 0b00001000]).unwrap(), (0b1000_0000000_0000000_0000000_0000000, 5));
        assert_eq!(read_u32(&[0b10000001, 0b10000010, 0b10000100, 0b10001000, 0b00001000]).unwrap(), (0b1000_0001000_0000100_0000010_0000001, 5));
        assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00001111]).unwrap(), (0b1111_1111111_1111111_1111111_1111111, 5));
        //assert_eq!(read_u32(&[0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00010000]), (0b1111_1111111_1111111_1111111_1111111, 5));
    }

    #[test]
    fn test_write_i32() {
        let mut buf = [0xffu8; 8];

        // 0-7 bits
        assert_eq!(write_i32(&mut buf, 0b0000000).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0000]);
        assert_eq!(write_i32(&mut buf, 0b0000001).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0000_0001]);
        assert_eq!(write_i32(&mut buf, 0b0111111).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0011_1111]);
        assert_eq!(write_i32(&mut buf, -1).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0111_1111]);
        assert_eq!(write_i32(&mut buf, -64).unwrap(), 1);
        assert_eq!(&buf[..1], &[0b0100_0000]);

        // 8-14 bits
        assert_eq!(write_i32(&mut buf, 0b1_0000000).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0000_0001]);
        assert_eq!(write_i32(&mut buf, 0b1_1111111).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0000_0001]);
        assert_eq!(write_i32(&mut buf, -127).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0001, 0b0111_1111]);
        assert_eq!(write_i32(&mut buf, -128).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1000_0000, 0b0111_1111]);
        assert_eq!(write_i32(&mut buf, -129).unwrap(), 2);
        assert_eq!(&buf[..2], &[0b1111_1111, 0b0111_1110]);
    }

    #[test]
    fn test_read_i32() {
        // 0-7 bits
        assert_eq!(read_i32(&[0x0]).unwrap(), (0, 1));
        assert_eq!(read_i32(&[0x2]).unwrap(), (2, 1));
        assert_eq!(read_i32(&[0x7e]).unwrap(), (-2, 1));

        assert_eq!(read_i32(&[0xff, 0]).unwrap(), (127, 2));
        assert_eq!(read_i32(&[0x80, 1]).unwrap(), (128, 2));
        assert_eq!(read_i32(&[0x81, 1]).unwrap(), (129, 2));

        assert_eq!(read_i32(&[0x81, 0x7f]).unwrap(), (-127, 2));
        assert_eq!(read_i32(&[0x80, 0x7f]).unwrap(), (-128, 2));
        assert_eq!(read_i32(&[0xff, 0x7e]).unwrap(), (-129, 2));
    }
}
