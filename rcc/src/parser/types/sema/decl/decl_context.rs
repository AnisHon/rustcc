use crate::lex::types::token_kind::Symbol;
use crate::parser::types::ast::decl::Decl;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub type DeclContextRef = Rc<RefCell<DeclContext>>;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DeclContextKind {
    Global,
    Block,
    Struct,
    Param,
    Enum,
}

#[derive(Debug, Clone)]
pub struct DeclContext {
    pub kind: DeclContextKind,
    pub decls: FxHashMap<Symbol, Rc<Decl>>,
    pub parent: Option<DeclContextRef>,
}

impl DeclContext {
    pub fn new(kind: DeclContextKind, parent: Option<DeclContextRef>) -> Self {
        Self {
            kind,
            decls: FxHashMap::default(),
            parent,
        }
    }

    /// 添加decl定义，如果没有名字就忽略
    pub fn insert(&mut self, decl: Rc<Decl>) {
        if let Some(name) = decl.get_name() {
            assert!(self.decls.contains_key(&name.symbol));
            self.decls.insert(name.symbol, Rc::clone(&decl));
        }
    }

    pub fn lookup(&self, name: Symbol) -> Option<Rc<Decl>> {
        self.decls.get(&name).cloned()
    }
    
    pub fn lookup_chain(&self, name: Symbol) -> Option<Rc<Decl>> {
        // 查找当前表
        if let Some(v) = self.decls.get(&name) {
            return Some(Rc::clone(v));
        }
        // 递归父作用域
        if let Some(ref parent) = self.parent {
            return parent.borrow().lookup_chain(name);
        }
        None
    }
    
}