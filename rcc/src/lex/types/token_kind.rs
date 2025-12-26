use enum_as_inner::EnumAsInner;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};

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

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,    // &
    Pipe,   // |
    Caret,  // ^
    Tilde,  // ~
    Bang,   // !
    Assign, // =
    Lt,
    Gt, // < >

    LParen,
    RParen,
    LBrace,
    RBrace, // ( ) { }
    LBracket,
    RBracket, // [ ]
    Comma,
    Semi,
    Colon,
    Dot,
    Arrow, // , ; : . ->
    Question,
    Ellipsis, // ? ...

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
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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
    Bool,      // _Bool
    Complex,   // _Complex
    Imaginary, // _Imaginary
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, EnumAsInner)]
pub enum LiteralKind {
    Integer {
        value: Symbol,
        suffix: Option<IntSuffix>,
    },
    Float {
        value: Symbol,
        suffix: Option<FloatSuffix>,
    }, // float交给后期解析
    Char {
        value: Symbol,
    },
    String {
        value: Symbol,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum IntSuffix {
    U,
    L,
    UL,
    LL,
    ULL,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum FloatSuffix {
    F,
    L,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use TokenKind::*;
        let str = match self {
            Ident(_) => "identifier".to_owned(),
            Keyword(x) => x.to_string(),
            Literal(x) => x.to_string(),
            Plus => "+".to_owned(),
            Minus => "-".to_owned(),
            Star => "*".to_owned(),
            Slash => "/".to_owned(),
            Percent => "%".to_owned(),
            Amp => "&".to_owned(),
            Pipe => "|".to_owned(),
            Caret => "caret".to_owned(),
            Tilde => "~".to_owned(),
            Bang => "!".to_owned(),
            Assign => "=".to_owned(),
            Lt => "<".to_owned(),
            Gt => ">".to_owned(),
            LParen => "(".to_owned(),
            RParen => ")".to_owned(),
            LBrace => "{".to_owned(),
            RBrace => "}".to_owned(),
            LBracket => "[".to_owned(),
            RBracket => "]".to_owned(),
            Comma => ",".to_owned(),
            Semi => ";".to_owned(),
            Colon => ":".to_owned(),
            Dot => ".".to_owned(),
            Arrow => "->".to_owned(),
            Question => "?".to_owned(),
            Ellipsis => "...".to_owned(),
            Eq => "==".to_owned(),
            Ne => "!=".to_owned(),
            Le => "<=".to_owned(),
            Ge => ">=".to_owned(),
            And => "&&".to_owned(),
            Or => "||".to_owned(),
            Shl => "<<".to_owned(),
            Shr => ">>".to_owned(),
            PlusEq => "+=".to_owned(),
            MinusEq => "-=".to_owned(),
            StarEq => "*=".to_owned(),
            SlashEq => "/=".to_owned(),
            PercentEq => "%=".to_owned(),
            ShlEq => "<<=".to_owned(),
            ShrEq => ">>=".to_owned(),
            AmpEq => "&=".to_owned(),
            PipeEq => "|=".to_owned(),
            CaretEq => "^=".to_owned(),
            Inc => "++".to_owned(),
            Dec => "--".to_owned(),
            Eof => "eof".to_owned(),
        };
        write!(f, "{}", str)
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
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
        };
        write!(f, "{}", msg)
    }
}

impl Display for LiteralKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            LiteralKind::Integer { .. } => "integer",
            LiteralKind::Float { .. } => "float",
            LiteralKind::Char { .. } => "char",
            LiteralKind::String { .. } => "string",
        };
        write!(f, "{}", str)
    }
}
