#![macro_escape]
pub type Single = u32;

#[macro_export]
macro_rules! from_f32 {
    ($float:expr) => ({
        unsafe { * ((&$float as *const f32) as *const u32) }
    });
}

macro_rules! extract_sign {
    ($single: expr) => ({
        if ($single & 0x80000000) != 0 { 1 } else { 0 }
    });
}

macro_rules! extract_exponent {
    ($single: expr) => ({
        ((($single >> 23) & 0xFF) as i32 - 127)
    });
}

macro_rules! extract_mantissa {
    ($single: expr) => ({
        if $single & 0x7FFFFFFF == 0 { 0 } else { $single & 0x7FFFFF | 0x800000 }
    });
}

pub fn from_f32(float: &f32) -> Single {
    unsafe { * ((float as *const f32) as *const u32) }
}

pub fn fpadd(x: Single, y: Single) -> Single {
    let (mut l_sign, mut l_exp, mut l_mant) = (extract_sign!(x), extract_exponent!(x), extract_mantissa!(x));
    let (mut r_sign, mut r_exp, mut r_mant) = (extract_sign!(y), extract_exponent!(y), extract_mantissa!(y));

    // We might should check to see if our floating point values are bad here, but we won't... yet

    // If exponents are not equal, denormalize value with the smaller exponent

    let mut d_sign: u32;
    let mut d_exp: i32;
    let mut d_mant: u32;

    if (r_exp > l_exp) {
        l_mant = shift_and_round(l_mant, (r_exp - l_exp) as usize);
        d_exp = r_exp;
    } else if (l_exp > r_exp) {
        r_mant = shift_and_round(r_mant, (l_exp - r_exp) as usize);
        d_exp = l_exp;
    } else { d_exp = r_exp; }
    
    // If signs are the same we are adding, otherwise we are subtracting

    if l_sign ^ r_sign == 1 {
        // Signs are opposite, so let's subtract the larger one from the smaller
        if (l_mant > r_mant) {
            d_mant = l_mant - r_mant;
            d_sign = l_sign;
        } else {
            d_mant = r_mant - l_mant;
            d_sign = r_sign;
        }
    } else {
        // Signs are the same, so we'll add the values
        d_mant = r_mant + l_mant;
        d_sign = l_sign;
    }

    // Check for overflow or normalize the result
    if d_mant >= 0x1000000 {
        // We overflowed, so squeeze the result back into 24 bits and round
        d_mant = shift_and_round(d_mant, 1);
        d_exp += 1;
    } else {
        // Normalize the result (HO bit should be 1)
        if (d_mant != 0) {
            while d_mant < 0x800000 && d_exp > -127 {
                d_mant = d_mant << 1;
                d_exp -= 1;
            }
        } else {
            d_sign = 0; // we'll always return +0 instead of -0
            d_exp = 0;
        }
    }

    pack_single(d_sign, d_exp, d_mant)
}

pub fn fpsub(x: Single, y: Single) -> Single {
    fpadd(x, y ^ 0x80000000)
}

// Shift the given mantissa to the right n bits; rounds according to IEEE 754 rules
//
// * Truncate if last bit shifted was 0
// * Increase mantissa by 1 if last bit shifted was 1 and at least one other 1 shifted out
// * Increase mantissa by 1 if last bit shifted was 1, all other bits were 0, and mantissa LO bit is 1

// Masks for shift_and_round to isolate shifted bits

const MASKS: [u32; 24] = [0x0, 0x1, 0x3, 0x7,
                          0xF, 0x1F, 0x3F, 0x7F,
                          0xFF, 0x1FF, 0x3FF, 0x7FF,
                          0xFFF, 0x1FFF, 0x3FFF, 0x7FFF,
                          0xFFFF, 0x1FFFF, 0x3FFFF, 0x7FFF,
                          0xFFFFF, 0x1FFFFF, 0x3FFFFF, 0x7FFFFF,];

const HO_MASKS: [u32; 24] = [0x0,
                             0x1, 0x2, 0x4, 0x8,
                             0x10, 0x20, 0x40, 0x80,
                             0x100, 0x200, 0x400, 0x800,
                             0x1000, 0x2000, 0x4000, 0x8000,
                             0x10000, 0x20000, 0x40000, 0x80000,
                             0x100000, 0x200000, 0x400000];

fn shift_and_round(mantissa: u32, n: usize) -> u32 {
    assert!(n <= 23);

    let shifted = mantissa & MASKS[n];

    let val = mantissa >> n;

    // Round as needed
    if shifted > HO_MASKS[n] {
        // The last bit shifted was a 1, and we shifted out at least 1 other 1, so round up
        val + 1
    } else if shifted == HO_MASKS[n] {
        // The last bit shifted was 1, but we didn't shift out any other 1s,
        // so round up if LO bit is 1
        val + (val & 1)
    } else {
        // The last bit shifted out was 0, so leave it alone
        val
    }
}

fn pack_single(sign: u32, exponent: i32, mantissa: u32) -> u32 {
    (sign << 31) |
    ((exponent + 127) << 23) as u32 |
    (mantissa & 0x7FFFFF)
}

#[cfg(test)]
mod tests {
    use super::Single;

    #[test]
    fn create_float_macro() {
        assert_eq!(from_f32!(0.085f32), 0x3DAE147B);
    }
    
    #[test]
    fn create_float() {
        assert_eq!(super::from_f32(&0.085f32), 0x3DAE147B);
    }

    #[test]
    fn is_positive_number_positive() {
        assert_eq!(extract_sign!(0x3DAE147B), 0);
    }

    #[test]
    fn is_negative_number_negative() {
        assert_eq!(extract_sign!(0xBDAE147B), 1);
    }

    #[test]
    fn is_exponent_neg_4() {
        assert_eq!(extract_exponent!(0x3DAE147B), -4);
    }

    #[test]
    fn is_mantissa_0xae147b() {
        assert_eq!(extract_mantissa!(0x3DAE147B), 0xAE147Bu32);
    }

    #[test]
    fn is_mantissa_rounded_up_case_1() {
        assert_eq!(super::shift_and_round(0x3DAE147B, 4), 0x3DAE148);
    }

    #[test]
    fn is_mantissa_rounded_up_case_2() {
        assert_eq!(super::shift_and_round(0x3DAE1478, 4), 0x3DAE148);
    }

    #[test]
    fn is_mantissa_truncated() {
        assert_eq!(super::shift_and_round(0x3DAE1473, 4), 0x3DAE147);
    }

    #[test]
    fn is_packed_single_0x3DAE147B() {
        assert_eq!(super::pack_single(0, -4, 0xAE147B), 0x3DAE147B);
    }
    
    #[test]
    fn is_result_of_addition_0x3F8AE148() {
        assert_eq!(super::fpadd(from_f32!(1f32),from_f32!(0.085f32)), 0x3F8AE148);
    }

    #[test]
    fn is_result_of_addition_0xBFAE148() {
        assert_eq!(super::fpadd(from_f32!(-1f32),from_f32!(-0.085f32)), 0xBF8AE148);
    }

    #[test]
    fn is_result_of_addition_0xBF6A3D71() {
        assert_eq!(super::fpadd(from_f32!(-1f32),from_f32!(0.085f32)), 0xBF6A3D71);
    }

    #[test]
    fn is_result_of_addition_0x3F6A3D71() {
        assert_eq!(super::fpadd(from_f32!(1f32),from_f32!(-0.085f32)), 0x3F6A3D71);
    }
}
