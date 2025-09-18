use crate::lex::decl_yy::*;
use num_derive::FromPrimitive;

macro_rules! define_token {
    ($($name:ident $value:expr),* $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive)]
        pub enum TokenKind {
            $(
                $name = $value as isize,
            )*
        }
    };
}

/// 对于几个中间值，单独设置，这些值不会到达parser，
/// 10001 开始一定不会冲突
const TOKEN_HEX: isize = 10001;
const TOKEN_OCT: isize = 10002;

define_token!(
    KeywordAuto         KEYWORD_AUTO,
    KeywordBreak        KEYWORD_BREAK,
    KeywordCase         KEYWORD_CASE,
    KeywordChar         KEYWORD_CHAR,
    KeywordConst        KEYWORD_CONST,
    KeywordContinue     KEYWORD_CONTINUE,
    KeywordDefault      KEYWORD_DEFAULT,
    KeywordDo           KEYWORD_DO,
    KeywordDouble       KEYWORD_DOUBLE,
    KeywordElse         KEYWORD_ELSE,
    KeywordEnum         KEYWORD_ENUM,
    KeywordExtern       KEYWORD_EXTERN,
    KeywordFloat        KEYWORD_FLOAT,
    KeywordFor          KEYWORD_FOR,
    KeywordGoto         KEYWORD_GOTO,
    KeywordIf           KEYWORD_IF,
    KeywordInt          KEYWORD_INT,
    KeywordLong         KEYWORD_LONG,
    KeywordRegister     KEYWORD_REGISTER,
    KeywordReturn       KEYWORD_RETURN,
    KeywordShort        KEYWORD_SHORT,
    KeywordSigned       KEYWORD_SIGNED,
    KeywordSizeof       KEYWORD_SIZEOF,
    KeywordStatic       KEYWORD_STATIC,
    KeywordStruct       KEYWORD_STRUCT,
    KeywordSwitch       KEYWORD_SWITCH,
    KeywordTypedef      KEYWORD_TYPEDEF,
    KeywordUnion        KEYWORD_UNION,
    KeywordUnsigned     KEYWORD_UNSIGNED,
    KeywordVoid         KEYWORD_VOID,
    KeywordVolatile     KEYWORD_VOLATILE,
    KeywordWhile        KEYWORD_WHILE,
    OpEllipsis          OP_ELLIPSIS,
    OpArrow             OP_ARROW,
    OpInc               OP_INC,
    OpDec               OP_DEC,
    OpAddAssign         OP_ADD_ASSIGN,
    OpSubAssign         OP_SUB_ASSIGN,
    OpMulAssign         OP_MUL_ASSIGN,
    OpDivAssign         OP_DIV_ASSIGN,
    OpModAssign         OP_MOD_ASSIGN,
    OpLShiftAssign      OP_L_SHIFT_ASSIGN,
    OpRShiftAssign      OP_R_SHIFT_ASSIGN,
    OpAndAssign         OP_AND_ASSIGN,
    OpXorAssign         OP_XOR_ASSIGN,
    OpOrAssign          OP_OR_ASSIGN,
    OpEq                OP_EQ,
    OpNe                OP_NE,
    OpGe                OP_GE,
    OpLe                OP_LE,
    OpAnd               OP_AND,
    OpOr                OP_OR,
    OpLShift            OP_L_SHIFT,
    OpRShift            OP_R_SHIFT,
    OpPlus              '+',
    OpMinus             '-',
    OpTimes             '*',
    OpDivide            '/',
    OpMod               '%',
    OpNot               '!',
    OpBitand            '&',
    OpBitor             '|',
    OpXor               '^',
    OpBitNot            '~',
    OpAssign            '=',
    OpGt                '>',
    OpLt                '<',
    Lparen              '(',
    Rparen              ')',
    Lbrace              '{',
    Rbrace              '}',
    Lbracket            '[',
    Rbracket            ']',
    Semicolon           ';',
    Comma               ',',
    Dot                 '.',
    Question            '?',
    Colon               ':',
    Id                  ID,
    Hex                 TOKEN_HEX,  // 没定义
    Oct                 TOKEN_OCT,  // 没定义
    Int                 INT,
    Float               FLOAT,
    StringLiteral       STRING_LITERAL,
    CharacterConstant   CHARACTER_CONSTANT,
    TypeName            TYPE_NAME,
);

