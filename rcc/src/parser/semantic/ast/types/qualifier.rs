#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Qualifier {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

impl Default for Qualifier {
    fn default() -> Self {
        Self {
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }
}