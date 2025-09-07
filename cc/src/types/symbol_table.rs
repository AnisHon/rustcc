//! date: 2025/08/31
//! author: anishan
//!
//! 符号表以及符号表相关结构
//!

use std::cell::RefCell;
use std::rc::Rc;
use indexmap::IndexMap;
use crate::err::parser_error::{ParserError, ParserResult};

///
/// 符号类型
///
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {

}

///
/// 作用域结构，保存当前作用域符号表以及上一个作用域指针
/// 
pub struct Scope<V> {
    table: IndexMap<String, Rc<V>>,
    parent: Option<Rc<RefCell<Scope<V>>>>,
}

impl<V> Scope<V> {
    fn new(parent: Option<Rc<RefCell<Scope<V>>>>) -> Self {
        Self {
            table: IndexMap::new(),
            parent
        }
    }


    ///
    /// 插入
    ///
    pub fn insert(&mut self, name: &str, v: V) -> ParserResult<()> {
        if self.table.contains_key(name) { 
            return Err(ParserError::new(0, 0, format!("Duplicate Define {}", name).as_str(), ""));
        }
        self.table.insert(name.to_string(), Rc::new(v));
        Ok(())
    }

    ///
    /// 查询
    ///
    pub fn lookup(&self, name: &str) -> Option<Rc<V>> {
        // 先在当前表找
        if let Some(v) = self.table.get(name) {
            return Some(Rc::clone(v));
        }
        // 否则递归父作用域
        if let Some(ref parent) = self.parent {
            return parent.borrow().lookup(name);
        }
        None
    }
}

///
/// 命令式符号表总结构
/// 
/// # Members
/// - 'global': 全局作用域
/// - ‘current’: 当前作用域
/// - 'stack': 总用于栈
pub struct SymbolTable<V = Symbol> {
    global: Rc<RefCell<Scope<V>>>,
    current: Rc<RefCell<Scope<V>>>,
    stack: Vec<Rc<RefCell<Scope<V>>>>
}

impl<V> SymbolTable<V> {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Scope::new(None)));
        Self {
            global: Rc::clone(&global),
            current: Rc::clone(&global),
            stack: vec![global]
        }
    }

    /// 进入作用域，改变当前scope
    pub fn enter_scope(&mut self) {
        let scope = Scope::new(Some(Rc::clone(&self.current)));
        let scope = Rc::new(RefCell::new(scope));
        self.stack.push(Rc::clone(&scope));
        self.current = scope;
    }

    pub fn exit_scope(&mut self) -> Rc<RefCell<Scope<V>>> {
        let last_scope = self.stack.pop().unwrap();
        assert!(last_scope.borrow().parent.is_some()); // 不允许弹出全局作用域
        self.current = Rc::clone(&last_scope);
        last_scope
    }

    ///
    /// 从当前开始插入
    ///
    /// # Error
    /// 当出现重复定义时会抛出一个ParserError，但是行 列 推导式信息需要后续自行填充
    /// 
    pub fn insert(&mut self, name: &str, v: V) -> ParserResult<()> {
        self.current.borrow_mut().insert(name, v)
    }

    ///
    /// 从当前开始查询
    ///
    pub fn lookup(&self, name: &str) -> Option<Rc<V>> {
        self.current.borrow().lookup(name)
    }

    /// 查询全局
    pub fn lookup_global(&self, name: &str) -> Option<Rc<V>> {
        self.global.borrow().lookup(name)
    }
}