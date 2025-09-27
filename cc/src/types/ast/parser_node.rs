//!
//! 由于Rust的强类型，语义栈必须使用enum包装（也就是C/C++的union包装），才能在语义栈中存储不同类型的 节点。
//!
//! # Contents
//! 该文件定义了ParserNode大enum，并且用 原类型+Noded命名 加以区分
//! 配套定义了From Into(包括Option<T>)，在Parser阶段调用Into自动解开
//!

use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::{CompleteDecl, DeclChunkList, DeclSpec, Declarator, ParamList, PointerChunkList, TypeQual, TypeSpec};
use crate::types::ast::struct_info::{EnumList, EnumSpec, Enumerator, StructDeclarator, StructDeclaratorList, StructMember, StructMemberList, StructOrUnionSpec};
use crate::types::lex::token::Token;
use crate::types::span::SepList;
use macros::{EnumAutoFrom, EnumAutoInto, EnumAutoIntoOption};
use crate::types::ast::initializer::{InitDeclList, InitDeclarator, InitInfo, InitList};

#[derive(Debug)]
#[derive(EnumAutoInto, EnumAutoFrom, EnumAutoIntoOption, Default)]
pub enum ParserNode {
    TranslationUnitNode(TranslationUnit),
    ExternalDeclarationNode(ExternalDeclaration),
    FunctionDefinitionNode(Box<FunctionDefinition>),
    DeclNode(Box<Decl>),
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
    CompoundStatementNode(CompoundStatement),
    ExpressionNode(Box<Expression>),
    ExpressionListNode(ExpressionList),
    ConstantNode(Constant),
    DeclSpecNode(DeclSpec),
    DeclaratorNode(Declarator),
    CompleteDeclNode(Box<CompleteDecl>),
    DeclChunkListNode(DeclChunkList),
    PointerChunkListNode(PointerChunkList),
    TypeSpecNode(TypeSpec),
    TypeQualNode(TypeQual),
    TypeQualListNode(Vec<TypeQual>),
    StructOrUnionSpecNode(Box<StructOrUnionSpec>),
    StructMemberNode(StructMember),
    StructMemberNodeList(StructMemberList),
    StructDeclaratorNode(StructDeclarator),
    StructDeclaratorNodeList(StructDeclaratorList),
    EnumSpecNode(Box<EnumSpec>),
    EnumListNode(EnumList),
    EnumeratorNode(Enumerator),
    TokenNode(Token),
    TokenListNode(IdentList),
    ParamListNode(ParamList),
    InitInfoNode(InitInfo),
    InitListNode(InitList),
    InitDeclaratorNode(InitDeclarator),
    InitDeclListNode(InitDeclList),
    #[enum_auto_ignore]
    #[default]
    None
}

pub type IdentList = SepList<Token>;







pub fn make_ident_list(ident_list: Option<SepList<Token>>, comma: Option<Token>, ident: Token) -> ParserNode {
    let mut ident_list = ident_list.unwrap_or_default();
    ident_list.push_item(ident);

    if let Some(x) = comma {
        ident_list.push_sep(x.span);
    }

    ident_list.into()
}