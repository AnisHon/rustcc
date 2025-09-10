//!
//! 由于Rust的强类型，语义栈必须使用enum包装（也就是C/C++的union包装），才能在语义栈中存储不同类型的 节点。
//!
//! # Contents
//! 该文件定义了ParserNode大enum，并且用 原类型+Noded命名 加以区分
//! 配套定义了From Into(包括Option<T>)，在Parser阶段调用Into自动解开
//!


use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::{DeclSpec, Declarator, DeclaratorChunk, StructOrUnionSpec, TypeQual, TypeSpec};
use crate::types::token::Token;


// =============================
// 宏定义
// =============================
macro_rules! impl_from_variants {
    ($enum:ident { $($variant:ident($ty:ty)),* $(,)? }) => {
        $(
            impl From<$ty> for $enum {
                fn from(v: $ty) -> Self {
                    $enum::$variant(v)
                }
            }

            impl Into<$ty> for $enum {
                fn into(self) -> $ty {
                    match self {
                        $enum::$variant(inner) => inner,
                        _ => panic!("failed to convert {} to {}", stringify!($enum), stringify!($variant)),
                    }
                }
            }

            impl Into<Option<$ty>> for ParserNode {
                fn into(self) -> Option<$ty> {
                    match self {
                        ParserNode::None => None,
                        _ => Some(self.into()),
                    }
                }
            }

        )*
    }
}


#[derive(Debug)]
pub enum ParserNode {
    TranslationUnitNode(TranslationUnit),
    ExternalDeclarationNode(ExternalDeclaration),
    FunctionDefinitionNode(FunctionDefinition),
    DeclarationNode(Declaration),
    TypeNode(Type),
    IntegerSizeNode(IntegerSize),
    FloatSizeNode(FloatSize),
    StorageClassNode(StorageClass),
    QualifiersNode(Qualifiers),
    FieldNode(Field),
    ParameterNode(Parameter),
    InitializerNode(Initializer),
    BlockNode(Block),
    BlockItemNode(BlockItem),
    StatementNode(Statement),
    ExpressionNode(Expression),
    ExpressionListNode(Vec<Expression>),
    ExpressionKindNode(ExpressionKind),
    ConstantNode(Constant),
    UnaryOpNode(UnaryOp),
    BinaryOpNode(BinaryOp),
    AssignOpNode(AssignOp),
    DeclSpecNode(DeclSpec),
    DeclaratorNode(Declarator),
    DeclChunkNode(DeclaratorChunk),
    DeclChunkList(Vec<DeclaratorChunk>),
    TypeSpecNode(TypeSpec),
    TypeQualNode(TypeQual),
    TypeQualListNode(Vec<TypeQual>),
    StructOrUnionSpecNode(StructOrUnionSpec),
    TokenNode(Token),
    TokenListNode(Vec<Token>),
    None,
}



// =============================
//  Into From 实现
// =============================
impl_from_variants!(ParserNode {
    TranslationUnitNode(TranslationUnit),
    ExternalDeclarationNode(ExternalDeclaration),
    FunctionDefinitionNode(FunctionDefinition),
    DeclarationNode(Declaration),
    TypeNode(Type),
    IntegerSizeNode(IntegerSize),
    FloatSizeNode(FloatSize),
    StorageClassNode(StorageClass),
    QualifiersNode(Qualifiers),
    FieldNode(Field),
    ParameterNode(Parameter),
    InitializerNode(Initializer),
    BlockNode(Block),
    BlockItemNode(BlockItem),
    StatementNode(Statement),
    ExpressionNode(Expression),
    ExpressionListNode(Vec<Expression>),
    ExpressionKindNode(ExpressionKind),
    ConstantNode(Constant),
    UnaryOpNode(UnaryOp),
    BinaryOpNode(BinaryOp),
    AssignOpNode(AssignOp),
    DeclaratorNode(Declarator),
    DeclChunkNode(DeclaratorChunk),
    DeclChunkList(Vec<DeclaratorChunk>),
    TypeSpecNode(TypeSpec),
    TypeQualNode(TypeQual),
    TypeQualListNode(Vec<TypeQual>),
    StructOrUnionSpecNode(StructOrUnionSpec),
    TokenNode(Token),
    TokenListNode(Vec<Token>),
});

pub fn make_ident_list(ident_list: Option<Vec<Token>>, ident: Token) -> ParserNode {
    let mut ident_list = ident_list.unwrap_or_else(Vec::new);
    ident_list.push(ident);
    ident_list.into()
}