//! Source buffers and compact source-location handles.

mod location;
mod source_manager;

pub use location::{FileId, FileLocation, PresumedLocation, SourceLocation, SourceRange};
pub use source_manager::{ExpansionFrame, IncludeFrame, SourceError, SourceManager};
