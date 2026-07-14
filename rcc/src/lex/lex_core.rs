use crate::err::{Diagnostic, ErrorKind};
use crate::lex::token::{Keyword, Literal, StringEncoding, Token, TokenKind};
use crate::lex::{PPToken, PPTokenKind};
use crate::source::SourceRange;
use unicode_ident::{is_xid_continue, is_xid_start};

#[derive(Clone, Copy)]
struct Position {
    offset: usize,
}

fn lex(source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    Lexer::new(source).run()
}

/// Classify an already-preprocessed token stream into C language tokens.
/// Keyword recognition intentionally happens here, after macro replacement.
pub fn classify_preprocessed(
    preprocessing_tokens: Vec<PPToken>,
) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let mut output = Vec::with_capacity(preprocessing_tokens.len());
    let mut diagnostics = Vec::new();
    for preprocessing_token in preprocessing_tokens {
        let range = preprocessing_token.range;
        if preprocessing_token.kind == PPTokenKind::EndOfFile {
            output.push(Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                range,
            });
            continue;
        }
        match lex(&preprocessing_token.spelling) {
            Ok(mut classified) if classified.len() == 2 => {
                let mut token = classified.remove(0);
                token.range = range;
                output.push(token);
            }
            Ok(_) => diagnostics.push(Diagnostic::new(
                ErrorKind::Lexical,
                format!(
                    "preprocessing token '{}' is not one C token",
                    preprocessing_token.spelling
                ),
                range,
            )),
            Err(mut errors) => {
                for error in &mut errors {
                    error.range = range;
                }
                diagnostics.extend(errors);
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(output)
    } else {
        Err(diagnostics)
    }
}

struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    line: usize,
    column: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
            column: 1,
            diagnostics: vec![],
        }
    }
    fn position(&self) -> Position {
        Position { offset: self.pos }
    }
    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }
    fn starts(&self, s: &str) -> bool {
        self.src[self.pos..].starts_with(s)
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1
        } else {
            self.column += 1
        };
        Some(c)
    }
    fn token(&self, start: Position, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: self.src[start.offset..self.pos].to_string(),
            range: SourceRange::default(),
        }
    }
    fn error(&mut self, _start: Position, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic::new(
            ErrorKind::Lexical,
            msg,
            SourceRange::default(),
        ));
    }

    fn run(mut self) -> Result<Vec<Token>, Vec<Diagnostic>> {
        let mut out = vec![];
        while self.peek().is_some() {
            self.skip_trivia();
            let Some(c) = self.peek() else { break };
            let start = self.position();
            let prefixed = [
                ("u8\"", 2, StringEncoding::Utf8, '"'),
                ("u\"", 1, StringEncoding::Utf16, '"'),
                ("U\"", 1, StringEncoding::Utf32, '"'),
                ("L\"", 1, StringEncoding::Wide, '"'),
                ("u'", 1, StringEncoding::Utf16, '\''),
                ("U'", 1, StringEncoding::Utf32, '\''),
                ("L'", 1, StringEncoding::Wide, '\''),
            ]
            .into_iter()
            .find(|(prefix, _, _, _)| self.starts(prefix));
            let token = if let Some((_, prefix_len, encoding, quote)) = prefixed {
                self.quoted(start, quote, prefix_len, encoding)
            } else if c == '_' || is_xid_start(c) {
                self.identifier(start)
            } else if c.is_ascii_digit()
                || (c == '.' && self.src[self.pos + 1..].starts_with(|x: char| x.is_ascii_digit()))
            {
                self.number(start)
            } else if c == '\'' || c == '"' {
                self.quoted(start, c, 0, StringEncoding::Narrow)
            } else {
                self.punctuator(start)
            };
            if let Some(t) = token {
                out.push(t);
            }
        }
        out.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            range: SourceRange::default(),
        });
        if self.diagnostics.is_empty() {
            Ok(out)
        } else {
            Err(self.diagnostics)
        }
    }
    fn skip_trivia(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.bump();
            }
            if self.starts("//") {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.bump();
                }
                continue;
            }
            if self.starts("/*") {
                let s = self.position();
                self.bump();
                self.bump();
                while !self.starts("*/") && self.peek().is_some() {
                    self.bump();
                }
                if self.peek().is_none() {
                    self.error(s, "unterminated block comment");
                    return;
                }
                self.bump();
                self.bump();
                continue;
            }
            // Preprocessing is a separate translation phase. Ignore directives so
            // preprocessed and simple source files are both accepted by the API.
            if self.column == 1 && self.peek() == Some('#') {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.bump();
                }
                continue;
            }
            break;
        }
    }
    fn identifier(&mut self, start: Position) -> Option<Token> {
        while self.peek().is_some_and(|c| c == '_' || is_xid_continue(c)) {
            self.bump();
        }
        let s = &self.src[start.offset..self.pos];
        let kind = Keyword::from_str(s)
            .map(TokenKind::Keyword)
            .unwrap_or_else(|| TokenKind::Identifier(s.into()));
        Some(self.token(start, kind))
    }
    fn number(&mut self, start: Position) -> Option<Token> {
        let mut float = false;
        if self.starts("0x") || self.starts("0X") {
            self.bump();
            self.bump();
            let digits = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                self.bump();
            }
            if self.peek() == Some('.') {
                float = true;
                self.bump();
                while self.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                    self.bump();
                }
            }
            if self.pos == digits || (self.pos == digits + 1 && float) {
                self.error(start, "hexadecimal literal requires digits");
                return None;
            }
            if self.peek().is_some_and(|c| c == 'p' || c == 'P') {
                float = true;
                self.bump();
                if self.peek().is_some_and(|c| c == '+' || c == '-') {
                    self.bump();
                }
                let exponent = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.bump();
                }
                if self.pos == exponent {
                    self.error(start, "hexadecimal floating exponent requires digits");
                    return None;
                }
            } else if float {
                self.error(start, "hexadecimal floating literal requires a p exponent");
                return None;
            }
        } else {
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
            }
            if self.peek() == Some('.') {
                float = true;
                self.bump();
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.bump();
                }
            }
            if self.peek().is_some_and(|c| c == 'e' || c == 'E') {
                float = true;
                self.bump();
                if self.peek().is_some_and(|c| c == '+' || c == '-') {
                    self.bump();
                }
                let exponent = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.bump();
                }
                if self.pos == exponent {
                    self.error(start, "floating exponent requires digits");
                    return None;
                }
            }
        }
        while self
            .peek()
            .is_some_and(|c| matches!(c, 'u' | 'U' | 'l' | 'L' | 'f' | 'F'))
        {
            if matches!(self.peek(), Some('f' | 'F')) {
                float = true
            }
            self.bump();
        }
        let raw = self.src[start.offset..self.pos].to_string();
        Some(self.token(
            start,
            TokenKind::Literal(if float {
                Literal::Floating(raw)
            } else {
                Literal::Integer(raw)
            }),
        ))
    }
    fn quoted(
        &mut self,
        start: Position,
        quote: char,
        prefix_len: usize,
        encoding: StringEncoding,
    ) -> Option<Token> {
        for _ in 0..prefix_len {
            self.bump();
        }
        self.bump();
        let mut escaped = false;
        while let Some(c) = self.peek() {
            self.bump();
            if c == '\n' && !escaped {
                self.error(start, "newline in literal");
                return None;
            }
            if c == quote && !escaped {
                let raw = self.src[start.offset..self.pos].to_string();
                let value = raw[prefix_len + 1..raw.len() - 1].to_string();
                return Some(self.token(
                    start,
                    TokenKind::Literal(if quote == '\'' {
                        Literal::Character { value, encoding }
                    } else {
                        Literal::String { value, encoding }
                    }),
                ));
            }
            escaped = c == '\\' && !escaped;
            if c != '\\' {
                escaped = false
            }
        }
        self.error(start, "unterminated literal");
        None
    }
    fn punctuator(&mut self, start: Position) -> Option<Token> {
        use TokenKind::*;
        let pairs = [
            ("<<=", ShlEq),
            (">>=", ShrEq),
            ("...", Ellipsis),
            ("->", Arrow),
            ("++", Inc),
            ("--", Dec),
            ("==", Eq),
            ("!=", Ne),
            ("<=", Le),
            (">=", Ge),
            ("&&", And),
            ("||", Or),
            ("<<", Shl),
            (">>", Shr),
            ("+=", PlusEq),
            ("-=", MinusEq),
            ("*=", StarEq),
            ("/=", SlashEq),
            ("%=", PercentEq),
            ("&=", AmpEq),
            ("|=", PipeEq),
            ("^=", CaretEq),
        ];
        for (s, k) in pairs {
            if self.starts(s) {
                for _ in s.chars() {
                    self.bump();
                }
                return Some(self.token(start, k));
            }
        }
        let c = self.bump().unwrap();
        let kind = match c {
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '/' => Slash,
            '%' => Percent,
            '&' => Amp,
            '|' => Pipe,
            '^' => Caret,
            '~' => Tilde,
            '!' => Bang,
            '=' => Assign,
            '<' => Lt,
            '>' => Gt,
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBracket,
            ']' => RBracket,
            ',' => Comma,
            ';' => Semi,
            ':' => Colon,
            '.' => Dot,
            '?' => Question,
            _ => {
                self.error(start, format!("unexpected character {c:?}"));
                return None;
            }
        };
        Some(self.token(start, kind))
    }
}
