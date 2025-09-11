//!
//! temp_nodes 是中间节点的类型定义（或者说是未解析节点定义），辅助sema构造
//!

use crate::parser::span::Span;
use crate::parser::ast::ast_nodes;
use crate::types::token::Token;



pub struct DeclSpec {
    ty: ast_nodes::Type, // ast节点，可以表示Union Struct Enum
    declarator: DeclaratorInfo,
}

/// declarator 的中间结构
#[derive(Debug, Clone)]
pub struct DeclaratorInfo {
    pub chunks: Vec<DeclaratorChunk>, // 注意顺序
}

/// type qualifier
#[derive(Debug, Clone)]
pub enum QualifierInfo {
    Const,
    Volatile,
}

#[derive(Debug, Clone)]
pub enum DeclaratorChunk {
    Decl(DeclaratorInfo),
    Pointer(Vec<QualifierInfo>),
    Array {
        size: Option<ast_nodes::Expression>, // 大小
        asm: ArraySizeModifier, // Array类型(Normal, Static, VLA)
        span: Span
    },
    Function {
        is_variadic: bool,      // 是否是可变参数
        has_prototype: bool,    // 是否有原型，就是类型
        params: Vec<ParamInfo>, // 参数列表
        span: Span
    },
}

#[derive(Debug, Clone)]
pub enum ArraySizeModifier {
    Normal,     // int a[10];
    Static,     // int a[] = ...;
    VLA         // int a[*];  (VLA without size)，C89不支持
}

#[derive(Debug, Clone)]
pub enum ParamInfo {
    Ident(Token),
    Decl(ast_nodes::Declaration),
}








