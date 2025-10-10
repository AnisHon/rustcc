use std::fmt::format;
use crate::err::lex_error::LexResult;
use crate::util::utf8::unescape_str;

/// 主体 Token 枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(Symbol),
    Keyword(Keyword),

    Literal(LiteralKind),

    Plus, Minus, Star, Slash, Percent,
    Amp,    // &
    Pipe,   // |
    Caret,  // ^
    Tilde,  // ~
    Bang,   // !
    Assign, // =
    Lt, Gt, // < >

    LParen, RParen, LBrace, RBrace, // ( ) { }
    LBracket, RBracket, // [ ]
    Comma, Semi, Colon, Dot, Arrow, // , ; : . ->
    Question, Ellipsis, // ? ...

    Eq, Ne, Le, Ge, And, Or, Shl, Shr,
    PlusEq, MinusEq, StarEq, SlashEq, PercentEq,
    ShlEq, ShrEq, AmpEq, PipeEq, CaretEq,
    Inc, Dec,

    // Comment(Symbol),  // 可选保留注释文本
    Eof,
}

/// 符号池中的索引（用于标识符/字符串等）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

impl Symbol {
    
    pub fn new(string: &str) -> Self {
        Self(string.to_owned())
    }
    pub fn get(self) -> String {
        self.0
    }
}

/// 标记符类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Bool,       // _Bool
    Complex,    // _Complex
    Imaginary,  // _Imaginary
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Integer { value: u64, suffix: Option<IntSuffix> },
    Float   { value: String, suffix: Option<FloatSuffix> }, // float交给后期解析
    Char    { value: String },
    String  { value: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSuffix {
    U, L, UL, LL, ULL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSuffix {
    F, L,
}



