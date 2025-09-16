//!
//! 声明相关中间节点的类型定义（或者说是未解析节点定义），辅助sema构造
//!
//! # Members
//! - `DeclSpec`:
//! - `TypeQual`:
//! - `DeclaratorInfo`:
//! - `DeclaratorChunk`:
//! - `ArraySizeModifier`:
//! - `ParamList`:
//! - `ParamInfo`:
//!

use crate::lex::lex_yy::TokenType;
use crate::types::span::{Delim, SepList, Span, UnwrapSpan};
use crate::types::ast::ast_nodes;
use crate::types::ast::ast_nodes::{ExpressionKind, StorageClass};
use crate::types::ast::parser_node::ParserNode;
use crate::types::ast::struct_info::{EnumSpec, StructOrUnionSpec};
use crate::types::token::Token;

#[derive(Debug, Clone)]
pub enum TypeSpec {
    Void(Span),
    Char(Span),
    Short(Span),
    Int(Span),
    Long(Span),
    Signed(Span),
    Unsigned(Span),
    Float(Span),
    Double(Span),
    StructOrUnion(Box<StructOrUnionSpec>),
    Enum(Box<EnumSpec>),
    TypeName(String, Span)
}

impl TypeSpec {
    pub fn make_simple(token: Token) -> ParserNode {
        let span = Span::from_token(&token);
        let result = match token.as_type().unwrap() {
            TokenType::KeywordVoid => TypeSpec::Void(span),
            TokenType::KeywordChar => TypeSpec::Char(span),
            TokenType::KeywordShort => TypeSpec::Short(span),
            TokenType::KeywordInt => TypeSpec::Int(span),
            TokenType::KeywordLong => TypeSpec::Long(span),
            TokenType::KeywordSigned => TypeSpec::Signed(span),
            TokenType::KeywordUnsigned => TypeSpec::Unsigned(span),
            TokenType::KeywordFloat => TypeSpec::Float(span),
            TokenType::KeywordDouble => TypeSpec::Double(span),
            TokenType::TypeName => {
                let typename = token.value.into_string().unwrap();
                TypeSpec::TypeName(typename, span)
            },
            _ => unreachable!()
        };

        result.into()
    }

    pub fn make_struct_or_union(struct_or_union_spec: Box<StructOrUnionSpec>) -> ParserNode {
        TypeSpec::StructOrUnion(struct_or_union_spec).into()
    }

    pub fn make_enum(enum_spec: Box<EnumSpec>) -> ParserNode {
        TypeSpec::Enum(enum_spec).into()
    }
}

