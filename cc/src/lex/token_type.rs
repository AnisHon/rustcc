macro_rules! define_token {
    ($($name:ident $value:expr),* $(,)?) => {
        #[repr(isize)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum TokenKind1 {
            $(
                $name = $value as isize,
            )*
        }

        impl TokenKind1 {
            pub fn from_isize(value: isize) -> Option<Self> {
                match value {
                    $(
                        $value => Some(Self::$name),
                    )*
                    _ => None,
                }
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    KeywordAuto,
    KeywordBreak,
    KeywordCase,
    KeywordChar,
    KeywordConst,
    KeywordContinue,
    KeywordDefault,
    KeywordDo,
    KeywordDouble,
    KeywordElse,
    KeywordEnum,
    KeywordExtern,
    KeywordFloat,
    KeywordFor,
    KeywordGoto,
    KeywordIf,
    KeywordInt,
    KeywordLong,
    KeywordRegister,
    KeywordReturn,
    KeywordShort,
    KeywordSigned,
    KeywordSizeof,
    KeywordStatic,
    KeywordStruct,
    KeywordSwitch,
    KeywordTypedef,
    KeywordUnion,
    KeywordUnsigned,
    KeywordVoid,
    KeywordVolatile,
    KeywordWhile,
    OpEllipsis,
    OpArrow,
    OpInc,
    OpDec,
    OpAddAssign,
    OpSubAssign,
    OpMulAssign,
    OpDivAssign,
    OpModAssign,
    OpLShiftAssign,
    OpRShiftAssign,
    OpAndAssign,
    OpXorAssign,
    OpOrAssign,
    OpEq,
    OpNe,
    OpGe,
    OpLe,
    OpAnd,
    OpOr,
    OpLShift,
    OpRShift,
    OpPlus,
    OpMinus,
    OpTimes,
    OpDivide,
    OpMod,
    OpNot,
    OpBitand,
    OpBitor,
    OpXor,
    OpBitNot,
    OpAssign,
    OpGt,
    OpLt,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Lbracket,
    Rbracket,
    Semicolon,
    Comma,
    Dot,
    Question,
    Colon,
    Id,
    Hex,
    Oct,
    Int,
    Float,
    StringLiteral,
    CharacterConstant,
    LineComment,
    BlockComment,
    Whitespace,
    TypeName,

}

define_token!(
    Auto    1,
    a       2
);



