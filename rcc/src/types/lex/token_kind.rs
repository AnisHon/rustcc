
/// Token 种类
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// 标识符
    Ident(Symbol),

    /// 关键字
    Keyword(Keyword),

    /// 字面量
    Literal { kind: LiteralKind, symbol: Symbol },

    /// 二元操作符
    BinOp(BinOp),

    /// 分隔符
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Comma, Semi, Dot,
    Assign,

    /// 文件结束
    Eof,
}

thread_local! {}

/// 符号池中的索引（用于标识符/字符串等）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Int,
    Float,     // 1.23
    Char,      // 'a'
    String,    // "hello"
}

/// 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Plus, Minus, Star, Slash, Percent,
    EqEq, NotEq, Lt, Le, Gt, Ge,
    AndAnd, OrOr,
}