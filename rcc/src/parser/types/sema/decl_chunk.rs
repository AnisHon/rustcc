use crate::lex::types::token::Token;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::types::ast::decl::{Initializer};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::Ident;
use crate::types::span::Span;

#[derive(Debug, Clone)]
pub enum DeclSpec {
    Storage(StorageSpec),
    TypeSpec(TypeSpec),
    TypeQual(TypeQual),
    FuncSpec(FuncSpec),
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
    Struct(StructOrUnionSpec),
    Union(StructOrUnionSpec),
    Enum(EnumSpec),
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



#[derive(Clone, Debug)]
pub struct Declarator {
    pub name: Option<Ident>,
    pub decl_specs: Vec<DeclSpec>,
    pub chunks: Vec<DeclaratorChunk>,
    pub span: Span
}

impl Declarator {
    pub fn new(decl_specs: Vec<DeclSpec>, chunks: Vec<DeclaratorChunk>, span: Span) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum DeclaratorChunkKind {
    Ident(Ident),
    Paren { l: Span, declarator: Declarator, r: Span },
    Array { l: Span, type_quals: Option<Vec<TypeQual>>, expr: Option<Box<Expr>>, r: Span },
    Pointer { star: Span, type_quals: Vec<TypeQual> },
    Function { l: Span, param: ParamDecl, r: Span },
}

#[derive(Clone, Debug)]
pub struct DeclaratorChunk {
    pub kind: DeclaratorChunkKind,
    pub span: Span
}

impl DeclaratorChunk {
    pub fn new(kind: DeclaratorChunkKind, span: Span) -> DeclaratorChunk {
        Self { kind, span }
    }
}



#[derive(Clone, Debug)]
pub enum ParamDecl {
    Idents(IdentList),
    Params {
        params: Vec<Declarator>,
        commas: Vec<Span>,
        ellipsis: Option<Span>,
    },
}

#[derive(Clone, Debug)]
pub struct StructOrUnionSpec {
    pub struct_span: Span,
    pub name: Option<Ident>,
    pub l: Option<Span>,
    pub var_decls: Option<Vec<StructVarDecl>>,
    pub r: Option<Span>,
    pub span: Span
}

#[derive(Clone, Debug)]
pub struct StructVarDecl {
    pub spec_quals: Vec<DeclSpec>,
    pub list: StructDeclaratorList,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDeclarator {
    pub declarator: Option<Declarator>,
    pub colon: Option<Span>,
    pub bit_field: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDeclaratorList {
    pub declarators: Vec<StructDeclarator>,
    pub commas: Vec<Span>,
    pub span: Span
}

#[derive(Clone, Debug)]
pub struct EnumSpec {
    
}

pub struct EnumeratorDecl {
    pub ident: Ident,
    pub eq: Option<Span>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

pub struct EnumeratorDeclList {
    pub decls: Vec<EnumeratorDecl>,
    pub commas: Vec<Span>,
    pub span: Span
}

#[derive(Clone, Debug)]

pub struct IdentList {
    pub idents: Vec<Ident>,
    pub commas: Vec<Span>,
    pub span: Span
}

impl IdentList {
    pub fn new() -> Self {
        Self {
            idents: Vec::new(),
            commas: Vec::new(),
            span: Span::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub eq: Option<Span>,
    pub init: Option<Initializer>,
}

#[derive(Clone, Debug)]
pub struct InitDeclaratorList {
    pub inits: Vec<InitDeclarator>,
    pub commas: Vec<Span>,
    pub span: Span
}

impl InitDeclaratorList {
    pub fn new() -> Self {
        Self { inits: Vec::new(), commas: Vec::new(), span: Span::default() }
    }
}

#[derive(Clone, Debug)]
pub struct InitializerList {
    pub inits: Vec<Initializer>,
    pub commas: Vec<Span>,
    pub span: Span
}

impl InitializerList {
    pub fn new() -> Self {
        Self { inits: Vec::new(), commas: Vec::new(), span: Span::default() }
    }
}

