//!
//! struct union enum类型声明相关的AST临时节点定义
//!

use crate::types::lex::token_kind::TokenKind;
use crate::types::ast::ast_nodes::Expression;
use crate::types::ast::decl_info::{DeclSpec, Declarator};
use crate::types::ast::sematic_value::SemanticValue;
use crate::types::span::{Delim, SepList, Span};
use crate::types::lex::token::Token;

pub type StructDeclList = Vec<Box<StructDecl>>;

pub type StructDeclaratorList = SepList<StructDeclarator>;

pub type EnumList = SepList<Enumerator>;

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub spec_qual: DeclSpec,
    pub list: StructDeclaratorList,
    pub span: Span
}

impl StructDecl {
    pub fn make(spec_qual: DeclSpec, list: StructDeclaratorList, semi: Token) -> SemanticValue {
        let span = semi.span.merge(&spec_qual.span);
        Box::new(Self {
            spec_qual,
            list,
            span,
        }).into()
    }

    pub fn make_list(struct_decl: Box<StructDecl>) -> SemanticValue {
        StructDeclList::from([struct_decl]).into()
    }

    pub fn push(mut list: StructDeclList, struct_decl: Box<StructDecl>) -> SemanticValue {
        list.push(struct_decl);
        list.into()
    }
}


#[derive(Debug, Clone)]
pub enum StructKind {
    Struct(Span),
    Union(Span),
}

/// 结构体
#[derive(Debug, Clone)]
pub struct StructUnionSpec {
    pub kind: StructKind,       // struct 或 union
    pub name: Option<String>,   // 可能是匿名 struct
    pub members: Option<Delim<StructDeclList>>,     // 如果有 { ... } 就填，否则 None
    pub span: Span,
}

impl StructUnionSpec {

    /// struct ID
    pub fn make_decl(kind: Token, name: Token) -> SemanticValue {
        let kind_span = kind.span;
        let span = kind_span.merge(&name.span);
        let name = name.value.into_string().unwrap();
        let kind = match kind.kind {
            TokenKind::KeywordStruct => StructKind::Struct(kind_span),
            TokenKind::KeywordUnion => StructKind::Union(kind_span),
            _ => unreachable!()
        };

        let result = Self {
            kind,
            name: Some(name),
            members: None,
            span,
        };
        Box::new(result).into()
    }

    /// struct ID? { ... }
    pub fn make_def(kind: Token, name: Option<Token>, lparen: Token, members: StructDeclList, rparen: Token) -> SemanticValue {
        let kind_span = kind.span;
        let span = kind_span.merge(&rparen.span);

        let name = name.map(|x| x.value.into_string().unwrap());

        let kind = match kind.kind {
            TokenKind::KeywordStruct => StructKind::Struct(kind_span),
            TokenKind::KeywordUnion => StructKind::Union(kind_span),
            _ => unreachable!()
        };

        let result = Self {
            kind,
            name,
            members: Some(Delim::new(&lparen, members, &rparen)),
            span,
        };
        Box::new(result).into()
    }

}

#[derive(Debug, Clone)]
pub struct StructDeclarator {
    pub declarator: Option<Declarator>,
    pub bit_field: Option<Box<Expression>>,
    pub span: Span,
}

impl StructDeclarator {

    pub fn make(declarator: Option<Declarator>, colon: Option<Token>, bit_field: Option<Box<Expression>>) -> SemanticValue {
        assert!(!(declarator.is_none() && bit_field.is_none())); // 不能同时None

        let span = match (&declarator, &bit_field) {
            (Some(declarator), Some(bit_field)) => declarator.span.merge(&bit_field.span),
            (None, Some(bit_field)) => colon.unwrap().span.merge(&bit_field.span),
            (Some(declarator), None) => declarator.span,
            (_, _) => unreachable!()
        };

        Self {
            declarator,
            bit_field,
            span
        }.into()
    }

    pub fn make_list(struct_declarator: StructDeclarator) -> SemanticValue {
        SepList::new(struct_declarator).into()
    }
    
    pub fn push(mut list: StructDeclaratorList, comma: Token, struct_declarator: StructDeclarator) -> SemanticValue {
        list.push(comma.span, struct_declarator);
        list.into()
    }
}



#[derive(Debug, Clone)]
pub struct EnumSpec {
    pub name: Option<String>,
    pub enums: Option<Delim<SepList<Enumerator>>>,
    pub span: Span,
}

impl EnumSpec {
    /// 匿名（也可能有名字）
    pub fn make_anon(keyword_enum: Token, name: Option<Token>, lbrace: Token, enums: SepList<Enumerator>, rbrace: Token) -> SemanticValue {
        let span = Span::from_tokens(vec![&keyword_enum, &rbrace]);

        let name = name.map(|x| x.value.into_string().unwrap());
        let enums = Delim::new(&lbrace, enums, &rbrace);
        let enum_spec = Self {
            name,
            enums: Some(enums),
            span,
        };
        Box::new(enum_spec).into()
    }

    /// 具名枚举
    pub fn make_named(keyword_enum: Token, name: Token) -> SemanticValue {
        let span = Span::from_tokens(vec![&keyword_enum, &name]);
        let name = name.value.into_string().unwrap();
        let enum_spec = Self {
            name: Some(name),
            enums: None,
            span,
        };
        Box::new(enum_spec).into()
    }

}

#[derive(Debug, Clone)]
pub struct Enumerator {
    pub name: String,
    pub value: Option<Box<Expression>>, // 可以有初始化值
    pub span: Span,
}

impl Enumerator {

    pub fn make_list(enumerator: Enumerator) -> SemanticValue {
        let enums = SepList::new(enumerator);
        enums.into()
    }
    pub fn push(enums: EnumList, comma: Token, enumerator: Enumerator) -> SemanticValue {
        let mut enums = enums;
        enums.push_item(enumerator);
        enums.push_sep(comma.span);
        enums.into()
    }

    pub fn make(name: Token, value: Option<Box<Expression>>) -> SemanticValue {
        let span = match &value {
            None => name.span,
            Some(x) => name.span.merge(&x.span),
        };
        let name = name.value.into_string().unwrap();
        
        Self {
            name,
            value,
            span,
        }.into()
    }


}

#[derive(Debug, Clone)]
pub struct CompleteDecl {
    pub decl_spec: DeclSpec,
    pub declarator: Option<Declarator>,
}

impl CompleteDecl {
    pub fn make(decl_spec: DeclSpec, declarator: Option<Declarator>) -> SemanticValue {
        Box::new(Self {
            decl_spec,
            declarator,
        }).into()
    }
}