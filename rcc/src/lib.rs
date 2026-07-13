//! A self-contained C11 front end.
//!
//! The old prototype modules are intentionally no longer part of the crate's
//! compilation graph.  They mixed several incompatible AST generations.  The
//! public API below has one source of truth for tokens, types and semantic AST.

pub mod frontend;

pub use frontend::*;
