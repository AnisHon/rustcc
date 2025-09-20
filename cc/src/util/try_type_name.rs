use std::cell::RefCell;
use std::rc::Rc;
use crate::types::lex::token_kind::TokenKind;
use crate::types::symbol_table::SymbolTable;
use crate::types::lex::token::Token;

/// 尝试识别TYPE_NAME
pub fn try_type_name<T>(token: &mut Token, symbol_table: &Rc<RefCell<SymbolTable<T>>>) {
    if token.kind != TokenKind::ID { // 非ID类型
        return;
    }

    // ID类型
    let id = token.value.as_string().unwrap().as_str();
    let symbol_table = symbol_table.borrow();
    if symbol_table.lookup(id).is_some() { // 符号表中存在, 修改为TYPE_NAME
        token.kind = TokenKind::TypeName;
    }
}