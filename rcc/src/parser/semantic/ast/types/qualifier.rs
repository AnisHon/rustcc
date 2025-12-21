use crate::parser::semantic::decl_spec::TypeQuals;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Qualifier {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

impl Qualifier {
    pub fn new(type_quals: &TypeQuals) -> Self {
        Self {
            is_const: type_quals.is_const.is_some(),
            is_volatile: type_quals.is_volatile.is_some(),
            is_restrict: type_quals.is_restrict.is_some(),
        }
    }
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
