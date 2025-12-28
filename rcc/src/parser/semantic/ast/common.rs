/// 由于出现了循环引用，所以提取了一个common模块，便于后期继续拓展
use crate::{
    lex::types::{token::Token, token_kind::Keyword},
    types::span::Span,
};
use slotmap::new_key_type;

new_key_type! {
    pub struct ExprKey;
    pub struct TypeKey;
    pub struct DeclKey;
    pub struct StmtKey;

}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum RecordKind {
    Struct,
    Union,
}

#[derive(Debug, Clone)]
pub struct StructOrUnion {
    pub kind: RecordKind,
    pub span: Span,
}

impl StructOrUnion {
    pub fn new(token: Token) -> Self {
        let kind = match token.kind.into_keyword().unwrap() {
            Keyword::Struct => RecordKind::Struct,
            Keyword::Union => RecordKind::Union,
            _ => unreachable!(),
        };
        Self {
            kind,
            span: token.span,
        }
    }
}
