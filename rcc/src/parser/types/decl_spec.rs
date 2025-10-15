use std::rc::Rc;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::{Ident, IdentList};
use crate::parser::types::declarator::*;
use crate::types::span::{Pos, Span};

pub type TypeQualType = [Option<TypeQual>; 3];

#[derive(Debug, Clone)]
pub struct DeclSpec {
    pub storage: StorageSpec,
    pub type_spec: TypeSpec,
    pub type_quals: TypeQualType,
    pub func_spec: Option<FuncSpec>,
    pub span: Span
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

impl Default for StorageSpec {
    fn default() -> StorageSpec {
        Self {
            kind: StorageSpecKind::Extern,
            span: Span::default(),
        }
    }
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
    Struct(StructSpec),
    Union(StructSpec),
    Enum(EnumSpec),
    Typedef(Ident)
}

#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub kind: TypeSpecKind,
    pub span: Span
}

impl TypeSpec {
    pub fn new(token: Token) -> Self {
        use Keyword::*;
        let keyword = token.kind.into_keyword().unwrap();
        let kind = match keyword {
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
        };
        Self { kind, span: token.span }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeQualKind {
    Const = 0,
    Restrict = 1,
    Volatile = 2,
}

#[derive(Debug, Clone)]
#[derive(Copy)]
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
pub enum ParamDecl {
    Idents(IdentList),
    Params(ParamVarDeclList),
}

#[derive(Clone, Debug)]
pub struct ParamVarDeclList {
    pub params: Vec<Declarator>,
    pub commas: Vec<Pos>,
    pub ellipsis: Option<Span>,
    pub span: Span,
}

// struct or union
#[derive(Clone, Debug)]
pub struct StructSpec {
    pub struct_span: Span,
    pub name: Option<Ident>,
    pub l: Option<Pos>,
    pub var_decls: Option<Vec<StructVar>>,
    pub r: Option<Pos>,
    pub span: Span
}

#[derive(Clone, Debug)]
pub struct StructVar {
    pub decl_spec: Rc<DeclSpec>,
    pub declarators: Vec<StructDeclarator>,
    pub commas: Vec<Pos>,
    pub semi: Pos,
    pub span: Span,
}

impl StructVar {
    pub fn new(decl_spec: Rc<DeclSpec>) -> Self {
        Self {
            decl_spec,
            declarators: Vec::new(),
            commas: Vec::new(),
            semi: Pos::default(),
            span: Span::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructDeclarator {
    pub declarator: Declarator,
    pub colon: Option<Pos>,
    pub bit_field: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumSpec {
    pub enum_span: Span, // 关键字enum的span
    pub name: Option<Ident>,
    pub l: Option<Pos>,
    pub enumerators: Option<EnumeratorList>,
    pub r: Option<Pos>,
    pub span: Span
}

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub ident: Ident,
    pub eq: Option<Pos>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumeratorList {
    pub decls: Vec<Enumerator>,
    pub commas: Vec<Pos>,
    pub span: Span
}

