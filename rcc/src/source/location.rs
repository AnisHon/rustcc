use std::fmt::{Debug, Formatter};

/// Stable source buffer identifier. Zero is reserved for an invalid file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FileId(pub(crate) u32);

impl FileId {
    pub const INVALID: Self = Self(0);
    pub fn is_valid(self) -> bool {
        self != Self::INVALID
    }
    pub fn index(self) -> Option<usize> {
        self.is_valid().then_some((self.0 - 1) as usize)
    }
}

impl Debug for FileId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileId({})", self.0)
    }
}

/// Four-byte handle into a `SourceManager` location table.
/// Zero is always the invalid/unknown location.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SourceLocation(pub(crate) u32);

impl SourceLocation {
    pub const INVALID: Self = Self(0);
    pub fn is_valid(self) -> bool {
        self != Self::INVALID
    }
    pub(crate) fn index(self) -> Option<usize> {
        self.is_valid().then_some((self.0 - 1) as usize)
    }
}

impl Debug for SourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SourceLocation({})", self.0)
    }
}

/// Half-open source range `[begin, end)`. `end` points immediately after the
/// final byte/token represented by the range.
///
/// Both endpoints are handles, so a range stays small even when it refers to a macro expansion
/// carrying separate spelling and expansion locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SourceRange {
    pub begin: SourceLocation,
    pub end: SourceLocation,
}

impl SourceRange {
    pub fn new(begin: SourceLocation, end: SourceLocation) -> Self {
        Self { begin, end }
    }
    pub fn empty(at: SourceLocation) -> Self {
        Self { begin: at, end: at }
    }
    pub fn is_valid(self) -> bool {
        self.begin.is_valid() && self.end.is_valid()
    }
    pub fn join(self, other: Self) -> Self {
        Self::new(self.begin, other.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A resolved physical position inside one source buffer.
///
/// This is produced on demand from a compact `SourceLocation`; AST nodes should
/// keep the handle rather than caching this expanded representation.
pub struct FileLocation {
    pub file_id: FileId,
    pub byte_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// User-facing position after applying `#line` directives.
///
/// Diagnostics use this view for filenames and line numbers while retaining
/// the physical location for rendering the actual source line.
pub struct PresumedLocation {
    pub filename: String,
    pub line: u32,
    pub column: u32,
}
