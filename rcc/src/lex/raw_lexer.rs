use super::pp_token::{PPToken, PPTokenKind, Punctuator};
use crate::source::{FileId, SourceError, SourceManager, SourceRange};
use std::sync::Arc;
use unicode_ident::{is_xid_continue, is_xid_start};

pub struct RawLexer<'a> {
    sources: &'a mut SourceManager,
    file_id: FileId,
    buffer: Arc<str>,
    physical_offsets: Vec<u32>,
    offset: usize,
    start_of_line: bool,
    leading_space: bool,
    in_block_comment: bool,
}

impl<'a> RawLexer<'a> {
    pub fn new(sources: &'a mut SourceManager, file_id: FileId) -> Result<Self, SourceError> {
        let physical = sources.buffer_arc(file_id)?;
        let (buffer, physical_offsets) = translate_phase_one_and_two(&physical);
        Ok(Self {
            sources,
            file_id,
            buffer,
            physical_offsets,
            offset: 0,
            start_of_line: true,
            leading_space: false,
            in_block_comment: false,
        })
    }

    pub fn next_token(&mut self) -> Result<PPToken, SourceError> {
        self.skip_horizontal_space_and_comments();
        let start = self.offset;
        let start_of_line = self.start_of_line;
        let leading_space = std::mem::take(&mut self.leading_space);
        let Some(character) = self.peek() else {
            return self.token(start, PPTokenKind::EndOfFile, leading_space, start_of_line);
        };
        let kind = if matches!(character, '\n' | '\r') {
            self.consume_newline();
            self.start_of_line = true;
            PPTokenKind::NewLine
        } else if let Some((prefix, quote)) = self.literal_prefix() {
            for _ in 0..prefix {
                self.bump();
            }
            self.lex_quoted(quote);
            self.start_of_line = false;
            if quote == '\'' {
                PPTokenKind::Character
            } else {
                PPTokenKind::String
            }
        } else if character == '_' || is_xid_start(character) {
            self.bump();
            while self.peek().is_some_and(|c| c == '_' || is_xid_continue(c)) {
                self.bump();
            }
            self.start_of_line = false;
            PPTokenKind::Identifier
        } else if character.is_ascii_digit()
            || character == '.' && self.peek_n(1).is_some_and(|c| c.is_ascii_digit())
        {
            self.lex_pp_number();
            self.start_of_line = false;
            PPTokenKind::Number
        } else if matches!(character, '\'' | '"') {
            self.lex_quoted(character);
            self.start_of_line = false;
            if character == '\'' {
                PPTokenKind::Character
            } else {
                PPTokenKind::String
            }
        } else {
            let punctuator = self.lex_punctuator();
            self.start_of_line = false;
            punctuator.map_or(PPTokenKind::Invalid, PPTokenKind::Punctuator)
        };
        self.token(start, kind, leading_space, start_of_line)
    }

    fn skip_horizontal_space_and_comments(&mut self) {
        loop {
            let before = self.offset;
            if self.in_block_comment {
                while self.offset < self.buffer.len() && !self.starts_with("*/") {
                    if matches!(self.peek(), Some('\n' | '\r')) {
                        return;
                    }
                    self.bump();
                }
                if self.starts_with("*/") {
                    self.offset += 2;
                    self.in_block_comment = false;
                }
                self.leading_space = true;
            }
            while self
                .peek()
                .is_some_and(|c| matches!(c, ' ' | '\t' | '\u{b}' | '\u{c}'))
            {
                self.bump();
                self.leading_space = true;
            }
            if self.starts_with("//") {
                self.offset += 2;
                while self.peek().is_some_and(|c| !matches!(c, '\n' | '\r')) {
                    self.bump();
                }
                self.leading_space = true;
            } else if self.starts_with("/*") {
                self.offset += 2;
                self.in_block_comment = true;
                self.leading_space = true;
            }
            if self.offset == before {
                break;
            }
        }
    }

