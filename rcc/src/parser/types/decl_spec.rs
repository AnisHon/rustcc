use std::rc::Rc;
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::types::ast::decl::{Decl, DeclGroup, EnumField};
use crate::parser::types::ast::expr::Expr;
use crate::parser::types::common::{Ident, IdentList};
use crate::parser::types::declarator::*;
use crate::types::span::{Pos, Span};

pub type TypeQualType = [Option<TypeQual>; 3];

#[derive(Debug, Clone)]
pub struct DeclSpec {
    pub storage: Option<StorageSpec>, // 全局上下文的时候默认extern
    pub type_spec: TypeSpec,
    pub type_quals: TypeQualType,
    pub func_spec: Option<FuncSpec>,
    pub span: Span
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            StorageSpecKind::Typedef => "typedef",
            StorageSpecKind::Extern => "extern",
            StorageSpecKind::Static => "static",
            StorageSpecKind::Auto => "auto",
            StorageSpecKind::Register => "register"
        }
    }
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
    Struct(Decl),
    Union(Decl),
    Enum(Decl),
    TypeName(Ident)
}

impl TypeSpecKind {
    pub fn is_same(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
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

    pub fn kind_str(&self) -> &'static str {
        match &self.kind {
            TypeSpecKind::Void => "void",
            TypeSpecKind::Char => "char",
            TypeSpecKind::Short => "short",
            TypeSpecKind::Int => "int",
            TypeSpecKind::Long => "long",
            TypeSpecKind::Float => "float",
            TypeSpecKind::Double => "double",
            TypeSpecKind::Signed => "signed",
            TypeSpecKind::Unsigned => "unsigned",
            TypeSpecKind::Struct(_) => "struct",
            TypeSpecKind::Union(_) => "union",
            TypeSpecKind::Enum(_) => "enum",
            TypeSpecKind::TypeName(_) => "type-name"
        }
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
    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            TypeQualKind::Const => "const",
            TypeQualKind::Restrict => "restrict",
            TypeQualKind::Volatile => "volatile",
        }
    }
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
    pub fn kind_str(&self) -> &'static str {
        match self.kind { FuncSpecKind::Inline => "inline" }
    }
}

#[derive(Clone, Debug)]
pub enum ParamDecl {
    Idents(IdentList),
    Params(ParamList),
}

#[derive(Clone, Debug)]
pub struct ParamList {
    pub params: Vec<Decl>,
    pub commas: Vec<Pos>,
    pub ellipsis: Option<Span>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructSpecBody {
    pub l: Pos,
    pub group: Vec<DeclGroup>,
    pub r: Pos,
}

// struct or union
#[derive(Clone, Debug)]
pub struct StructSpec {
    pub kw: Token,
    pub name: Option<Ident>,
    pub body: Option<StructSpecBody>,
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
pub struct EnumSpecBody {
    pub l: Pos,
    pub list: EnumFieldList,
    pub r: Pos,
}

#[derive(Clone, Debug)]
pub struct EnumSpec {
    pub enum_span: Span, // 关键字enum的span
    pub name: Option<Ident>,
    pub body: Option<EnumSpecBody>,
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
pub struct EnumFieldList {
    pub decls: Vec<EnumField>,
    pub commas: Vec<Pos>,
    pub span: Span
}

impl Default for EnumFieldList {
    fn default() -> Self {
        Self {
            decls: Vec::new(),
            commas: Vec::new(),
            span: Span::default(),
        }
    }
}
