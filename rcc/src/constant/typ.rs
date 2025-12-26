use lazy_static::lazy_static;
use num_bigint::BigInt;

lazy_static! {
    pub static ref MAX_ARRAY_LEN: BigInt = BigInt::from(u64::MAX);
    pub static ref MAX_ULL: BigInt = BigInt::from(u64::MAX);
    pub static ref MAX_UINT: BigInt = BigInt::from(u32::MAX as u64);
}

/// 默认的对齐大小
pub static DEFAULT_ALIGN: u64 = 1;
pub static DEFAULT_SIZE: u64 = 1;
