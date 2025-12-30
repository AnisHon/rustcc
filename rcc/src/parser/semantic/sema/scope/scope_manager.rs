use crate::err::scope_error::ScopeError;
use crate::lex::types::token_kind::Symbol;
use crate::parser::ast::DeclKey;
use crate::parser::semantic::sema::scope::scope_struct::{Scope, ScopeKind};

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

/// 生成 pop_xxx leave_xxx 函数
/// - `enter`: enter 函数名
/// - `leave`: leave 函数名
/// - `kind`: scope kind
/// - `fields`: pop 和 push 的内容
macro_rules! scope_enter_leave {
    ($enter:ident, $leave:ident, $kind:ident, $($field:ident),+) => {
        pub fn $enter(&mut self) {
            self.kinds.push(ScopeKind::$kind);
            $(
                self.$field.push(Scope::default());
            )*
        }

        pub fn $leave(&mut self) {
            assert!(!self.kinds.is_empty());
            assert_eq!(self.kinds.last().unwrap(), ScopeKind::$kind);
            self.ctx.pop();
            $(
                assert!(!self.$field.is_empty());
                self.$field.pop();
            )*
        }
    };
}

/// Scope 管理器
/// - `tags`: record enum
/// - `members`: record fields
/// - `labels`: goto label
/// - `idents`: typedef var
/// - `ctx`: current scope context
pub struct ScopeMgr {
    tags: Vec<Scope>,
    members: Vec<Scope>,
    labels: Vec<Scope>,
    idents: Vec<Scope>,
    kinds: Vec<ScopeKind>,
}

impl ScopeMgr {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            members: Vec::new(),
            labels: Vec::new(),
            idents: Vec::new(),
            kinds: Vec::new(),
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

    pub fn pop_tag(&mut self) -> Scope {
        assert!(!self.tags.is_empty());
        match self.tags.pop() {
            Some(x) => x,
            None => unreachable!("`{}` can't be empty", stringify!(tags)),
        }
    }

    // scope_enter_pop!(enter_tag, pop_tag, tags);
    // scope_enter_pop!(enter_member, pop_member, members);
    // scope_enter_pop!(enter_label, pop_label, labels);
    // scope_enter_pop!(enter_ident, pop_ident, idents);

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

    scope_enter_leave!(enter_global, leave_global, Global, tags, idents);

    scope_enter_leave!(
        enter_function,
        leave_function,
        Function,
        tags,
        idents,
        labels
    );

    scope_enter_leave!(enter_block, leave_block, Block, tags, idents);

    scope_enter_leave!(enter_param, leave_param, ParamList, idents);

    scope_enter_leave!(enter_record, leave_record, Record, tags);

    pub fn get_kind(&self) -> ScopeKind {
        assert!(!self.kinds.is_empty());
        self.kinds.last().expect("impossible").clone()
    }
}
