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
    fn is_mantissa_0x2e147b() {
        assert_eq!(extract_mantissa!(0x3DAE147B), 0x2E147Bu32);
    }
}
