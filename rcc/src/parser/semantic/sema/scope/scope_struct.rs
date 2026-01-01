use crate::err::scope_error::ScopeError;
use crate::lex::types::token_kind::Symbol;
use crate::parser::ast::{DeclKey, StmtKey, TypeKey};
use rustc_hash::FxHashMap;

/// Scope 当前类型
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum ScopeKind {
    File,
    Function,
    Block,
    ParamList,
    Record,
    // Enum,
}

/// 不知道叫什么名字好
pub trait TScope {
    type Item;
    fn lookup(&mut self, symbol: Symbol) -> Option<&mut Self::Item>;

    fn insert(&mut self, symbol: Symbol, t: Self::Item) -> Result<(), ScopeError>;
}

/// Scope 的符号对象
/// - `name`: 符号名
/// - `decls`: 声明 decl 对象
/// - `def`: 定义 decl 对象
/// - `ty`: decl 的类型，无论声明还是定义类型应该是一致的，用于快速判断
#[derive(Debug)]
pub struct ScopeSymbol {
    pub name: Symbol,
    pub decls: Vec<DeclKey>, // 大多数情况下声明不会超过一个，所以用 Vec 可能有些重
    pub def: Option<DeclKey>,
    pub ty: TypeKey,
}

// 优先取 def，然后是 decls
impl ScopeSymbol {
    pub fn get_decl(&self) -> DeclKey {
        // 都已经放到表里了，两个同时是 none 是逻辑错误，大概率是 lookup or insert 导致的
        self.def
            .unwrap_or_else(|| *self.decls.last().expect("decls and def are both none"))
    }
}

/// Label 符号对象， 处理 label 和其他的不太一样，所以这里要单独设计一个符号
/// - `name`: 符号名称
/// - `stmt`: 跳转 label 声明，当函数退出后需要自行检查是否有 “悬挂” 的 goto 引用
/// - `pending_gotos`: 用于定义 “悬挂” 的 goto 引用，label 定义后进行回填
#[derive(Debug)]
pub struct LabelSymbol {
    pub name: Symbol,
    pub stmt: Option<StmtKey>,
    pub pending_gotos: Vec<StmtKey>,
}

/// Record Member 的成员不会存在 definition
#[derive(Debug)]
pub struct MemberSymbol {
    pub name: Symbol,
    pub decl: DeclKey,
    pub ty: Option<TypeKey>,
}

/// 用于 label 的 Scope 对象

#[derive(Debug, Default)]
pub struct LabelScope {
    pub sym_ht: FxHashMap<Symbol, LabelSymbol>,
}

impl LabelScope {
    /// 一定要手动插入声明或者定义，否则可能出错
    pub(crate) fn lookup_or_insert(&mut self, symbol: Symbol) -> &mut LabelSymbol {
        let symbol = self
            .sym_ht
            .entry(symbol)
            .or_insert_with_key(|x| LabelSymbol {
                name: symbol,
                stmt: None,
                pending_gotos: Vec::new(),
            });

        symbol
    }
}

// 表示一个作用域
/// - `sym_ht`: symbol hash table
#[derive(Debug, Default)]
pub struct Scope {
    pub sym_ht: FxHashMap<Symbol, ScopeSymbol>,
}

impl Scope {
    /// 一定要手动插入声明或者定义，否则可能出错
    ///
    /// # Arguments
    /// - `symbol`:
    /// - `curr`:
    /// - `ty`:
    pub(crate) fn lookup_or_insert(&mut self, symbol: Symbol, ty: TypeKey) -> &mut ScopeSymbol {
        let symbol = self
            .sym_ht
            .entry(symbol)
            .or_insert_with_key(|x| ScopeSymbol {
                name: *x,
                decls: Vec::new(),
                def: None,
                ty,
            });

        symbol
    }

    fn lookup(&mut self, symbol: Symbol) -> Option<&mut ScopeSymbol> {
        return self.sym_ht.get_mut(&symbol);
    }
}

/// 用于 member 的 Scope 对象
#[derive(Debug, Default)]
pub struct MemberScope {
    pub sym_ht: FxHashMap<Symbol, MemberSymbol>,
}
