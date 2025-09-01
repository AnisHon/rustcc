use std::cell::RefCell;
use std::rc::Rc;
use crate::lex::lex_yy::TokenType;
use crate::types::symbol_table::SymbolTable;
use crate::types::token::Token;

/// 尝试识别TYPE_NAME
pub fn try_type_name(token: &mut Token, symbol_table: &Rc<RefCell<SymbolTable<()>>>) {
    if !token.is(TokenType::Id) { // 非ID类型
        return;
    }

    // ID类型
    let id = token.value.as_string().unwrap().as_str();
    let symbol_table = symbol_table.borrow();
    if symbol_table.lookup(id).is_some() { // 符号表中存在, 修改为TYPE_NAME
        token.typ = TokenType::TypeName as usize;
    }
}