pub type ScopeResult<T> = Result<T, ScopeError>;

// todo 这个不适合放在这里
#[derive(Debug, Clone, Copy)]
pub enum ScopeSource {
    Tag,
    Label,
    Ident,
    Member,
}

#[derive(Debug)]
pub enum ScopeError {}

pub struct ScopeUndeclaredIdentError {}
