//!
//! temp_nodes 是中间节点的类型定义（或者说是未解析节点定义），辅助sema构造
//!

use crate::lex::lex_yy::TokenType;
use crate::parser::span::Span;
use crate::types::ast::ast_nodes;
use crate::types::ast::ast_nodes::ExpressionKind;
use crate::types::ast::parser_node::ParserNode;
use crate::types::token::Token;

#[derive(Debug, Clone)]
pub struct DeclSpec {
    pub storage_class: Option<Token>,
    pub type_quals: Vec<Token>,
    pub type_specs: Vec<TypeSpec>,
    pub declarator: Vec<DeclaratorChunk>,
    pub span: Span,
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

    pub fn make(token: Token) -> ParserNode {
        let span = Span::from_token(&token);
        let result = match token.as_type().unwrap() {
            TokenType::KeywordConst => TypeQual::Const(span),
            TokenType::KeywordVolatile => TypeQual::Volatile(span),
            _ => unreachable!()
        };
        result.into()
    }
}

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
    StructOrUnion(StructOrUnionSpec),
    Enum(EnumSpec),
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

    pub fn make_struct_or_union(struct_or_union_spec: StructOrUnionSpec) -> ParserNode {
        TypeSpec::StructOrUnion(struct_or_union_spec).into()
    }

    pub fn make_enum(enum_spec: EnumSpec) -> ParserNode {
        TypeSpec::Enum(enum_spec).into()
    }
}

/// 结构体
#[derive(Debug, Clone)]
pub struct StructOrUnionSpec {
    pub kind: StructKind, // struct 或 union
    pub name: Option<String>, // 可能是匿名 struct
    pub members: Option<Vec<StructMember>>, // 如果有 { ... } 就填，否则 None
    pub span: Span,
}

impl StructOrUnionSpec {
    pub fn make(kind: Token, name: Option<Token>, members: Option<Vec<StructMember>>, rparen: Token) -> ParserNode {
        let kind_span = Span::from_token(&rparen);
        let span = kind_span.merge(&Span::from_token(&rparen));
        let name = name.map(|x| x.value.into_string().unwrap());
        let kind = match kind.as_type().unwrap() {
            TokenType::KeywordStruct => StructKind::Struct(kind_span),
            TokenType::KeywordUnion => StructKind::Union(kind_span),
            _ => unreachable!()
        };
        
        Self {
            kind,
            name,
            members,
            span,
        }.into()
    }
}

#[derive(Debug, Clone)]
pub enum StructKind {
    Struct(Span),
    Union(Span),
}

#[derive(Debug, Clone)]
pub struct StructMember {
    pub decl_spec: DeclSpec,
    pub declarators: Vec<DeclSpec>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumSpec {
    pub name: Option<String>,
    pub enumerators: Option<Vec<Enumerator>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Enumerator {
    pub name: String,
    pub value: Option<ast_nodes::Expression>, // 可以有初始化值
    pub span: Span,
}


#[derive(Debug, Clone)]
pub enum DeclaratorChunk {
    Pointer{
        quals: Vec<TypeQual>,
        span: Span, // 只是*符号的span位置
    },
    Array {
        size: Option<ast_nodes::Expression>, // 大小
        asm: ArraySizeModifier, // Array类型(Normal, Static, VLA)
        span: Span,
    },
    Function {
        param_list: Option<ParamList>,
        span: Span
    },
}

impl DeclaratorChunk {

    /// 构建pointer
    /// # Returns
    /// 注意返回的Chunk的数组
    pub fn make_pointer(chunks: Option<Vec<DeclaratorChunk>>, token: Token, type_quals: Option<Vec<TypeQual>>) -> ParserNode {
        let span = Span::from_token(&token);
        let quals = type_quals.unwrap_or_else(Vec::new);
        let mut chunks = chunks.unwrap_or_else(Vec::new);
        chunks.push(Self::Pointer { span, quals });
        chunks.into()
    }

    pub fn make_array(chunks: Option<Vec<DeclaratorChunk>>, lbracket: Token, size: Option<ast_nodes::Expression>, rbracket: Token) -> ParserNode {
        let span = Span::from_tokens(&[lbracket, rbracket]);
        let mut chunks = chunks.unwrap_or_else(Vec::new);
        let asm = match &size {
            Some(x) => match x.kind {
                ExpressionKind::Literal(_, _) => ArraySizeModifier::Normal,
                _ => panic!("VLA 未实现")
            }
            None => ArraySizeModifier::Static
        };

        chunks.push(Self::Array { size, asm, span });
        chunks.into()
    }

    pub fn make_function(chunks: Option<Vec<DeclaratorChunk>>, lparen: Token, param_list: Option<ParamList>, rparen: Token) -> ParserNode {
        let span = Span::from_tokens(&[lparen, rparen]);
        let mut chunks = chunks.unwrap_or_else(Vec::new);

        chunks.push(Self::Function { param_list, span });
        chunks.into()
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
    pub params: Vec<ParamInfo>, // 参数列表
}

#[derive(Debug, Clone)]
pub enum ParamInfo {
    Ident(Token),
    Decl(ast_nodes::Declaration),
}