    fn lex_pp_number(&mut self) {
        self.bump();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric()
                || matches!(c, '_' | '.')
                || matches!(c, '+' | '-')
                    && self
                        .previous()
                        .is_some_and(|p| matches!(p, 'e' | 'E' | 'p' | 'P'))
            {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn lex_quoted(&mut self, quote: char) {
        self.bump();
        while let Some(c) = self.peek() {
            self.bump();
            if c == '\\' {
                self.bump();
                continue;
            }
            if c == quote || matches!(c, '\n' | '\r') {
                break;
            }
        }
    }

    fn literal_prefix(&self) -> Option<(usize, char)> {
        [
            ("u8\"", 2, '"'),
            ("u\"", 1, '"'),
            ("U\"", 1, '"'),
            ("L\"", 1, '"'),
            ("u'", 1, '\''),
            ("U'", 1, '\''),
            ("L'", 1, '\''),
        ]
        .into_iter()
        .find_map(|(text, width, quote)| self.starts_with(text).then_some((width, quote)))
    }

    fn lex_punctuator(&mut self) -> Option<Punctuator> {
        use Punctuator::*;
        let punctuators = [
            ("%:%:", HashHash),
            (">>=", ShrEq),
            ("<<=", ShlEq),
            ("...", Ellipsis),
            ("##", HashHash),
            ("->", Arrow),
            ("++", Inc),
            ("--", Dec),
            ("&&", And),
            ("||", Or),
            ("<=", Le),
            (">=", Ge),
            ("==", Eq),
            ("!=", Ne),
            ("*=", StarEq),
            ("/=", SlashEq),
            ("%=", PercentEq),
            ("+=", PlusEq),
            ("-=", MinusEq),
            ("<<", Shl),
            (">>", Shr),
            ("&=", AmpEq),
            ("^=", CaretEq),
            ("|=", PipeEq),
            ("<:", LBracket),
            (":>", RBracket),
            ("<%", LBrace),
            ("%>", RBrace),
            ("%:", Hash),
        ];
        for (text, kind) in punctuators {
            if self.starts_with(text) {
                self.offset += text.len();
                return Some(kind);
            }
        }
        Some(match self.bump().unwrap() {
            '[' => LBracket,
            ']' => RBracket,
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '.' => Dot,
            '&' => Amp,
            '*' => Star,
            '+' => Plus,
            '-' => Minus,
            '~' => Tilde,
            '!' => Bang,
            '/' => Slash,
            '%' => Percent,
            '<' => Lt,
            '>' => Gt,
            '^' => Caret,
            '|' => Pipe,
            '?' => Question,
            ':' => Colon,
            ';' => Semi,
            '=' => Assign,
            ',' => Comma,
            '#' => Hash,
            _ => return None,
        })
    }

    fn token(
        &mut self,
        start: usize,
        kind: PPTokenKind,
        leading_space: bool,
        start_of_line: bool,
    ) -> Result<PPToken, SourceError> {
        let begin = self
            .sources
            .file_location(self.file_id, self.physical_offsets[start])?;
        let end = self
            .sources
            .file_location(self.file_id, self.physical_offsets[self.offset])?;
        Ok(PPToken {
            kind,
            spelling: self.buffer[start..self.offset].to_string(),
            range: SourceRange::new(begin, end),
            leading_space,
            start_of_line,
        })
    }

    fn peek(&self) -> Option<char> {
        self.buffer[self.offset..].chars().next()
    }
    fn peek_n(&self, n: usize) -> Option<char> {
        self.buffer[self.offset..].chars().nth(n)
    }
    fn previous(&self) -> Option<char> {
        self.buffer[..self.offset].chars().next_back()
    }
    fn starts_with(&self, text: &str) -> bool {
        self.buffer[self.offset..].starts_with(text)
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.offset += c.len_utf8();
        Some(c)
    }
    fn consume_newline(&mut self) {
        if self.starts_with("\r\n") {
            self.offset += 2
        } else {
            self.bump();
        }
    }
}

fn translate_phase_one_and_two(source: &str) -> (Arc<str>, Vec<u32>) {
    let bytes = source.as_bytes();
    let mut logical = String::with_capacity(source.len());
    let mut offsets = vec![0_u32];
    let mut physical = 0;
    while physical < bytes.len() {
        let (character, width) = trigraph(bytes, physical).unwrap_or_else(|| {
            let character = source[physical..].chars().next().unwrap();
            (character, character.len_utf8())
        });
        if character == '\\' {
            let newline = physical + width;
            let splice_width = if bytes.get(newline) == Some(&b'\n') {
                Some(1)
            } else if bytes.get(newline..newline + 2) == Some(b"\r\n") {
                Some(2)
            } else if bytes.get(newline) == Some(&b'\r') {
                Some(1)
            } else {
                None
            };
            if let Some(splice_width) = splice_width {
                physical = newline + splice_width;
                *offsets.last_mut().unwrap() = physical as u32;
                continue;
            }
        }
        logical.push(character);
        physical += width;
        for _ in 0..character.len_utf8() {
            offsets.push(physical as u32);
        }
    }
    (Arc::from(logical), offsets)
}

fn trigraph(bytes: &[u8], offset: usize) -> Option<(char, usize)> {
    if bytes.get(offset..offset + 2) != Some(b"??") {
        return None;
    }
    let replacement = match *bytes.get(offset + 2)? {
        b'=' => '#',
        b'/' => '\\',
        b'\'' => '^',
        b'(' => '[',
        b')' => ']',
        b'!' => '|',
        b'<' => '{',
        b'>' => '}',
        b'-' => '~',
        _ => return None,
    };
    Some((replacement, 3))
}
