use crate::lex::types::token::Token;
use crate::lex::types::token_kind::Keyword;
use crate::lex::types::token_kind::TokenKind;
use crate::parser::ast::DeclKey;
use crate::parser::ast::ExprKey;
use crate::parser::ast::TypeKey;
use crate::parser::ast::common::StructOrUnion;
use crate::parser::semantic::ast::decl::DeclGroup;
use crate::parser::semantic::common::{Ident, IdentList};
use crate::parser::semantic::declarator::*;
use crate::parser::semantic::sema::type_ctx::type_builder::TypeBuilderKind;
use crate::types::span::{Pos, Span};
use enum_as_inner::EnumAsInner;
use std::fmt::{Display, Formatter};

///
/// # Members
/// - `storage`:
/// - `type_base`: Void Int Double Enum Struct TypeName
/// - `type_size`: Char Short Long Longlong Float Double LongDouble
/// - `signed`: Signed Unsigned
/// - `type_quals`:
/// - `func_spec`:
/// - `span`:
#[derive(Debug, Clone)]
pub struct DeclSpec {
    pub storage: Option<StorageSpec>, // 全局上下文的时候默认extern
    pub kind: TypeBuilderKind,
    pub type_quals: TypeQuals,
    pub func_spec: Option<FuncSpec>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, EnumAsInner)]
pub enum StorageSpecKind {
    Typedef,
    Extern,
    Static,
    Auto,
    Register,
}

#[derive(Debug, Clone)]
pub struct StorageSpec {
    pub kind: StorageSpecKind,
    pub span: Span,
}

impl Display for StorageSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use StorageSpecKind::*;
        let str = match self.kind {
            Typedef => "typedef",
            Extern => "extern",
            Static => "static",
            Auto => "auto",
            Register => "register",
        };
        write!(f, "{}", str)
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
                _ => unreachable!(),
            },
            _ => unreachable!("{:?}", token),
        };
        Self {
            kind,
            span: token.span,
        }
    }

    pub fn from_kind(kind: StorageSpecKind) -> Self {
        Self {
            kind,
            span: Span::default(),
        }
    }
}

#[derive(Debug, Clone, EnumAsInner)]
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
    Record(DeclKey),
    Enum(DeclKey),
    TypeName(Ident, DeclKey),
}

impl TypeSpecKind {
    pub fn new(kw: Keyword) -> Self {
        use Keyword::*;
        let kind = match kw {
            Void => TypeSpecKind::Void,
            Char => TypeSpecKind::Char,
            Short => TypeSpecKind::Short,
            Int => TypeSpecKind::Int,
            Long => TypeSpecKind::Long,
            Float => TypeSpecKind::Float,
            Double => TypeSpecKind::Double,
            Signed => TypeSpecKind::Signed,
            Unsigned => TypeSpecKind::Unsigned,
            _ => unreachable!(),
        };

        kind
    }
}

#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub kind: TypeSpecKind,
    pub span: Span,
}

impl TypeSpec {
    pub fn new(token: Token) -> Self {
        let keyword = token.kind.into_keyword().unwrap();
        let kind = TypeSpecKind::new(keyword);
        Self {
            kind,
            span: token.span,
        }
    }

    pub fn from_kind(kind: TypeSpecKind) -> Self {
        Self {
            kind,
            span: Span::default(),
        }
    }

    pub fn is(&self, kind: &TypeSpecKind) -> bool {
        std::mem::discriminant(&self.kind) == std::mem::discriminant(kind)
    }
}

impl Display for TypeSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match &self.kind {
            TypeSpecKind::Void => "void",
            TypeSpecKind::Char => "char",
            TypeSpecKind::Short => "short",
            TypeSpecKind::Int => "int",
            TypeSpecKind::Long => "long",
            TypeSpecKind::Float => "float",
            TypeSpecKind::Double => "double",
            TypeSpecKind::Signed => "signed",
            TypeSpecKind::Unsigned => "unsigned",
            TypeSpecKind::Record(_) => "record",
            TypeSpecKind::Enum(_) => "enum",
            TypeSpecKind::TypeName(_, _) => "type-name",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeQualKind {
    Const,
    Restrict,
    Volatile,
}

#[derive(Debug, Clone, Copy)]
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
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        Self {
            kind,
            span: token.span,
        }
    }
}

impl Display for TypeQual {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TypeQualKind::*;
        let str = match self.kind {
            Const => "const",
            Restrict => "restrict",
            Volatile => "volatile",
        };
        write!(f, "{}", str) 
    }
}
/// Qualifier
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeQuals {
    pub is_const: Option<TypeQual>,
    pub is_restrict: Option<TypeQual>,
    pub is_volatile: Option<TypeQual>,
}

#[derive(Debug, Clone)]
pub enum FuncSpecKind {
    Inline,
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
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
        Self {
            kind,
            span: token.span,
        }
    }
}

impl Display for FuncSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self.kind {
            FuncSpecKind::Inline => "inline",
        };
        write!(f, "{}", str)
    } 
}

#[derive(Clone, Debug)]
pub enum ParamDecl {
    Idents(IdentList),
    Params(ParamList),
}

#[derive(Clone, Debug)]
pub struct ParamList {
    pub params: Vec<DeclKey>,
    pub is_variadic: bool,
    pub span: Span,
}

impl Default for ParamList {
    fn default() -> Self {
        Self {
            params: Vec::new(),
            is_variadic: false,
            span: Span::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructSpecBody {
    pub l: Pos,
    pub groups: Vec<DeclGroup>,
    pub r: Pos,
}

// struct or union
#[derive(Clone, Debug)]
pub struct StructSpec {
    pub kind: StructOrUnion,
    pub name: Option<Ident>,
    pub body: Option<StructSpecBody>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDeclarator {
    pub declarator: Declarator,
    pub bit_field: Option<ExprKey>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub name: Ident,
    pub expr: Option<ExprKey>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumSpec {
    pub enum_span: Span, // 关键字enum的span
    pub name: Option<Ident>,
    pub body: Option<Vec<DeclKey>>, // 内部声明，应该都是 enumerator
    pub span: Span,
}