impl UnwrapSpan for TypeSpec {
    fn unwrap_span(&self) -> Span {
        match self {
            TypeSpec::Void(x)
            | TypeSpec::Char(x)
            | TypeSpec::Short(x)
            | TypeSpec::Int(x)
            | TypeSpec::Long(x)
            | TypeSpec::Signed(x)
            | TypeSpec::Unsigned(x)
            | TypeSpec::Float(x)
            | TypeSpec::Double(x)
            | TypeSpec::TypeName(_, x) => *x,
            TypeSpec::StructOrUnion(x) => x.span,
            TypeSpec::Enum(x) => x.span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeQual {
    Const(Span),
    Volatile(Span),
}

impl TypeQual {
    pub fn unwrap_span(&self) -> Span {
        match self {
            TypeQual::Const(x)
            | TypeQual::Volatile(x) => *x
        }
    }

    pub fn make(list: Option<Vec<TypeQual>>, token: Token) -> ParserNode {
        let span = Span::from_token(&token);
        let mut list = list.unwrap_or_default();
        let result = match token.as_type().unwrap() {
            TokenType::KeywordConst => TypeQual::Const(span),
            TokenType::KeywordVolatile => TypeQual::Volatile(span),
            _ => unreachable!()
        };

        list.push(result);
        list.into()
    }
}

#[derive(Debug, Clone)]
pub struct DeclSpec {
    pub storage_class: Vec<StorageClass>,
    pub type_quals: Vec<TypeQual>,
    pub type_specs: Vec<TypeSpec>,
    pub span: Span,
}

impl DeclSpec {
    fn new(span: Span) -> Self {
        Self {
            storage_class: Vec::new(),
            type_quals: Vec::new(),
            type_specs: Vec::new(),
            span
        }
    }

    pub fn make_storage(spec: Token, decl_spec: Option<Box<DeclSpec>>) -> ParserNode {
        let span = Span::from_token(&spec);
        let mut decl_spec = decl_spec.unwrap_or_else(|| Box::from(Self::new(span)));
        let spec = match spec.as_type().unwrap() {
            TokenType::KeywordTypedef => StorageClass::Typedef(span),
            TokenType::KeywordExtern => StorageClass::Extern(span),
            TokenType::KeywordStatic => StorageClass::Static(span),
            TokenType::KeywordAuto => StorageClass::Auto(span),
            TokenType::KeywordRegister => StorageClass::Register(span),
            _ => unreachable!()
        };


        decl_spec.span.merge_self(&span);
        decl_spec.storage_class.push(spec);
        decl_spec.into()
    }

    pub fn make_qual(qual: Token, decl_spec: Option<Box<DeclSpec>>) -> ParserNode {
        let span = Span::from_token(&qual);
        let mut decl_spec = decl_spec.unwrap_or_else(|| Box::new(Self::new(span)));

        let qual = match qual.as_type().unwrap() {
            TokenType::KeywordConst => TypeQual::Const(span),
            TokenType::KeywordVolatile => TypeQual::Volatile(span),
            _ => unreachable!()
        };

        decl_spec.span.merge_self(&span);
        decl_spec.type_quals.push(qual);
        decl_spec.into()
    }

    pub fn make_spec(spec: TypeSpec, decl_spec: Option<Box<DeclSpec>>) -> ParserNode {
        let span = spec.unwrap_span();
        let mut decl_spec = decl_spec.unwrap_or_else(|| Box::new(Self::new(span)));

        decl_spec.span.merge_self(&span);
        decl_spec.type_specs.push(spec);
        decl_spec.into()

    }


}

impl UnwrapSpan for DeclSpec {
    fn unwrap_span(&self) -> Span {
        self.span
    }
}

///
/// Declarator是类型声明的前部分:
///     指针部分 + 名称部分 + 函数/数组/嵌套的Declarator
/// 同样作为direct_declarator abstract_declarator的中间结构，因为direct_declarator涉及到ID和递归的declarator
///
/// 一般这个机构如果传递给Sema阶段应该是有名字的
///
#[derive(Debug, Clone)]
pub struct Declarator {
    pub name: Option<String>,
    pub chunks: Vec<DeclaratorChunk>,
    pub span: Span,
}

impl Declarator {
    pub fn new(span: Span) -> Self {
        Self {
            name: None,
            chunks: Vec::new(),
            span
        }
    }

    pub fn make(token: Token) -> ParserNode {
        assert!(token.is(TokenType::Id));

        let span = Span::from_token(&token);
        let name = token.value.into_string().unwrap();
        Self {
            name: Some(name),
            chunks: Vec::new(),
            span,
        }.into()
    }

    pub fn make_pointer(ptr: Option<Vec<DeclaratorChunk>>, declarator: Option<Declarator>) -> Declarator {
        assert!(ptr.is_some() || declarator.is_some());

        let span = match (&ptr, &declarator) {
            (Some(ptr), Some(declarator)) => {
                let first = ptr.first().unwrap().span; // 既然有列表应该是一定有元素的
                first.merge(&declarator.span)
            }
            (Some(ptr), None) => {
                let first = ptr.first().unwrap().span;
                let last = ptr.last().unwrap().span;
                first.merge(&last)
            }
            (None, Some(declarator)) => declarator.span,
            (None, None) => unreachable!()
        };

        match (ptr, declarator) {
            (Some(ptr), Some(mut declarator)) => {
                declarator.chunks.extend(ptr.into_iter());
                declarator
            }
            (Some(ptr), None) => {
                Declarator {
                    name: None,
                    chunks: ptr,
                    span
                }
            }
            (None, Some(declarator)) => {
                declarator
            }
            (None, None) => unreachable!()
        }
    }

    /// 对应 '(' declarator ')'拓展一下span
    pub fn add_span(lparen: Token, mut declarator: Declarator, rparen: Token) -> ParserNode {
        let span = Span::from_tokens(vec![&lparen, &rparen]);
        declarator.span.merge_self(&span);
        declarator.into()
    }
}

/// 组合类型就是Spec + Declarator + 初始化的一个完整的初始化组合定义 
#[derive(Debug, Clone)]
pub struct CompleteDecl {
    pub spec: Box<DeclSpec>,
    pub declarator: Declarator,
    pub init: Option<ast_nodes::Initializer>,     // 可选初始化
    pub span: Span,                     // 覆盖整个 declaration
}

impl CompleteDecl {
    pub fn make(spec: Box<DeclSpec>, declarator: Declarator, init: Option<ast_nodes::Initializer>) -> ParserNode {
        let span = spec.span.merge(&declarator.span);
        let result = Box::new(Self {
            spec,
            declarator,
            init,
            span
        });
        result.into()
    }
    
    
}


///
/// Declarator的部分声明，涵盖：
/// - `pointer`: 对应`Pointer`，quals就是type_qualifier_list
/// - `direct_declarator`的部分
///     - `Array`
///     - `Function`
///
#[derive(Debug, Clone)]
pub struct DeclaratorChunk {
    pub chunk: DeclaratorChunkKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum DeclaratorChunkKind {
    Pointer{ quals: Option<Vec<TypeQual>>, },
    Array {
        size: Option<Box<ast_nodes::Expression>>, // 大小
        asm: ArraySizeModifier, // Array类型(Normal, Static, VLA)
    },
    Function { param_list: Delim<Option<Box<ParamList>>>, },
}

impl DeclaratorChunk {
    pub fn new(chunk: DeclaratorChunkKind, span: Span) -> Self {
        Self { chunk, span }
    }

    /// 构建pointer
    /// # Returns
    /// 注意返回的Chunk的数组
    pub fn make_pointer(token: Token, quals: Option<Vec<TypeQual>>, chunks: Option<Vec<DeclaratorChunk>>) -> ParserNode {
        let span = Span::from_token(&token);
        let mut chunks = chunks.unwrap_or_default();
        let kind = DeclaratorChunkKind::Pointer { quals };
        chunks.push(DeclaratorChunk::new(kind, span));
        chunks.into()
    }

    pub fn make_array(declarator: Option<Declarator>, lbracket: Token, size: Option<Box<ast_nodes::Expression>>, rbracket: Token) -> ParserNode {
        let span = Span::from_tokens(vec![&lbracket, &rbracket]);
        let mut declarator = declarator.unwrap_or_else(|| Declarator::new(span));


        let asm = match &size {
            Some(x) => match x.kind {
                ExpressionKind::Literal(_, _) => ArraySizeModifier::Normal,
                _ => panic!("VLA 未实现")
            }
            None => ArraySizeModifier::Static
        };

        let kind = DeclaratorChunkKind::Array { size, asm };
        declarator.chunks.push(DeclaratorChunk::new(kind, span));
        declarator.into()
    }

    /// ANSI 函数声明
    pub fn make_function(declarator: Option<Declarator>, lparen: Token, param_list: Option<ParamList>, rparen: Token) -> ParserNode {
        let mut declarator = declarator.unwrap_or_else(|| Declarator::new(Span::from_tokens(vec![&lparen, &rparen])));
        let span = declarator.span.merge(&Span::from_token(&rparen));

        let kind = DeclaratorChunkKind::Function {
            param_list: Delim::new(&lparen, param_list.map(Box::new), &rparen),
        };
        declarator.chunks.push(DeclaratorChunk::new(kind, span));
        declarator.into()
    }

    /// K&R 函数声明
    pub fn make_old_function(mut declarator: Declarator, lparen: Token, ident_list: Option<SepList<Token>>, rparen: Token) -> ParserNode {
        let span = declarator.span.merge(&Span::from_token(&rparen));

        let ident_list = ident_list.unwrap_or_default();
        let params: Vec<_> = ident_list.list.into_iter().map(|x| {
            let span = Span::from_token(&x);
            let name = x.value.into_string().unwrap();
            ParamInfo::Ident(name, span)
        }).collect();


        let param_list = ParamList {
            is_variadic: false,
            has_prototype: false,
            params: SepList { list: params, sep: ident_list.sep },
            span,
        };

        let kind = DeclaratorChunkKind::Function {
            param_list: Delim::new(&lparen, Some(Box::new(param_list)), &rparen)
        };

        declarator.chunks.push(DeclaratorChunk::new(kind, span));
        declarator.into()
    }

}

#[derive(Debug, Clone)]
pub enum ArraySizeModifier {
    Normal,     // int a[10];
    Static,     // int a[] = ...;
    VLA         // int a[*];  (VLA without size)，C89不支持
}

#[derive(Debug, Clone)]
pub struct ParamList {
    pub is_variadic: bool,      // 是否是可变参数
    pub has_prototype: bool,    // 是否有原型，就是类型
    pub params: SepList<ParamInfo>, // 参数列表
    pub span: Span
}

impl ParamList {
    pub fn set_variadic(mut param_list: Box<ParamList>, comma: Token) -> ParserNode {
        let span = Span::from_token(&comma);
        param_list.is_variadic = true;
        param_list.params.sep.push(span);
        param_list.into()
    }

    pub fn make_list(param_info: ParamInfo) -> ParserNode {
        let span = param_info.unwrap_span();
        Box::new(Self {
            is_variadic: false,
            has_prototype: true,
            params: SepList::new(param_info),
            span,
        }).into()
    }

    pub fn append_list(mut param_list: Box<ParamList>, comma: Token, param_info: ParamInfo) -> ParserNode {
        let span = Span::from_token(&comma);

        param_list.span.merge_self(&param_info.unwrap_span()); // 增大span
        param_list.params.push_item(param_info);
        param_list.params.push_sep(span);

        param_list.into()
    }
}

/// todo 这里两个大小差别很大，处理
#[derive(Debug, Clone)]
pub enum ParamInfo {
    Ident(String, Span),
    Decl(Box<CompleteDecl>, Span),
}

impl UnwrapSpan for ParamInfo {
    fn unwrap_span(&self) -> Span {
        match self {
            ParamInfo::Ident(_, x)
            | ParamInfo::Decl(_, x) => *x
        }
    }
}









