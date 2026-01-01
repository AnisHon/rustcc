use crate::err::scope_error::ScopeError;
use crate::err::scope_error::ScopeErrorKind;
use crate::err::scope_error::ScopeResult;
use crate::err::scope_error::ScopeSource;
use crate::lex::types::token_kind::Symbol;
use crate::parser::common::Ident;
use crate::parser::semantic::sema::scope::scope_struct::{
    LabelScope, LabelSymbol, MemberScope, MemberSymbol, Scope, ScopeKind, ScopeSymbol,
};
use std::collections::hash_map::Entry;

// macro_rules! scope_enter_pop {
//     ($enter:ident, $pop:ident, $field:ident) => {
//         pub fn $enter(&mut self) {
//             self.$field.push(Scope::default());
//         }

//         pub fn $pop(&mut self) -> Scope {
//             debug_assert!(!self.$field.is_empty());
//             match self.$field.pop() {
//                 Some(x) => x,
//                 None => unreachable!("`{}` can't be empty", stringify!($field)),
//             }
//         }
//     };
// }

macro_rules! scope_lookup {
    ($lookup:ident, $lookup_local:ident, $entry_local:ident, $must_lookup:ident, $insert:ident, $field:ident, $return:ty, $scope_source:ident) => {
        pub fn $lookup<'a>(&'a self, ident: &Ident) -> Option<&'a $return> {
            let sym = ident.symbol;
            let stack = &self.$field;
            for scope in stack.iter().rev() {
                let key = scope.sym_ht.get(&sym);
                if key.is_some() {
                    return key;
                }
            }

            None
        }

        pub fn $must_lookup(&mut self, ident: Ident) -> ScopeResult<&$return> {
            let kind = ScopeErrorKind::Undefined;
            self.$lookup(&ident).ok_or(ScopeError {
                kind,
                name: ident.symbol.get(),
                scope: ScopeSource::$scope_source,
                span: ident.span,
            })
        }

        pub fn $lookup_local(&self, sym: Symbol) -> Option<&$return> {
            let scope = match self.$field.last() {
                Some(x) => x,
                None => unreachable!("`{}` can't be empty", stringify!($field)),
            };
            scope.sym_ht.get(&sym)
        }

        pub fn $entry_local(&mut self, sym: Symbol) -> Entry<Symbol, $return> {
            let scope = match self.$field.last_mut() {
                Some(x) => x,
                None => unreachable!("`{}` can't be empty", stringify!($field)),
            };
            scope.sym_ht.entry(sym)
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
                self.$field.push(Default::default());
            )*
        }

        pub fn $leave(&mut self) {
            debug_assert!(!self.kinds.is_empty());
            debug_assert_eq!(*self.kinds.last().unwrap(), ScopeKind::$kind);
            self.kinds.pop();
            $(
                debug_assert!(!self.$field.is_empty());
                self.$field.pop();
            )*
        }
    };
}

/// Scope 管理器
/// - `tags`: record enum
/// - `members`: record fields，可能用不到待定
/// - `labels`: goto label
/// - `idents`: typedef var
/// - `ctx`: current scope context
pub struct ScopeMgr {
    pub(crate) tags: Vec<Scope>,
    pub(crate) members: Vec<MemberScope>,
    pub(crate) labels: Vec<LabelScope>,
    pub(crate) idents: Vec<Scope>,
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

    pub fn pop_tag(&mut self) -> Scope {
        debug_assert!(!self.tags.is_empty());
        match self.tags.pop() {
            Some(x) => x,
            None => unreachable!("`{}` can't be empty", stringify!(tags)),
        }
    }

    // scope_enter_pop!(enter_tag, pop_tag, tags);
    // scope_enter_pop!(enter_member, pop_member, members);
    // scope_enter_pop!(enter_label, pop_label, labels);
    // scope_enter_pop!(enter_ident, pop_ident, idents);

    // 生成 一堆 lookup
    scope_lookup!(
        lookup_tag,
        lookup_local_tag,
        entry_local_tag,
        must_lookup_tag,
        insert_tag,
        tags,
        ScopeSymbol,
        Tag
    );

    scope_lookup!(
        lookup_member,
        lookup_local_member,
        entry_local_member,
        must_lookup_member,
        insert_member,
        members,
        MemberSymbol,
        Member
    );

    scope_lookup!(
        lookup_label,
        lookup_local_label,
        entry_local_label,
        must_lookup_label,
        insert_label,
        labels,
        LabelSymbol,
        Label
    );

    scope_lookup!(
        lookup_ident,
        lookup_local_ident,
        entry_local_ident,
        must_lookup_ident,
        insert_ident,
        idents,
        ScopeSymbol,
        Ident
    );

    scope_enter_leave!(enter_global, leave_global, File, tags, idents);

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

    scope_enter_leave!(enter_record, leave_record, Record, members);

    pub fn get_kind(&self) -> ScopeKind {
        debug_assert!(!self.kinds.is_empty());
        self.kinds.last().expect("impossible").clone()
    }

    pub fn insert_tag(&mut self, name: Ident, symbol: ScopeSymbol) -> ScopeResult<()> {
        // 符号重复定义
        if let Some(prev) = self.lookup_local_ident(name.symbol) {
            let kind = ScopeErrorKind::Redefined {
                prev: prev.get_decl(),
            };
            let error = ScopeError {
                kind,
                name: name.symbol.get(),
                scope: ScopeSource::Tag,
                span: name.span,
            };
            return Err(error);
        }

        // 当前的Scope
        let scope = self.tags.last_mut().expect("should not be empty");
        scope.sym_ht.insert(name.symbol, symbol);

        Ok(())
    }
}
