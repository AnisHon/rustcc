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

use crate::types::lex::token_kind::TokenKind;
use crate::types::span::{Delim, SepList, Span, UnwrapSpan};
use crate::types::ast::ast_nodes;
use crate::types::ast::ast_nodes::{ExpressionKind, StorageClass};
use crate::types::ast::parser_node::{IdentList, ParserNode};
use crate::types::ast::struct_info::{EnumSpec, StructOrUnionSpec};
use crate::types::lex::token::{Token};

pub type DeclChunkList = Vec<DeclChunk>;
pub type PointerChunkList = Vec<PointerChunk>;

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
    pub fn make(token: Token) -> ParserNode {
        let span = token.span;
        let result = match token.kind {
            TokenKind::KeywordVoid => TypeSpec::Void(span),
            TokenKind::KeywordChar => TypeSpec::Char(span),
            TokenKind::KeywordShort => TypeSpec::Short(span),
            TokenKind::KeywordInt => TypeSpec::Int(span),
            TokenKind::KeywordLong => TypeSpec::Long(span),
            TokenKind::KeywordSigned => TypeSpec::Signed(span),
            TokenKind::KeywordUnsigned => TypeSpec::Unsigned(span),
            TokenKind::KeywordFloat => TypeSpec::Float(span),
            TokenKind::KeywordDouble => TypeSpec::Double(span),
            TokenKind::TypeName => {
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
        let span = token.span;
        let mut list = list.unwrap_or_default();
        let result = match token.kind {
            TokenKind::KeywordConst => TypeQual::Const(span),
            TokenKind::KeywordVolatile => TypeQual::Volatile(span),
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

    pub fn push_storage(spec: Token, decl_spec: Option<DeclSpec>) -> ParserNode {
        let span = spec.span;
        let mut decl_spec = decl_spec.unwrap_or_else(|| Self::new(span));
        let spec = match spec.kind {
            TokenKind::KeywordTypedef => StorageClass::Typedef(span),
            TokenKind::KeywordExtern => StorageClass::Extern(span),
            TokenKind::KeywordStatic => StorageClass::Static(span),
            TokenKind::KeywordAuto => StorageClass::Auto(span),
            TokenKind::KeywordRegister => StorageClass::Register(span),
            _ => unreachable!()
        };


        decl_spec.span.merge_self(&span);
        decl_spec.storage_class.push(spec);
        decl_spec.into()
    }

    pub fn push_qual(qual: Token, decl_spec: Option<DeclSpec>) -> ParserNode {
        let span = qual.span;
        let mut decl_spec = decl_spec.unwrap_or_else(|| Self::new(span));

        let qual = match qual.kind {
            TokenKind::KeywordConst => TypeQual::Const(span),
            TokenKind::KeywordVolatile => TypeQual::Volatile(span),
            _ => unreachable!()
        };

        decl_spec.span.merge_self(&span);
        decl_spec.type_quals.push(qual);
        decl_spec.into()
    }

    pub fn push_spec(spec: TypeSpec, decl_spec: Option<DeclSpec>) -> ParserNode {
        let span = spec.unwrap_span();
        let mut decl_spec = decl_spec.unwrap_or_else(|| Self::new(span));

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
    pub pointer_chunks: PointerChunkList,
    pub decl_chunks: DeclChunkList,
    pub span: Span,
}

impl Declarator {
    pub fn make(pointer_chunks: Option<PointerChunkList>, decl_chunks: DeclChunkList) -> ParserNode {
        let mut pointer_chunks = pointer_chunks.unwrap_or_default();
        pointer_chunks.reverse(); // 翻转
        
        // 要么从pointer中取，没有就从decl中取
        let first = pointer_chunks.first()
            .map(|x| &x.span)
            .unwrap_or_else(|| &decl_chunks.first().unwrap().span);
        
        let last = &decl_chunks.last().unwrap().span;
        let span = first.merge(last);
        
        Self {
            pointer_chunks,
            decl_chunks,
            span
        }.into()
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

#[derive(Debug, Clone)]
pub struct PointerChunk {
    pub quals: Vec<TypeQual>,
    pub span: Span
}

impl PointerChunk {
    pub fn make_list(chunk: PointerChunk) -> ParserNode {
        vec![chunk].into()
    }
    
    /// 构建pointer
    /// # Returns
    /// 注意返回的Chunk的数组
    pub fn make_pointer(token: Token, quals: Option<Vec<TypeQual>>) -> Self {
        let span = token.span;
        let quals = quals.unwrap_or_default();
        Self { quals, span }
    }

    /// 注意这里是push_front是逻辑上的，读取时需要反向使用
    pub fn push_front(chunk: PointerChunk, mut list: PointerChunkList) -> ParserNode {
        list.push(chunk);
        list.into()
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
pub struct DeclChunk {
    pub chunk: DeclChunkKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum DeclChunkKind {
    Array {
        size: Option<Box<ast_nodes::Expression>>, // 大小
        asm: ArraySizeModifier, // Array类型(Normal, Static, VLA)
    },
    Function { param_list: Delim<Option<Box<ParamList>>>, },
    Ident { name: String },
    Paren ( Delim<Declarator> )
}

impl DeclChunk {
    pub fn new(chunk: DeclChunkKind, span: Span) -> Self {
        Self { chunk, span }
    }

    pub fn make_array(lbracket: Token, size: Option<Box<ast_nodes::Expression>>, rbracket: Token) -> Self {
        let span = Span::from_tokens(vec![&lbracket, &rbracket]);
        
        let asm = match &size {
            Some(x) => match x.kind {
                ExpressionKind::Literal(_) => ArraySizeModifier::Normal,
                _ => panic!("VLA 未实现")
            }
            None => ArraySizeModifier::Static
        };

        let kind = DeclChunkKind::Array { size, asm };
        Self::new(kind, span)
    }

    /// ANSI 函数声明
    pub fn make_function(lparen: Token, param_list: Option<ParamList>, rparen: Token) -> Self {
        let span = lparen.span.merge(&rparen.span);
        let kind = DeclChunkKind::Function {
            param_list: Delim::new(&lparen, param_list.map(Box::new), &rparen),
        };
        Self::new(kind, span)
    }

    /// K&R 函数声明
    pub fn make_old_function(lparen: Token, ident_list: Option<IdentList>, rparen: Token) -> Self {
        let span = lparen.span.merge(&rparen.span);
        let ident_list = ident_list.unwrap_or_default();
        let params: Vec<_> = ident_list.list.into_iter().map(|x| {
            let span = x.span;
            let name = x.value.into_string().unwrap();
            ParamInfo::Ident(name, span)
        }).collect();


        let param_list = ParamList {
            is_variadic: false,
            has_prototype: false,
            params: SepList { list: params, sep: ident_list.sep },
            span,
        };

        let kind = DeclChunkKind::Function {
            param_list: Delim::new(&lparen, Some(Box::new(param_list)), &rparen)
        };
        
        Self::new(kind, span)
    }
    
    pub fn make_paren(lparen: Token, declarator: Declarator, rparen: Token) -> Self {
        let span = lparen.span.merge(&rparen.span);
        let delim = Delim::new(&lparen, declarator, &rparen);
        let kind = DeclChunkKind::Paren(delim);
        Self::new(kind, span)
    }
    
    pub fn make_ident(ident: Token) -> Self {
        let span = ident.span;
        let kind = DeclChunkKind::Ident { name: ident.value.into_string().unwrap() };
        Self::new(kind, span)
    }
    
    pub fn make_list(chunk: DeclChunk) -> ParserNode {
        vec![chunk].into()
    }
    
    pub fn push(mut list: DeclChunkList, chunk: DeclChunk) -> ParserNode {
        list.push(chunk);
        list.into()
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
    pub fn set_variadic(mut param_list: ParamList, comma: Token) -> ParserNode {
        let span = comma.span;
        param_list.is_variadic = true;
        param_list.params.sep.push(span);
        param_list.into()
    }

    pub fn make_list(param_info: ParamInfo) -> ParserNode {
        let span = param_info.unwrap_span();
        Self {
            is_variadic: false,
            has_prototype: true,
            params: SepList::new(param_info),
            span,
        }.into()
    }

    pub fn append_list(mut param_list: ParamList, comma: Token, param_info: ParamInfo) -> ParserNode {
        let span = comma.span;

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









