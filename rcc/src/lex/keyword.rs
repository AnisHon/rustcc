use phf::phf_map;
use crate::lex::types::token_kind::Keyword;
use Keyword::*;

pub static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
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
    "int" => Int,
    "inline" => Inline,
    "long" => Long,
    "register" => Register,
    "restrict" => Restrict,
    "return" => Return,
    "short" => Short,
    "sizeof" => Sizeof,
    "signed" => Signed,
    "static" => Static,
    "struct" => Struct,
    "switch" => Switch,
    "typedef" => Typedef,
    "union" => Union,
    "unsigned" => Unsigned,
    "void" => Void,
    "volatile" => Volatile,
    "while" => While,
    "_Bool" => Bool,
    "_Complex" => Complex,
    "_Imaginary" => Imaginary,














};