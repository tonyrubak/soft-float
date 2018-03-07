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
        (($single >> 23) & 0xFF) - 127
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
    0
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
    (((exponent + 127) << 23) as u32) |
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
}
