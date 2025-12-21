use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Restrict requires a pointer or reference, ('{invalid}' is invalid)")]
    RestrictError{ invalid: String },
}
