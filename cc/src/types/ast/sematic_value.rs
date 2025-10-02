//!
//! 由于Rust的强类型，语义栈必须使用enum包装（也就是C/C++的union包装），才能在语义栈中存储不同类型的 节点。
//!
//! # Contents
//! 该文件定义了ParserNode大enum，并且用 原类型+Noded命名 加以区分
//! 配套定义了From Into(包括Option<T>)，在Parser阶段调用Into自动解开
//!

use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::*;
use crate::types::ast::type_info::*;
use crate::types::lex::token::Token;
use crate::types::span::SepList;
use macros::{EnumAutoFrom, EnumAutoInto, EnumAutoIntoOption};
use crate::types::ast::func_info::{ParamDecl, ParamList};
use crate::types::ast::initializer::{InitDeclList, InitDeclarator, InitInfo, InitList};

#[derive(Debug)]
#[derive(EnumAutoInto, EnumAutoFrom, EnumAutoIntoOption, Default)]
pub enum SemanticValue {
    TranslationUnitNode(TranslationUnit),
    ExternalDeclarationNode(ExternalDeclaration),
    FunctionDefinitionNode(Box<FunctionDefinition>),
    DeclNode(Box<Decl>),
    DeclStmtNode(DeclStmt),
    DeclListNode(DeclList),
    TypeNode(Type),
    StorageClassNode(StorageClass),
    QualifiersNode(Qualifiers),
    FieldNode(Box<Field>),
    ParameterNode(Box<Parameter>),
    InitializerNode(Initializer),
    BlockNode(BlockItemList),
    BlockItemNode(BlockItem),
    StatementNode(Statement),
    ExpressionNode(Box<Expression>),
    ExpressionListNode(ExpressionList),
    ConstantNode(Constant),
    DeclSpecNode(DeclSpec),
    DeclaratorNode(Declarator),
    DeclChunkListNode(DeclChunkList),
    PointerChunkListNode(PointerChunkList),
    TypeSpecNode(TypeSpec),
    TypeQualNode(TypeQual),
    TypeQualListNode(Vec<TypeQual>),
    StructOrUnionSpecNode(Box<StructUnionSpec>),
    StructMemberNode(Box<StructDecl>),
    StructMemberNodeList(StructDeclList),
    StructDeclaratorNode(StructDeclarator),
    StructDeclaratorNodeList(StructDeclaratorList),
    EnumSpecNode(Box<EnumSpec>),
    EnumListNode(EnumList),
    EnumeratorNode(Enumerator),
    TypeNameNode(Box<CompleteDecl>),
    TokenNode(Token),
    TokenListNode(IdentList),
    ParamDeclNode(Box<ParamDecl>),
    ParamListNode(ParamList),
    InitInfoNode(InitInfo),
    InitListNode(InitList),
    InitDeclaratorNode(Box<InitDeclarator>),
    InitDeclListNode(InitDeclList),
    #[enum_auto_ignore]
    #[default]
    None
}

pub type IdentList = SepList<Token>;

pub fn make_ident_list(ident: Token) -> SemanticValue {
    IdentList::new(ident).into()
}

pub fn push_ident_list(mut list: IdentList, comma: Token, ident: Token) -> SemanticValue {
    list.push(comma.span, ident);
    list.into()
}