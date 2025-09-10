//!
//! temp_nodes 是中间节点的类型定义（或者说是未解析节点定义），辅助sema构造
//!

use crate::parser::ast::ast_nodes::{Expression};
use crate::parser::ast::parser_node::ParserNode;
use crate::parser::span::Span;
use crate::parser::ast::ast_nodes;
use crate::types::token::Token;

pub struct DeclSpec {

}

/// declarator 的中间结构
#[derive(Debug, Clone)]
pub struct Declarator {
    pub chunks: Vec<DeclaratorChunk>, // 注意是顺序保存
}

/// type qualifier
#[derive(Debug, Clone)]
pub enum TypeQualifier {
    Const,
    Volatile,
}

#[derive(Debug, Clone)]
pub enum DeclaratorChunk {
    Pointer(Span),
    Array(Expression, Span),
    Function(Vec<ast_nodes::Declaration>, Span),
    OldFunction(Vec<Token>, Span) // 老式函数声明 foo(a, b) int a; int b; {  }
}









