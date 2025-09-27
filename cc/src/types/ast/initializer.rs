use crate::types::ast::ast_nodes;
use crate::types::ast::decl_info::Declarator;
use crate::types::ast::parser_node::ParserNode;
use crate::types::lex::token::Token;
use crate::types::span::{SepList, Span};

pub type InitList = SepList<InitInfo>;

#[derive(Debug, Clone)]
pub enum InitInfo {
    Expr(Box<ast_nodes::Expression>),
    InitList{
        lbrace: Span,
        list: InitList,
        rbrace: Span,
    },
}

impl InitInfo {
    pub fn make_expr(expr: Box<ast_nodes::Expression>) -> ParserNode {
        InitInfo::Expr(expr).into()
    }

    pub fn make_init_list(lbrace: Token, mut list: InitList, comma: Option<Token>, rbrace: Token) -> ParserNode {
        if let Some(comma) = comma {
            list.push_sep(comma.span);

        }
        InitInfo::InitList {
            lbrace: lbrace.span,
            list,
            rbrace: rbrace.span
        }.into()
    }


    pub fn make_list(init: InitInfo) -> ParserNode {
        InitList::new(init).into()
    }

    pub fn push(mut list: InitList, comma: Token, init: InitInfo) -> ParserNode {
        list.push(comma.span, init);
        list.into()
    }



}

pub type InitDeclList = SepList<InitDeclarator>;

#[derive(Debug, Clone)]
pub struct InitDeclarator {
    pub decl: Declarator,
    pub eq: Option<Span>,
    pub init: Option<InitInfo>,
}

impl InitDeclarator {
    
    pub fn make(decl: Declarator, eq: Option<Token>, init: Option<InitInfo>) -> ParserNode {
        Self {
            decl,
            eq: eq.map(|x| x.span),
            init,
        }.into()
    }
    
    pub fn make_list(init: InitDeclarator) -> ParserNode {
        InitDeclList::new(init).into()
    }
    
    pub fn push(mut list: InitDeclList, comma: Token, init: InitDeclarator) -> ParserNode {
        list.push(comma.span, init);
        list.into()
    }
    

}