use crate::source::SourceRange;

/// C punctuators as recognized during preprocessing.
///
/// Alternative spellings such as digraphs map to the same semantic variant;
/// the original spelling remains available on `PPToken` for macro operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Punctuator {
    LBracket,
    RBracket,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Dot,
    Arrow,
    Inc,
    Dec,
    Amp,
    Star,
    Plus,
    Minus,
    Tilde,
    Bang,
    Slash,
    Percent,
    Shl,
    Shr,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    Caret,
    Pipe,
    And,
    Or,
    Question,
    Colon,
    Semi,
    Ellipsis,
    Assign,
    StarEq,
    SlashEq,
    PercentEq,
    PlusEq,
    MinusEq,
    ShlEq,
    ShrEq,
    AmpEq,
    CaretEq,
    PipeEq,
    Comma,
    Hash,
    HashHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The deliberately small set of preprocessing-token categories from C11 6.4.
///
/// Identifiers are not classified as keywords yet, and numeric spelling is not
/// parsed yet. Those decisions happen after macro expansion.
pub enum PPTokenKind {
    Invalid,
    Identifier,
    Number,
    Character,
    String,
    Punctuator(Punctuator),
    NewLine,
    EndOfFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A raw or macro-produced preprocessing token.
///
/// Whitespace flags are semantic input to directives, stringification, and
/// function-like macro recognition; `range` preserves spelling/expansion provenance.
pub struct PPToken {
    pub kind: PPTokenKind,
    pub spelling: String,
    pub range: SourceRange,
    pub leading_space: bool,
    pub start_of_line: bool,
}
