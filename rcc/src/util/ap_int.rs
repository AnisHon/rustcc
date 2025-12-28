use ibig::{IBig, ibig};

use crate::constant::typ::INT_BITWIDTH;

/// 目前先用大数类型表示
#[derive(Debug, Clone)]
pub struct APInt {
    is_signed: bool,
    bit_width: usize,
    value: IBig,
}

impl APInt {
    /// 创建 ApInt，会自动将输入值截断到 bit_width 位
    pub fn new(is_signed: bool, bit_width: usize, value: impl Into<IBig>) -> Self {
        let mut res = Self {
            is_signed,
            bit_width,
            value: value.into(),
        };
        res.truncate();
        res
    }

    pub fn from_bool(b: bool) -> Self {
        Self::new(true, INT_BITWIDTH, value)
    }

    /// 截断到 bit_width
    fn truncate(&mut self) {
        if self.bit_width == 0 {
            return;
        }
        let mask = (IBig::from(1) << self.bit_width) - 1;
        self.value &= mask;
        if self.is_signed {
            // 检查符号位，如果高位是 1，做符号扩展
            let sign_bit = IBig::from(1) << (self.bit_width - 1);
            if &self.value & &sign_bit != IBig::from(0) {
                self.value -= IBig::from(1) << self.bit_width;
            }
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.is_signed, other.is_signed, "signedness mismatch");
        assert_eq!(self.bit_width, other.bit_width, "bit_width mismatch");

        let mut res = Self {
            is_signed: self.is_signed,
            bit_width: self.bit_width,
            value: &self.value + &other.value,
        };
        res.truncate();
        res
    }

    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.is_signed, other.is_signed, "signedness mismatch");
        assert_eq!(self.bit_width, other.bit_width, "bit_width mismatch");

        let mut res = Self {
            is_signed: self.is_signed,
            bit_width: self.bit_width,
            value: &self.value - &other.value,
        };
        res.truncate();
        res
    }

    // 按位与
    pub fn bitand(&self, other: &Self) -> Self {
        assert_eq!(self.bit_width, other.bit_width);
        Self::new(self.is_signed, self.bit_width, &self.value & &other.value)
    }

    // 按位或
    pub fn bitor(&self, other: &Self) -> Self {
        assert_eq!(self.bit_width, other.bit_width);
        Self::new(self.is_signed, self.bit_width, &self.value | &other.value)
    }

    // 按位异或
    pub fn bitxor(&self, other: &Self) -> Self {
        assert_eq!(self.bit_width, other.bit_width);
        Self::new(self.is_signed, self.bit_width, &self.value ^ &other.value)
    }

    // 左移
    pub fn shl(&self, shift: usize) -> Self {
        let mut res = Self::new(self.is_signed, self.bit_width, &self.value << shift);
        res.truncate();
        res
    }

    // 右移
    pub fn shr(&self, shift: usize) -> Self {
        let mut res = Self::new(self.is_signed, self.bit_width, &self.value >> shift);
        res.truncate();
        res
    }

    pub fn neg(&self) -> Self {
        Self::new(self.is_signed, self.bit_width, -&self.value)
    }

    pub fn as_bool(&self) -> bool {
        self.value != ibig!(0)
    }

    pub fn bitnot(&self) -> Self {
        Self::new(self.is_signed, self.bit_width, !&self.value)
    }

    pub fn as_usize(&self) -> usize {
        todo!("这里截断掉")
    }
}
