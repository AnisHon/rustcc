use crate::source::SourceRange;

/// Encoding prefix attached to a character or string literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Narrow,
    Utf8,
    Utf16,
    Utf32,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Reserved C11 words after preprocessing has completed.
///
/// Keeping keyword classification out of `RawLexer` allows macro names and
/// replacement lists to follow preprocessing-token rules.
pub enum Keyword {
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Inline,
    Int,
    Long,
    Register,
    Restrict,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
    Alignas,
    Alignof,
    Atomic,
    Bool,
    Complex,
    Generic,
    Imaginary,
    Noreturn,
    StaticAssert,
    ThreadLocal,
}

impl Keyword {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        use Keyword::*;
        Some(match s {
            "auto" => Auto,
            "break" => Break,
            "case" => Case,
            "char" => Char,
            "const" => Const,
            "continue" => Continue,
            "default" => Default,
            "do" => Do,
            "double" => Double,
            "else" => Else,
            "enum" => Enum,
            "extern" => Extern,
            "float" => Float,
            "for" => For,
            "goto" => Goto,
            "if" => If,
            "inline" => Inline,
            "int" => Int,
            "long" => Long,
            "register" => Register,
            "restrict" => Restrict,
            "return" => Return,
            "short" => Short,
            "signed" => Signed,
            "sizeof" => Sizeof,
            "static" => Static,
            "struct" => Struct,
            "switch" => Switch,
            "typedef" => Typedef,
            "union" => Union,
            "unsigned" => Unsigned,
            "void" => Void,
            "volatile" => Volatile,
            "while" => While,
            "_Alignas" => Alignas,
            "_Alignof" => Alignof,
            "_Atomic" => Atomic,
            "_Bool" => Bool,
            "_Complex" => Complex,
            "_Generic" => Generic,
            "_Imaginary" => Imaginary,
            "_Noreturn" => Noreturn,
            "_Static_assert" => StaticAssert,
            "_Thread_local" => ThreadLocal,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Literal spelling classified for Parser/Sema consumption.
///
/// Numeric values stay textual here so Sema can choose a C type using suffix,
/// radix, target widths, and overflow rules.
pub enum Literal {
    Integer(String),
    Floating(String),
    Character {
        value: String,
        encoding: StringEncoding,
    },
    String {
        value: String,
        encoding: StringEncoding,
    },
}

#[derive(Debug, Clone, PartialEq)]
/// Language-token categories consumed by the C parser.
pub enum TokenKind {
    Identifier(String),
    Keyword(Keyword),
    Literal(Literal),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Bang,
    Assign,
    Lt,
    Gt,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Dot,
    Arrow,
    Question,
    Ellipsis,
    Eq,
    Ne,
    Le,
    Ge,
    And,
    Or,
    Shl,
    Shr,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    ShlEq,
    ShrEq,
    AmpEq,
    PipeEq,
    CaretEq,
    Inc,
    Dec,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
/// A post-preprocessing token with its original spelling and compact range.
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub range: SourceRange,
}
