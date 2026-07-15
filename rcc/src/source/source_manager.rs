use super::{FileId, FileLocation, PresumedLocation, SourceLocation, SourceRange};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct SourceFile {
    name: String,
    buffer: Arc<str>,
    line_starts: Vec<u32>,
    include_location: SourceLocation,
    line_directives: Vec<LineDirective>,
}

#[derive(Debug, Clone)]
struct LineDirective {
    offset: u32,
    physical_line: u32,
    presumed_line: u32,
    presumed_filename: Option<String>,
}

#[derive(Debug, Clone)]
enum LocationEntry {
    /// A byte boundary in an owned source buffer.
    File(FileLocation),
    /// A logical location whose text was spelled elsewhere and expanded at `expansion`.
    Expansion {
        spelling: SourceLocation,
        expansion: SourceRange,
        macro_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeFrame {
    pub included_file: FileId,
    pub include_location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionFrame {
    pub macro_name: String,
    pub spelling_location: SourceLocation,
    pub expansion_range: SourceRange,
}

#[derive(Debug)]
pub enum SourceError {
    Io(std::io::Error),
    BufferTooLarge,
    InvalidFile,
    InvalidOffset { file_id: FileId, offset: u32 },
    InvalidLocation,
}

impl Display for SourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::BufferTooLarge => write!(f, "source buffer exceeds 4 GiB"),
            Self::InvalidFile => write!(f, "invalid source file id"),
            Self::InvalidOffset { file_id, offset } => {
                write!(f, "offset {offset} is outside {file_id:?}")
            }
            Self::InvalidLocation => write!(f, "invalid source location"),
        }
    }
}

impl std::error::Error for SourceError {}
impl From<std::io::Error> for SourceError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Owns every source buffer and every source-location record for one compile.
/// It deliberately has no dependency on lexing, parsing, semantic analysis or IR.
#[derive(Debug, Default)]
pub struct SourceManager {
    // Files and locations are append-only for the duration of a compilation. Their vector indices
    // are therefore stable and can safely be compressed into FileId/SourceLocation handles.
    files: Vec<SourceFile>,
    locations: Vec<LocationEntry>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(
        &mut self,
        path: impl AsRef<Path>,
        include_location: SourceLocation,
    ) -> Result<FileId, SourceError> {
        let path = path.as_ref();
        let buffer = std::fs::read_to_string(path)?;
        self.add_buffer(path.to_string_lossy(), buffer, include_location)
    }

    pub fn add_memory_buffer(
        &mut self,
        name: impl Into<String>,
        buffer: impl Into<String>,
    ) -> Result<FileId, SourceError> {
        self.add_buffer(name.into(), buffer.into(), SourceLocation::INVALID)
    }

    pub fn add_included_buffer(
        &mut self,
        name: impl Into<String>,
        buffer: impl Into<String>,
        include_location: SourceLocation,
    ) -> Result<FileId, SourceError> {
        self.add_buffer(name.into(), buffer.into(), include_location)
    }

