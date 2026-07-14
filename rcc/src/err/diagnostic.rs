use crate::source::SourceRange;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: ErrorKind,
    pub message: String,
    pub range: SourceRange,
}

impl Diagnostic {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>, range: SourceRange) -> Self {
        Self {
            kind,
            message: message.into(),
            range,
        }
    }

    pub fn render(&self, sources: &crate::source::SourceManager) -> String {
        let Ok(location) = sources.presumed_location(self.range.begin) else {
            return self.to_string();
        };
        let mut rendered = format!(
            "{}:{}:{}: {:?}: {}",
            location.filename, location.line, location.column, self.kind, self.message
        );
        if let Ok(line) = sources.source_line(self.range.begin) {
            rendered.push('\n');
            rendered.push_str(line);
            rendered.push('\n');
            rendered.push_str(&" ".repeat(location.column.saturating_sub(1) as usize));
            rendered.push('^');
        }
        if let Ok(frames) = sources.expansion_stack(self.range.begin) {
            for frame in frames {
                if let Ok(expansion) = sources.presumed_location(frame.expansion_range.begin) {
                    rendered.push_str(&format!(
                        "\nnote: expanded from macro '{}' at {}:{}:{}",
                        frame.macro_name, expansion.filename, expansion.line, expansion.column
                    ));
                }
            }
        }
        if let Ok(position) = sources.file_position(self.range.begin)
            && let Ok(frames) = sources.include_stack(position.file_id)
        {
            for frame in frames {
                if let Ok(include) = sources.presumed_location(frame.include_location) {
                    rendered.push_str(&format!(
                        "\nnote: included from {}:{}:{}",
                        include.filename, include.line, include.column
                    ));
                }
            }
        }
        rendered
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Diagnostic {}
