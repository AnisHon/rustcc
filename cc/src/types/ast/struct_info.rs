//!
//! struct union enum类型声明相关的AST临时节点定义
//!

use crate::types::lex::token_kind::TokenKind;
use crate::types::ast::ast_nodes::Expression;
use crate::types::ast::decl_info::{DeclSpec, Declarator};
use crate::types::ast::parser_node::ParserNode;
use crate::types::span::{Delim, SepList, Span};
use crate::types::lex::token::Token;

#[derive(Debug, Clone)]
pub enum StructKind {
    Struct(Span),
    Union(Span),
}

/// 结构体
#[derive(Debug, Clone)]
pub struct StructOrUnionSpec {
    pub kind: StructKind,       // struct 或 union
    pub name: Option<String>,   // 可能是匿名 struct
    pub members: Option<Delim<Vec<StructMember>>>,     // 如果有 { ... } 就填，否则 None
    pub span: Span,
}

impl StructOrUnionSpec {
    pub fn make(kind: Token, name: Token) -> ParserNode {
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
    pub fn make_decl(kind: Token, name: Option<Token>, lparen: Token, members: Vec<StructMember>, rparen: Token) -> ParserNode {
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
pub struct StructMember {
    pub decl_spec: Box<DeclSpec>,
    pub declarators: SepList<StructDeclarator>,
    pub span: Span,
}

impl StructMember {
    pub fn make_list(list: Option<Vec<StructMember>>, member: StructMember) -> ParserNode {
        let mut list = list.unwrap_or_default();
        list.push(member);
        list.into()
    }
}

#[derive(Debug, Clone)]
pub struct StructDeclarator {
    pub declarator: Option<Box<Declarator>>,
    pub bit_field: Option<Box<Expression>>,
    pub span: Span,
}

impl StructDeclarator {
    pub fn make(declarator: Option<Declarator>,  bit_field: Option<Expression>) -> ParserNode {
        assert!(!(declarator.is_none() && bit_field.is_none())); // 不能同时None，这不对

        let span = match (&declarator, &bit_field) {
            (Some(declarator), Some(bit_field)) => declarator.span.merge(&bit_field.span),
            (None, Some(bit_field)) => bit_field.span,
            (Some(declarator), None) => declarator.span,
            (_, _) => unreachable!()
        };

        Self {
            declarator: declarator.map(Box::new),
            bit_field: bit_field.map(Box::new),
            span
        }.into()
    }

    pub fn make_list(list: Option<SepList<StructDeclarator>>, comma: Token, struct_declarator: StructDeclarator) -> ParserNode {
        let mut list = list.unwrap_or_default();
        list.push_item(struct_declarator);
        list.push_sep(comma.span);
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
    pub fn make_detail(keyword_enum: Token, name: Option<Token>, lbrace: Token, enums: SepList<Enumerator>, rbrace: Token) -> ParserNode {
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

    pub fn make_simple(keyword_enum: Token, name: Token) -> ParserNode {
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

    pub fn make_list(enumerator: Enumerator) -> ParserNode {
        let enums = SepList::new(enumerator);
        enums.into()
    }
    pub fn append_list(enums: SepList<Enumerator>, comma: Token, enumerator: Enumerator) -> ParserNode {
        let mut enums = enums;
        enums.push_item(enumerator);
        enums.push_sep(comma.span);
        enums.into()
    }

    pub fn make(name: Token, value: Option<Expression>) -> ParserNode {
        let span = match &value {
            None => name.span,
            Some(x) => name.span.merge(&x.span),
        };
        let name = name.value.into_string().unwrap();
        
        Self {
            name,
            value: value.map(Box::new),
            span,
        }.into()
    }


}