    fn add_buffer(
        &mut self,
        name: impl Into<String>,
        buffer: String,
        include_location: SourceLocation,
    ) -> Result<FileId, SourceError> {
        if buffer.len() > u32::MAX as usize {
            return Err(SourceError::BufferTooLarge);
        }
        let mut line_starts = vec![0];
        for (index, byte) in buffer.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push((index + 1) as u32);
            }
        }
        let id = FileId(self.files.len() as u32 + 1);
        self.files.push(SourceFile {
            name: name.into(),
            buffer: Arc::from(buffer),
            line_starts,
            include_location,
            line_directives: vec![],
        });
        Ok(id)
    }

    pub fn buffer(&self, file_id: FileId) -> Result<&str, SourceError> {
        Ok(&self.file(file_id)?.buffer)
    }

    pub fn buffer_arc(&self, file_id: FileId) -> Result<Arc<str>, SourceError> {
        Ok(Arc::clone(&self.file(file_id)?.buffer))
    }

    pub fn filename(&self, file_id: FileId) -> Result<&str, SourceError> {
        Ok(&self.file(file_id)?.name)
    }

    pub fn file_location(
        &mut self,
        file_id: FileId,
        byte_offset: u32,
    ) -> Result<SourceLocation, SourceError> {
        let file = self.file(file_id)?;
        if byte_offset as usize > file.buffer.len() {
            return Err(SourceError::InvalidOffset {
                file_id,
                offset: byte_offset,
            });
        }
        self.push_location(LocationEntry::File(FileLocation {
            file_id,
            byte_offset,
        }))
    }

    pub fn expansion_location(
        &mut self,
        spelling: SourceLocation,
        expansion: SourceRange,
        macro_name: impl Into<String>,
    ) -> Result<SourceLocation, SourceError> {
        self.location(spelling)?;
        self.location(expansion.begin)?;
        self.location(expansion.end)?;
        self.push_location(LocationEntry::Expansion {
            spelling,
            expansion,
            macro_name: macro_name.into(),
        })
    }

    pub fn spelling_location(
        &self,
        location: SourceLocation,
    ) -> Result<SourceLocation, SourceError> {
        // Nested macro replacements may form several expansion records. Spelling resolution walks
        // outward until it reaches a physical file location.
        let mut current = location;
        while let LocationEntry::Expansion { spelling, .. } = self.location(current)? {
            current = *spelling;
        }
        Ok(current)
    }

    pub fn expansion_location_of(
        &self,
        location: SourceLocation,
    ) -> Result<SourceLocation, SourceError> {
        // Diagnostics normally point at the outermost user-visible invocation rather than a token
        // inside a macro definition or an intermediate replacement list.
        let mut current = location;
        while let LocationEntry::Expansion { expansion, .. } = self.location(current)? {
            current = expansion.begin;
        }
        Ok(current)
    }

    pub fn file_position(&self, location: SourceLocation) -> Result<FileLocation, SourceError> {
        match self.location(location)? {
            LocationEntry::File(position) => Ok(*position),
            LocationEntry::Expansion { .. } => {
                self.file_position(self.expansion_location_of(location)?)
            }
        }
    }

    pub fn presumed_location(
        &self,
        location: SourceLocation,
    ) -> Result<PresumedLocation, SourceError> {
        // Physical line lookup is binary search over precomputed byte offsets. #line directives
        // affect only the displayed filename/line; columns always come from the physical buffer.
        let position = self.file_position(location)?;
        let file = self.file(position.file_id)?;
        let line_index = file
            .line_starts
            .partition_point(|start| *start <= position.byte_offset)
            .saturating_sub(1);
        let physical_line = line_index as u32 + 1;
        let column = file.buffer
            [file.line_starts[line_index] as usize..position.byte_offset as usize]
            .chars()
            .count() as u32
            + 1;
        let directive = file
            .line_directives
            .iter()
            .rev()
            .find(|directive| directive.offset <= position.byte_offset);
        let (filename, line) = match directive {
            Some(directive) => (
                directive
                    .presumed_filename
                    .clone()
                    .unwrap_or_else(|| file.name.clone()),
                directive.presumed_line + physical_line - directive.physical_line,
            ),
            None => (file.name.clone(), physical_line),
        };
        Ok(PresumedLocation {
            filename,
            line,
            column,
        })
    }

    pub fn source_line(&self, location: SourceLocation) -> Result<&str, SourceError> {
        let position = self.file_position(location)?;
        let file = self.file(position.file_id)?;
        let line_index = file
            .line_starts
            .partition_point(|start| *start <= position.byte_offset)
            .saturating_sub(1);
        let begin = file.line_starts[line_index] as usize;
        let end = file
            .line_starts
            .get(line_index + 1)
            .copied()
            .map_or(file.buffer.len(), |offset| offset as usize);
        Ok(file.buffer[begin..end].trim_end_matches(['\r', '\n']))
    }

    pub fn add_line_directive(
        &mut self,
        file_id: FileId,
        byte_offset: u32,
        presumed_line: u32,
        presumed_filename: Option<String>,
    ) -> Result<(), SourceError> {
        let physical_line = {
            let file = self.file(file_id)?;
            if byte_offset as usize > file.buffer.len() {
                return Err(SourceError::InvalidOffset {
                    file_id,
                    offset: byte_offset,
                });
            }
            file.line_starts
                .partition_point(|start| *start <= byte_offset) as u32
        };
        self.file_mut(file_id)?.line_directives.push(LineDirective {
            offset: byte_offset,
            physical_line,
            presumed_line,
            presumed_filename,
        });
        Ok(())
    }

    pub fn include_stack(&self, file_id: FileId) -> Result<Vec<IncludeFrame>, SourceError> {
        let mut frames = Vec::new();
        let mut current = file_id;
        loop {
            let include_location = self.file(current)?.include_location;
            if !include_location.is_valid() {
                break;
            }
            frames.push(IncludeFrame {
                included_file: current,
                include_location,
            });
            current = self.file_position(include_location)?.file_id;
        }
        Ok(frames)
    }

    pub fn expansion_stack(
        &self,
        location: SourceLocation,
    ) -> Result<Vec<ExpansionFrame>, SourceError> {
        let mut frames = Vec::new();
        let mut current = location;
        while let LocationEntry::Expansion {
            spelling,
            expansion,
            macro_name,
        } = self.location(current)?
        {
            frames.push(ExpansionFrame {
                macro_name: macro_name.clone(),
                spelling_location: *spelling,
                expansion_range: *expansion,
            });
            current = expansion.begin;
        }
        Ok(frames)
    }

    fn file(&self, id: FileId) -> Result<&SourceFile, SourceError> {
        id.index()
            .and_then(|index| self.files.get(index))
            .ok_or(SourceError::InvalidFile)
    }
    fn file_mut(&mut self, id: FileId) -> Result<&mut SourceFile, SourceError> {
        id.index()
            .and_then(|index| self.files.get_mut(index))
            .ok_or(SourceError::InvalidFile)
    }
    fn location(&self, location: SourceLocation) -> Result<&LocationEntry, SourceError> {
        location
            .index()
            .and_then(|index| self.locations.get(index))
            .ok_or(SourceError::InvalidLocation)
    }
    fn push_location(&mut self, entry: LocationEntry) -> Result<SourceLocation, SourceError> {
        if self.locations.len() >= u32::MAX as usize {
            return Err(SourceError::BufferTooLarge);
        }
        self.locations.push(entry);
        Ok(SourceLocation(self.locations.len() as u32))
    }
}
