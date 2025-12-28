pub const MAX_ARRAY_LEN:usize = u64::MAX as usize;
pub const MAX_ULL:usize = u64::MAX as usize;
pub const MAX_UINT:usize = u32::MAX as usize;

/// 默认的对齐大小
pub const DEFAULT_ALIGN:usize = 1;
pub const DEFAULT_SIZE:usize = 1;

/// 默认类型位宽
pub const CHAR_BITWIDTH:usize = 8;
pub const SHORT_BITWIDTH:usize = 16;
pub const INT_BITWIDTH:usize = 32;
pub const LONG_BITWIDTH:usize = 64;
pub const LONGLONG_BITWIDTH:usize = 64;

pub const FLOAT_BITWIDTH:usize = 32;
pub const DOUBLE_BITWIDTH:usize = 64;
pub const LONGDOUBLE_BITWIDTH:usize = 80;


/// 默认类型byte
pub const CHAR_BYTES:usize = CHAR_BITWIDTH / 8;
pub const SHORT_BYTES:usize = SHORT_BITWIDTH / 8;
pub const INT_BYTES:usize = INT_BITWIDTH / 8;
pub const LONG_BYTES:usize = LONG_BITWIDTH / 8;
pub const LONGLONG_BYTES:usize = LONGLONG_BITWIDTH / 8;

