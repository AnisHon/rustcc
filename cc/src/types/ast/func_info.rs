use crate::types::ast::decl_info::{DeclSpec, Declarator};
use crate::types::ast::parser_node::ParserNode;
use crate::types::ast::type_info::CompleteDecl;
use crate::types::lex::token::Token;
use crate::types::span::{SepList, Span, UnwrapSpan};

#[derive(Debug, Clone)]
pub struct ParamList {
    pub is_variadic: bool,
    pub has_prototype: bool,
    pub list: SepList<Box<ParamDecl>>,
    pub span: Span,
}

impl ParamList {
    pub fn make_list(param_decl: Box<ParamDecl>) -> ParserNode {
        let span = param_decl.unwrap_span();
        Self {
            is_variadic: false, 
            has_prototype: param_decl.has_prototype,
            list: SepList::new(param_decl),
            span
        }.into()
    }

    pub fn push(mut param_list: ParamList, comma: Token, param_decl: Box<ParamDecl>) -> ParserNode {
        // 全部都有原型才算原型
        param_list.has_prototype = param_list.has_prototype && param_decl.has_prototype;
        param_list.span.merge_self(&param_decl.unwrap_span());
        param_list.list.push(comma.span, param_decl);
        param_list.into()
    }
    
    pub fn set_variadic(mut param_list: ParamList, comma: Token, ellipsis: Token) -> ParserNode {
        param_list.list.push_sep(comma.span);
        param_list.is_variadic = true;
        param_list.span.merge_self(&ellipsis.span);
        param_list.into()
    }
}


#[derive(Debug, Clone)]
pub struct ParamDecl {
    complete_decl: CompleteDecl,
    has_prototype: bool,
}

impl ParamDecl {
    pub fn make(decl_spec: DeclSpec, declarator: Option<Declarator>, has_prototype: bool) -> ParserNode {
        Box::new(Self {
            complete_decl: CompleteDecl {decl_spec, declarator},
            has_prototype,
        }).into()
    }
    
}

impl UnwrapSpan for ParamDecl {
    fn unwrap_span(&self) -> Span {
        let decl = &self.complete_decl;
        let mut span = decl.decl_spec.unwrap_span();
        if let Some(x) = &decl.declarator {
            span.merge_self(&x.span);
        }
        span
    }
}

