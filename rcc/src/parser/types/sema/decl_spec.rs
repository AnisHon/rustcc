use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword::{Auto, Char, Const, Double, Extern, Float, Inline, Int, Long, Register, Restrict, Short, Signed, Static, Typedef, Unsigned, Void, Volatile};
use crate::lex::types::token_kind::TokenKind;
use crate::parser::types::common::Ident;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub struct DeclSpec {
    pub storages: Vec<StorageSpec>,
    pub type_specs: Vec<TypeSpec>,
    pub type_quals: Vec<TypeQual>,
    pub func_specs: Vec<FuncSpec>,
    pub span: Span
}

impl DeclSpec {
    pub fn new() -> Self {
        Self {
            storages: Vec::new(),
            type_specs: Vec::new(),
            type_quals: Vec::new(),
            func_specs: Vec::new(),
            span: Span::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecQualList {
    pub type_specs: Vec<TypeSpec>,
    pub type_quals: Vec<TypeQual>,
    pub span: Span
}

impl SpecQualList {
    pub fn new() -> Self {
        Self {
            type_specs: Vec::new(),
            type_quals: Vec::new(),
            span: Span::default()
        }
    }
}
#[derive(Debug, Clone)]
pub enum StorageSpecKind {
    Typedef,
    Extern,
    Static,
    Auto,
    Register
}

#[derive(Debug, Clone)]
pub struct StorageSpec {
    pub kind: StorageSpecKind,
    pub span: Span,
}

impl StorageSpec {
    pub fn new(token: Token) -> Self {
        use crate::lex::types::token_kind::Keyword::*;
        let kind = match token.kind {
            TokenKind::Keyword(kw) => match kw {
                Typedef => StorageSpecKind::Typedef,
                Extern => StorageSpecKind::Extern,
                Static => StorageSpecKind::Static,
                Auto => StorageSpecKind::Auto,
                Register => StorageSpecKind::Register,
                _ => unreachable!()
            }
            _ => unreachable!(),
        };
        Self { kind, span: token.span }
    }
}


#[derive(Debug, Clone)]
pub enum TypeSpecKind {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    Struct(),
    Union(),
    Enum(),
    Typedef(Ident)
}

#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub kind: TypeSpecKind,
    pub span: Span,
}
impl TypeSpec {
    pub fn new(token: Token) -> Self {
        use crate::lex::types::token_kind::Keyword::*;
        let kind = match token.kind {
            TokenKind::Keyword(kw) => match kw {
                Void => TypeSpecKind::Void,
                Char => TypeSpecKind::Char,
                Short => TypeSpecKind::Short,
                Int => TypeSpecKind::Int,
                Long => TypeSpecKind::Long,
                Float => TypeSpecKind::Float,
                Double => TypeSpecKind::Double,
                Signed => TypeSpecKind::Signed,
                Unsigned => TypeSpecKind::Unsigned,
                _ => unreachable!()
            }
            _ => unreachable!(),
        };
        Self { kind, span: token.span }
    }
}

#[derive(Debug, Clone)]
pub enum TypeQualKind {
    Const,
    Restrict,
    Volatile,
}

#[derive(Debug, Clone)]
pub struct TypeQual {
    pub kind: TypeQualKind,
    pub span: Span,
}

impl TypeQual {
    pub fn new(token: Token) -> Self {
        use crate::lex::types::token_kind::Keyword::*;
        let kind = match token.kind {
            TokenKind::Keyword(kw) => match kw {
                Const => TypeQualKind::Const,
                Restrict => TypeQualKind::Restrict,
                Volatile => TypeQualKind::Volatile,
                _ => unreachable!()
            }
            _ => unreachable!(),
        };
        Self { kind, span: token.span }
    }
}

#[derive(Debug, Clone)]
pub enum FuncSpecKind {
    Inline
}

#[derive(Debug, Clone)]
pub struct FuncSpec {
    pub kind: FuncSpecKind,
    pub span: Span,
}

impl FuncSpec {
    pub fn new(token: Token) -> Self {
        use crate::lex::types::token_kind::Keyword::*;
        let kind = match token.kind {
            TokenKind::Keyword(kw) => match kw {
                Inline => FuncSpecKind::Inline,
                _ => unreachable!()
            }
            _ => unreachable!(),
        };
        Self { kind, span: token.span }
    }
}

