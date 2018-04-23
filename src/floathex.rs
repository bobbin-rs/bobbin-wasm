use core::fmt::{self, Write};
#[allow(unused_imports)] use core::num::Float;

pub fn f32_parts(v: f32) -> (u8, u8, u32) {
    let v = v.to_bits();
    let s = (v >> 31) as u8;
    let e = (v >> 23) as u8;
    let f = v << 9;
    (s, e, f)
}

pub fn f32_hex<W: Write>(out: &mut W, v: f32) -> fmt::Result {
    let (s, e, f) = f32_parts(v);

    match e {
        0x00 => {
            if s == 1 {
                write!(out, "-")?;
            }
            write!(out, "0x0")?;
            if f > 0 {
                write!(out, ".{:x}", f)?;
            }
            write!(out, "p+0")?;
        },        
        0xff => {
            if f & 1 << 31 != 0 {
                if s != 0 {
                    write!(out, "-nan")?;
                } else {
                    write!(out, "nan")?;
                }
            } else if f == 0 {
                if s != 0 {
                    write!(out, "-inf")?;
                } else {
                    write!(out, "inf")?;
                }
            } else {
                if s != 0 {
                    write!(out, "-nan:0x{:x}", f >> 9)?;
                } else {
                    write!(out, "nan:0x{:x}", f >> 9)?;
                }
            }
        },
        _ => {
            if s == 1 {
                write!(out, "-")?;
            }
            write!(out, "0x1")?;
            if f > 0 {
                write!(out, ".")?;
                let mut f = f;
                while f != 0 {
                    let v = f >> 28;
                    write!(out, "{:x}", v)?;
                    f = f << 4;
                }
            }
            let e = e as u32 as i32;
            let e = e - 127;
            write!(out, "p{:+}", e )?;
        }
    }
    Ok(())
}

pub fn f64_parts(v: f64) -> (u8, u16, u64) {
    let v = v.to_bits();
    let s = (v >> 63) as u8;
    let e = (((v >> 52) & (1 << 11) - 1)) as u16;
    let f = v << 12;
    (s, e, f)
}

pub fn f64_hex<W: Write>(out: &mut W, v: f64) -> fmt::Result {
    let (s, e, f) = f64_parts(v);

    match e {
        0x000 => {
            if s == 1 {
                write!(out, "-")?;
            }
            write!(out, "0x0")?;
            if f > 0 {
                write!(out, ".{:x}", f)?;
            }
            write!(out, "p+0")?;
        },        
        0x07ff => {
            if f & 1 << 63 != 0 {
                if s != 0 {
                    write!(out, "-nan")?;
                } else {
                    write!(out, "nan")?;
                }
            } else if f == 0 {
                if s != 0 {
                    write!(out, "-inf")?;
                } else {
                    write!(out, "inf")?;
                }
            } else {
                if s != 0 {
                    write!(out, "-nan:0x{:x}", f >> 12)?;
                } else {
                    write!(out, "nan:0x{:x}", f >> 12)?;
                }
            }
        },
        _ => {
            if s == 1 {
                write!(out, "-")?;
            }
            write!(out, "0x1")?;
            if f > 0 {
                write!(out, ".")?;
                let mut f = f;
                while f != 0 {
                    let v = f >> 60;
                    write!(out, "{:x}", v)?;
                    f = f << 4;
                }
            }
            let e = e as u32 as i32;
            let e = e - 1023;
            write!(out, "p{:+}", e )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_f32() {
        assert_eq!(f32_parts(1.0), (0, 127, 0));
        assert_eq!(f32_parts(0.5), (0, 126, 0));
        assert_eq!(f32_parts(2.0), (0, 128, 0));
        assert_eq!(f32_parts(3.0), (0, 128, 0x8000_0000));
        assert_eq!(f32_parts(4.0), (0, 129, 0));        
        assert_eq!(f32_hex(0.0),"0x0p+0");
        assert_eq!(f32_hex(1.0),"0x1p+0");
        assert_eq!(f32_hex(0.5),"0x1p-1");
        assert_eq!(f32_hex(2.0),"0x1p+1");
        assert_eq!(f32_hex(3.0),"0x1.8p+1");
        assert_eq!(f32_hex(4.0),"0x1p+2");
        assert_eq!(f32_hex(2.0f32.powi(8)),"0x1p+8");
        assert_eq!(f32_hex(0.857421875f32), "0x1.b7p-1");
        assert_eq!(f32_hex(f32::from_bits(0x65a9_6816)), "0x1.52d02cp+76");
        assert_eq!(f32_hex(f32::from_bits(0x374f_2040)), "0x1.9e408p-17");
        assert_eq!(f32_hex(f32::from_bits(0x7fc0_0000)), "nan");
        assert_eq!(f32_hex(f32::from_bits(0x7f80_0abc)), "nan:0xabc");
        assert_eq!(f32_hex(f32::from_bits(0xff80_0abc)), "-nan:0xabc");
        assert_eq!(f32_hex(f32::from_bits(0x7f80_0000)), "inf");
    }
    #[test]
    fn test_f64() {
        assert_eq!(f64_hex(0.0),"0x0p+0");
        assert_eq!(f64_hex(1.0),"0x1p+0");
        assert_eq!(f64_hex(0.5),"0x1p-1");
        assert_eq!(f64_hex(2.0),"0x1p+1");
        assert_eq!(f64_hex(3.0),"0x1.8p+1");
        assert_eq!(f64_hex(4.0),"0x1p+2");
        assert_eq!(f64_hex(2.0f64.powi(8)),"0x1p+8");
        assert_eq!(f64_hex(0.857421875f64), "0x1.b7p-1");
        // assert_eq!(f64_parts(f64::from_bits(0xbfef_9add_3c0e_56b8)), (0,0,0));        
        assert_eq!(f64_hex(f64::from_bits(0xbfef_9add_3c0e_56b8)), "-0x1.f9add3c0e56b8p-1");
        assert_eq!(f64_hex(f64::from_bits(0x4019_21fb_5444_2d18)), "0x1.921fb54442d18p+2");
        // assert_eq!(f64_parts(f64::from_bits(0x7ff8_0000_0000_0000)), (0,0,0));        
        assert_eq!(f64_hex(f64::from_bits(0x7ff8_0000_0000_0000)), "nan");
        // assert_eq!(f64_parts(f64::from_bits(0xfff8_0000_0000_0000)), (0,0,0));        
        assert_eq!(f64_hex(f64::from_bits(0xfff8_0000_0000_0000)), "-nan");
        assert_eq!(f64_hex(f64::from_bits(0x7ff0_0000_0000_0abc)), "nan:0xabc");
        assert_eq!(f64_hex(f64::from_bits(0xfff0_0000_0000_0abc)), "-nan:0xabc");
        assert_eq!(f64_hex(f64::from_bits(0x7ff0_0000_0000_0000)), "inf");
        // assert_eq!(f64_parts(f64::from_bits(0xbfe0_0000_0000_0000)), (0,0,0));        
        assert_eq!(f64_hex(f64::from_bits(0xbfe0_0000_0000_0000)), "-0x1p-1");
    }    
}
