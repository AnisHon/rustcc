use crate::generate_from;
use crate::parser::ast::decls::decl::DeclStatus;
use crate::parser::ast::DeclKey;
use crate::parser::common::Ident;
use crate::types::span::Span;

/// 声明相关错误
pub type DeclResult<T> = Result<T, DeclError>;

#[derive(Debug)]
pub enum DeclError {
    DifferentType(DeclDifferentTypeError),
    Redefinition(DeclRedefinitionError),
    UndeclaredIdent(Ident),
    IllegalInitializer(DeclIllegalInitializerError),
}

generate_from!(
    DeclError,
    DeclDifferentTypeError,
    DifferentType,
    DeclRedefinitionError,
    Redefinition,
    DeclIllegalInitializerError,
    IllegalInitializer
);

/// Redefinition / Redeclaration of 'xxx' with xxx different type: 'xxx' vs 'xxx'
#[derive(Debug)]
pub struct DeclDifferentTypeError {
    pub status: DeclStatus,
    pub prev: DeclKey,
    pub curr: Ident, // 出错的时候还没有构建 Decl 对象
}

impl DeclDifferentTypeError {
    pub fn new(status: DeclStatus, prev: DeclKey, curr: Ident) -> Self {
        DeclDifferentTypeError { status, prev, curr }
    }
}

#[derive(Debug)]
pub struct DeclRedefinitionError {
    pub prev: DeclKey,
    pub curr: Ident, // 出错的时候还没有构建 Decl 对象
}

impl DeclRedefinitionError {
    pub fn new(prev: DeclKey, curr: Ident) -> Self {
        DeclRedefinitionError { prev, curr }
    }
}

///
/// - `ident`: 出错的名字
/// - `init_span： `初始化语句的位置
#[derive(Debug)]
pub struct DeclIllegalInitializerError {
    pub ident: Ident,
    pub init_span: Span,
}

impl DeclIllegalInitializerError {
    pub fn new(ident: Ident, init_span: Span) -> Self {
        Self { ident, init_span }
    }
}
