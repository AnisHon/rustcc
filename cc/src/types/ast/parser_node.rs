//!
//! 由于Rust的强类型，语义栈必须使用enum包装（也就是C/C++的union包装），才能在语义栈中存储不同类型的 节点。
//!
//! # Contents
//! 该文件定义了ParserNode大enum，并且用 原类型+Noded命名 加以区分
//! 配套定义了From Into(包括Option<T>)，在Parser阶段调用Into自动解开
//!

use macros::{EnumAutoFrom, EnumAutoInto, EnumAutoIntoOption};
use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::{CompleteDecl, DeclSpec, Declarator, DeclaratorChunk, ParamList, TypeQual, TypeSpec};
use crate::types::ast::struct_info::{EnumSpec, Enumerator, StructDeclarator, StructMember, StructOrUnionSpec};
use crate::types::span::{SepList, Span};
use crate::types::lex::token::Token;


// =============================
// 宏定义
// =============================

#[derive(Debug)]
#[derive(EnumAutoInto, EnumAutoFrom, Default)]
pub enum ParserNode {
    TranslationUnitNode(TranslationUnit),
    ExternalDeclarationNode(Box<ExternalDeclaration>),
    FunctionDefinitionNode(Box<FunctionDefinition>),
    DeclarationNode(Box<Declaration>),
    TypeNode(Box<Type>),
    StorageClassNode(StorageClass),
    QualifiersNode(Qualifiers),
    FieldNode(Box<Field>),
    ParameterNode(Box<Parameter>),
    InitializerNode(Initializer),
    BlockNode(Block),
    BlockItemNode(BlockItem),
    StatementNode(Statement),
    ExpressionNode(Box<Expression>),
    ExpressionListNode(Vec<Expression>),
    ConstantNode(Constant),
    DeclSpecNode(Box<DeclSpec>),
    DeclaratorNode(Declarator),
    CompleteDeclNode(Box<CompleteDecl>),
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
    ParamListNode(Box<ParamList>),
    #[enum_auto_ignore]
    #[default]
    None
}

pub fn make_ident_list(ident_list: Option<SepList<Token>>, comma: Option<Token>, ident: Token) -> ParserNode {
    let mut ident_list = ident_list.unwrap_or_default();
    ident_list.push_item(ident);

    if let Some(x) = comma {
        ident_list.push_sep(Span::from_token(&x));
    }

    ident_list.into()
}