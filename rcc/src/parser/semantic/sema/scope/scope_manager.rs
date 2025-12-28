use crate::err::scope_error::ScopeError;
use crate::parser::ast::DeclKey;
use crate::parser::semantic::sema::scope::scope_struct::Scope;
use crate::{
    lex::types::token_kind::Symbol
};

macro_rules! scope_enter_pop {
    ($enter:ident, $pop:ident, $field:ident) => {
        pub fn $enter(&mut self) {
            self.$field.push(Scope::default());
        }

        pub fn $pop(&mut self) -> Scope {
            assert!(!self.$field.is_empty());
            match self.$field.pop() {
                Some(x) => x,
                None => unreachable!("`{}` can't be empty", stringify!($field)),
            }
        }
    };
}

macro_rules! scope_lookup {
    ($lookup:ident, $lookup_local:ident, $must_lookup:ident, $field:ident) => {
        pub fn $lookup(&self, sym: Symbol) -> Option<DeclKey> {
            Self::lookup_recursive(&self.$field, &sym)
        }

        pub fn $must_lookup(&self, sym: Symbol) -> Result<DeclKey, ScopeError> {
            self.$lookup(sym)
                .ok_or(ScopeError::Undefined { field: sym.get() })
        }

        pub fn $lookup_local(&self, sym: Symbol) -> Option<DeclKey> {
            let scope = match self.$field.last() {
                Some(x) => x,
                None => unreachable!("`{}` can't be empty", stringify!($field)),
            };
            scope.sym_ht.get(&sym).copied()
        }
    };
}

macro_rules! scope_insert {
    ($insert:ident, $field:ident) => {
        pub fn $insert(&mut self, name: Symbol, key: DeclKey) -> Result<(), ScopeError> {
            // 符号重复定义
            if let Some(prev) = self.lookup_local_ident(name) {
                return Err(ScopeError::Redefined {
                    field: name.get(),
                    prev,
                });
            }

            // 当前的Scope
            let scope = self.$field.last_mut().expect("should not be empty");
            let sym_ht = &mut scope.sym_ht;

            sym_ht.insert(name, key);

            return Ok(());
        }
    };
}

pub struct ScopeMgr {
    tags: Vec<Scope>,
    members: Vec<Scope>,
    labels: Vec<Scope>,
    idents: Vec<Scope>,
}

impl ScopeMgr {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            members: Vec::new(),
            labels: Vec::new(),
            idents: Vec::new(),
        }
    }

    /// 递归查找
    fn lookup_recursive(stack: &[Scope], sym: &Symbol) -> Option<DeclKey> {
        assert!(!stack.is_empty());
        for scope in stack.iter().rev() {
            let key = scope.sym_ht.get(sym);
            if key.is_some() {
                return key.copied();
            }
        }
        None
    }

    scope_enter_pop!(enter_tag, pop_tag, tags);
    scope_enter_pop!(enter_member, pop_member, members);
    scope_enter_pop!(enter_label, pop_label, labels);
    scope_enter_pop!(enter_ident, pop_ident, idents);

    scope_lookup!(lookup_tag, lookup_local_tag, must_lookup_tag, tags);
    scope_lookup!(
        lookup_member,
        lookup_local_member,
        must_lookup_member,
        members
    );
    scope_lookup!(lookup_label, lookup_local_label, must_lookup_label, labels);
    scope_lookup!(lookup_ident, lookup_local_ident, must_lookup_ident, idents);

    scope_insert!(insert_label, labels);
    scope_insert!(insert_member, members);
    scope_insert!(insert_ident, idents);
    scope_insert!(insert_tag, tags);
}