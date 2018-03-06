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
        assert_eq!(from_f32!(0.085f32), 0b0111101101011100001010001111011);
    }
    
    #[test]
    fn create_float() {
        assert_eq!(super::from_f32(&0.085f32), 0b0111101101011100001010001111011);
    }

    #[test]
    fn is_positive_number_positive() {
        assert_eq!(extract_sign!(0b0111101101011100001010001111011u32), 0);
    }

    fn is_negative_number_negative() {
        assert_eq!(extract_sign!(0b1111101101011100001010001111011u32), 1);
    }
}
