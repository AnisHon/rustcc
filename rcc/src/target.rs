//! Properties of the C implementation's compilation target.

/// Integer and pointer layout used by semantic analysis and constant evaluation.
///
/// Width and alignment fields are in bits. Keeping target facts here prevents
/// Parser and AST construction from hard-coding the host Rust platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetInfo {
    pub char_is_signed: bool,
    pub char_width: u16,
    pub short_width: u16,
    pub int_width: u16,
    pub long_width: u16,
    pub long_long_width: u16,
    pub pointer_width: u16,
    pub pointer_align: u16,
    pub wchar_width: u16,
}

impl TargetInfo {
    /// The common x86-64 Unix C ABI (LP64).
    pub const fn x86_64_lp64() -> Self {
        Self {
            char_is_signed: true,
            char_width: 8,
            short_width: 16,
            int_width: 32,
            long_width: 64,
            long_long_width: 64,
            pointer_width: 64,
            pointer_align: 64,
            wchar_width: 32,
        }
    }

    /// The common 64-bit Windows C ABI (LLP64).
    pub const fn x86_64_llp64() -> Self {
        Self {
            long_width: 32,
            ..Self::x86_64_lp64()
        }
    }
}

impl Default for TargetInfo {
    fn default() -> Self {
        Self::x86_64_lp64()
    }
}
