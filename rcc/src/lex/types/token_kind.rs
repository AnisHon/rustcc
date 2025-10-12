use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use enum_as_inner::EnumAsInner;

thread_local! {
    static SYMBOL_INTERNER: RefCell<Interner> = RefCell::new(Interner::new());
}
///
pub struct Interner {
    names: Vec<&'static str>,
    indices: FxHashMap<&'static str, Symbol>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            indices: FxHashMap::default(),
        }
    }

    /// 获取一个Symbol
    fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&sym) = self.indices.get(s) {
            return sym;
        }
        let owned = s.to_string();
        let leaked: &'static str = Box::leak(owned.into_boxed_str());
        let sym = Symbol(self.names.len());
        self.names.push(leaked);
        self.indices.insert(leaked, sym);
        sym
    }

    /// 通过Symbol获取字符串
    fn get(&self, sym: Symbol) -> &'static str {
        self.names[sym.0]
    }
}

/// 主体 Token 枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, EnumAsInner)]
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
#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct Symbol(usize);

impl Symbol {
    
    pub fn new(patten: &str) -> Self {
        SYMBOL_INTERNER.with_borrow_mut(|interner| interner.intern(patten))
    }
    pub fn get(self) -> &'static str {
        SYMBOL_INTERNER.with_borrow(|interner| interner.get(self))
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Symbol({:?})", self.get())
    }
}

/// 标记符类别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, EnumAsInner)]
pub enum LiteralKind {
    Integer { value: u64, suffix: Option<IntSuffix> },
    Float   { value: Symbol, suffix: Option<FloatSuffix> }, // float交给后期解析
    Char    { value: Symbol },
    String  { value: Symbol },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum IntSuffix {
    U, L, UL, LL, ULL,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum FloatSuffix {
    F, L,
}


impl TokenKind {
    pub fn kind_str(&self) -> &'static str {
        match self {
            TokenKind::Ident(_) => "identifier",
            TokenKind::Keyword(x) => x.kind_str(),
            TokenKind::Literal(x) => x.kind_str(),
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Amp => "&",
            TokenKind::Pipe => "|",
            TokenKind::Caret => "caret",
            TokenKind::Tilde => "~",
            TokenKind::Bang => "!",
            TokenKind::Assign => "=",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Semi => ";",
            TokenKind::Colon => ":",
            TokenKind::Dot => ".",
            TokenKind::Arrow => "->",
            TokenKind::Question => "?",
            TokenKind::Ellipsis => "...",
            TokenKind::Eq => "==",
            TokenKind::Ne => "!=",
            TokenKind::Le => "<=",
            TokenKind::Ge => ">=",
            TokenKind::And => "&&",
            TokenKind::Or => "||",
            TokenKind::Shl => "<<",
            TokenKind::Shr => ">>",
            TokenKind::PlusEq => "+=",
            TokenKind::MinusEq => "-=",
            TokenKind::StarEq => "*=",
            TokenKind::SlashEq => "/=",
            TokenKind::PercentEq => "%=",
            TokenKind::ShlEq => "<<=",
            TokenKind::ShrEq => ">>=",
            TokenKind::AmpEq => "&=",
            TokenKind::PipeEq => "|=",
            TokenKind::CaretEq => "^=",
            TokenKind::Inc => "++",
            TokenKind::Dec => "--",
            TokenKind::Eof => "eof",
        }
    }
}

impl Keyword {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Keyword::Auto => "auto",
            Keyword::Break => "break",
            Keyword::Case => "case",
            Keyword::Char => "char",
            Keyword::Const => "const",
            Keyword::Continue => "continue",
            Keyword::Default => "default",
            Keyword::Do => "do",
            Keyword::Double => "double",
            Keyword::Else => "else",
            Keyword::Enum => "enum",
            Keyword::Extern => "extern",
            Keyword::Float => "float",
            Keyword::For => "for",
            Keyword::Goto => "goto",
            Keyword::If => "if",
            Keyword::Inline => "inline",
            Keyword::Int => "int",
            Keyword::Long => "long",
            Keyword::Register => "register",
            Keyword::Restrict => "restrict",
            Keyword::Return => "return",
            Keyword::Short => "short",
            Keyword::Signed => "signed",
            Keyword::Sizeof => "sizeof",
            Keyword::Static => "static",
            Keyword::Struct => "struct",
            Keyword::Switch => "switch",
            Keyword::Typedef => "typedef",
            Keyword::Union => "union",
            Keyword::Unsigned => "unsigned",
            Keyword::Void => "void",
            Keyword::Volatile => "volatile",
            Keyword::While => "while",
            Keyword::Bool => "_Bool",
            Keyword::Complex => "_Complex",
            Keyword::Imaginary => "_Imaginary",
        }
    }
}

impl LiteralKind {
    pub fn kind_str(&self) -> &'static str {
        match self {
            LiteralKind::Integer { .. } => "integer",
            LiteralKind::Float { .. } => "float",
            LiteralKind::Char { .. } => "char",
            LiteralKind::String { .. } => "string",
        }
    }
}
