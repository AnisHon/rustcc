//!
//! 由于Rust的强类型，语义栈必须使用enum包装（也就是C/C++的union包装），才能在语义栈中存储不同类型的 节点。
//!
//! # Contents
//! 该文件定义了ParserNode大enum，并且用 原类型+Noded命名 加以区分
//! 配套定义了From Into(包括Option<T>)，在Parser阶段调用Into自动解开
//!

use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::{CompleteDecl, DeclSpec, Declarator, DeclaratorChunk, ParamList, TypeQual, TypeSpec};
use crate::types::ast::struct_info::{EnumSpec, Enumerator, StructDeclarator, StructMember, StructOrUnionSpec};
use crate::types::span::{SepList, Span};
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

            #[allow(clippy::from_over_into)]
            impl Into<$ty> for $enum {
                fn into(self) -> $ty {
                    match self {
                        $enum::$variant(inner) => inner,
                        _ => panic!("failed to convert {} to {}", stringify!($enum), stringify!($variant)),
                    }
                }
            }

            #[allow(clippy::from_over_into)]
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
    FunctionDefinitionNode(Box<FunctionDefinition>),
    DeclarationNode(Box<Declaration>),
    TypeNode(Box<Type>),
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
    DeclSpecNode(Box<DeclSpec>),
    DeclaratorNode(Declarator),
    CompleteDeclNode(CompleteDecl),
    DeclChunkNode(DeclaratorChunk),
    DeclChunkList(Vec<DeclaratorChunk>),
    TypeSpecNode(TypeSpec),
    TypeQualNode(TypeQual),
    TypeQualListNode(Vec<TypeQual>),
    StructOrUnionSpecNode(Box<StructOrUnionSpec>),
    StructMemberNode(Box<StructMember>),
    StructMemberListNode(Vec<StructMember>),
    StructDeclaratorNode(StructDeclarator),
    StructDeclaratorListNode(SepList<StructDeclarator>),
    EnumSpecNode(Box<EnumSpec>),
    EnumListNode(SepList<Enumerator>),
    EnumeratorNode(Enumerator),
    TokenNode(Token),
    TokenListNode(SepList<Token>),
    ParamListNode(ParamList),
    None,
}



// =============================
//  Into From 实现
// =============================
impl_from_variants!(ParserNode {
    TranslationUnitNode(TranslationUnit),
    ExternalDeclarationNode(ExternalDeclaration),
    FunctionDefinitionNode(Box<FunctionDefinition>),
    DeclarationNode(Box<Declaration>),
    TypeNode(Box<Type>),
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
    DeclSpecNode(Box<DeclSpec>),
    DeclaratorNode(Declarator),
    CompleteDeclNode(CompleteDecl),
    DeclChunkNode(DeclaratorChunk),
    DeclChunkList(Vec<DeclaratorChunk>),
    TypeSpecNode(TypeSpec),
    TypeQualNode(TypeQual),
    TypeQualListNode(Vec<TypeQual>),
    StructOrUnionSpecNode(Box<StructOrUnionSpec>),
    StructMemberNode(Box<StructMember>),
    StructMemberListNode(Vec<StructMember>),
    StructDeclaratorNode(StructDeclarator),
    StructDeclaratorListNode(SepList<StructDeclarator>),
    EnumSpecNode(Box<EnumSpec>),
    EnumListNode(SepList<Enumerator>),
    EnumeratorNode(Enumerator),
    TokenNode(Token),
    ParamListNode(ParamList),
    TokenListNode(SepList<Token>),
});

pub fn make_ident_list(ident_list: Option<SepList<Token>>, comma: Option<Token>, ident: Token) -> ParserNode {
    let mut ident_list = ident_list.unwrap_or_default();
    ident_list.push_item(ident);

    if let Some(x) = comma {
        ident_list.push_sep(Span::from_token(&x));
    }
    ident_list.into()
}