use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl_chunk::InitializerList;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub enum Initializer {
    Expr(Box<Expr>),
    InitList{ l: Span, inits: InitializerList, r: Span },
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub ident: Ident,
    pub eq: Span,
    pub init: Option<Initializer>,
    pub semi: Span,
    pub span: Span,
}

impl Decl {
    // pub fn new(ident: Token, eq: Token, ) -> Span {
    //
    // }
}

#[derive(Debug, Clone)]
pub enum StorageSpecKind {
    Typedef,
    Extern,
    Static,
    Auto,
    Register
}

pub struct StorageSpec {
    pub kind: StorageSpecKind,
    pub span: Span,
}

impl StorageSpec {
    pub fn new(token: Token) -> Self {
        use Keyword::*;
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
        use Keyword::*;
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
        use Keyword::*;
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
        use Keyword::*;
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

pub enum StructOrUnionKind {
    Struct,
    Union,
}

pub struct StructOrUnion {
    pub kind: StructOrUnionKind,
    pub span: Span,
}

pub enum StructOrUnionDeclKind {
    Ref {
        ident: Ident,
    },
    Decl {
        ident: Option<Ident>,
        l: Span,
        fields: Vec<StructField>,
        r: Span,
    }
}

pub struct StructOrUnionDecl {
    struct_or_union: StructOrUnion,
    kind: StructOrUnionDeclKind,
    span: Span,
}

pub struct StructField {
    pub ident: Option<Ident>,
    pub colon: Option<Token>,
    pub bit_field: Option<Box<Expr>>,
    pub semi: Span,
}

pub struct Enumerator {
    ident: Ident,
    eq: Option<Span>,
    expr: Option<Box<Expr>>,
    span: Span,
}

pub struct EnumeratorList {
    pub enums: Vec<Enumerator>,
    pub commas: Vec<Span>,
}

pub struct EnumDecl {
    pub enum_span: Span,
    pub l: Span,
    pub enums: EnumeratorList,
    pub r: Span,
    pub span: Span
}
