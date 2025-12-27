#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatTy {
    F32,
    F64,
    // long double 简化为 128 位, 使用 u128 或 rug::Float 模拟
    F128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum APFloat {
    F32(f32),
    F64(f64),
    F128(rug::Float),
}

impl APFloat {
    pub fn new_f32(val: f32) -> Self { APFloat::F32(val) }
    pub fn new_f64(val: f64) -> Self { APFloat::F64(val) }
    pub fn new_f128(val: rug::Float) -> Self { APFloat::F128(val) }

    /// 获取类型
    pub fn ty(&self) -> FloatTy {
        match self {
            APFloat::F32(_) => FloatTy::F32,
            APFloat::F64(_) => FloatTy::F64,
            APFloat::F128(_) => FloatTy::F128,
        }
    }

    /// 简单加法
    pub fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (APFloat::F32(a), APFloat::F32(b)) => APFloat::F32(a + b),
            (APFloat::F64(a), APFloat::F64(b)) => APFloat::F64(a + b),
            (APFloat::F128(a), APFloat::F128(b)) => APFloat::F128(a + b),
            _ => panic!("float type mismatch"), // 可以加类型转换规则
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        match (self, other) {
            (APFloat::F32(a), APFloat::F32(b)) => APFloat::F32(a - b),
            (APFloat::F64(a), APFloat::F64(b)) => APFloat::F64(a - b),
            (APFloat::F128(a), APFloat::F128(b)) => APFloat::F128(a - b),
            _ => panic!("float type mismatch"),
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        match (self, other) {
            (APFloat::F32(a), APFloat::F32(b)) => APFloat::F32(a * b),
            (APFloat::F64(a), APFloat::F64(b)) => APFloat::F64(a * b),
            (APFloat::F128(a), APFloat::F128(b)) => APFloat::F128(a * b),
            _ => panic!("float type mismatch"),
        }
    }

    pub fn div(&self, other: &Self) -> Self {
        match (self, other) {
            (APFloat::F32(a), APFloat::F32(b)) => APFloat::F32(a / b),
            (APFloat::F64(a), APFloat::F64(b)) => APFloat::F64(a / b),
            (APFloat::F128(a), APFloat::F128(b)) => APFloat::F128(a / b),
            _ => panic!("float type mismatch"),
        }
    }
}

