use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::Symbol;
use crate::parser::types::ast::decl::{Decl, DeclKind};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::err::parser_error;

pub type DeclContextRef = Rc<RefCell<dyn DeclContext>>;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DeclContextKind {
    Global,
    Block,
    Record,
    Enum,
    Param,
}

pub trait DeclContext: Debug {
    fn get_decls(&self) -> &FxHashMap<Symbol, Rc<Decl>>;
    // fn get_decls_mut(&mut self) -> &mut FxHashMap<Symbol, Rc<Decl>>;
    fn get_kind(&self) -> DeclContextKind;
    fn get_parent(&self) -> Option<DeclContextRef>;

    fn insert(&mut self, decl: Rc<Decl>) -> ParserResult<()>;

    fn lookup(&self, name: Symbol) -> Option<Rc<Decl>> {
        let decls = self.get_decls();
        decls.get(&name).cloned()
    }

    fn lookup_chain(&self, name: Symbol) -> Option<Rc<Decl>> {
        lookup_chain(name, self.get_decls(), self.get_parent())
    }
}

#[derive(Clone)]
pub struct CommonDeclContext {
    kind: DeclContextKind,
    decls: FxHashMap<Symbol, Rc<Decl>>,
    parent: Option<DeclContextRef>,
}

impl CommonDeclContext {
    pub fn new(kind: DeclContextKind, parent: Option<DeclContextRef>) -> Self {
        Self { kind, decls: FxHashMap::default(), parent }
    }
}

impl DeclContext for CommonDeclContext {

    fn get_decls(&self) -> &FxHashMap<Symbol, Rc<Decl>> {
        &self.decls
    }
    fn get_kind(&self) -> DeclContextKind {
        DeclContextKind::Global
    }

    fn get_parent(&self) -> Option<DeclContextRef> {
        None
    }

    /// 添加decl定义，如果没有名字就忽略
    fn insert(&mut self, decl: Rc<Decl>) -> ParserResult<()> {
        use DeclKind::*;
        let name = match decl.get_name() {
            Some(x) => x.symbol,
            None => return Ok(())
        };

        let lookup = match self.lookup(name) {
            Some(x) => x,
            None => {
                self.decls.insert(name, decl);
                return Ok(())
            }
        };


        match (&lookup.kind, &decl.kind) {
            (Enum { .. }, EnumRef { .. })
            | (EnumRef { .. }, EnumRef { .. })
            | (Record { .. }, RecordRef { .. })
            | (RecordRef { .. }, RecordRef { .. })
            => {}
            (EnumRef { .. }, Enum { .. })
            | (RecordRef { .. }, Record { .. })
            => {
                self.decls.remove(&name);
                self.decls.insert(name, decl);
            }
            (_, _) => {
                let kind = parser_error::ErrorKind::redefinition(name);
                let error = ParserError::new(kind, decl.span);
                return Err(error)
            }
        }
        Ok(())
    }

    fn lookup(&self, name: Symbol) -> Option<Rc<Decl>> {
        self.decls.get(&name).cloned()
    }

    /// global没有parent
    fn lookup_chain(&self, name: Symbol) -> Option<Rc<Decl>> {
        self.lookup(name)
    }
}

impl Debug for CommonDeclContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ decls: {:?} kind: {:?} }}", self.decls, self.kind)
    }
}

#[derive(Clone)]
pub struct RecordDeclContext {
    decls: FxHashMap<Symbol, Rc<Decl>>,
    parent: DeclContextRef,
}

impl RecordDeclContext {
    pub fn new(parent: DeclContextRef) -> Self {
        Self {
            decls: FxHashMap::default(),
            parent,
        }
    }
}

impl Debug for RecordDeclContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ decls: {:?} kind: {:?} }}", self.decls, self.get_kind())
    }
}

impl DeclContext for RecordDeclContext {
    fn get_decls(&self) -> &FxHashMap<Symbol, Rc<Decl>> {
        &self.decls
    }

    fn get_kind(&self) -> DeclContextKind {
        DeclContextKind::Record
    }

    fn get_parent(&self) -> Option<DeclContextRef> {
        Some(Rc::clone(&self.parent))
    }

    fn insert(&mut self, decl: Rc<Decl>) -> ParserResult<()> {

        match &decl.kind {
            DeclKind::RecordRef{ .. }
            | DeclKind::Record{ .. }
            | DeclKind::EnumRef { .. }
            | DeclKind::Enum{ .. } => {
                // 这些声明都放到父作用域
                return self.parent.borrow_mut().insert(decl);
            }
            DeclKind::RecordField { .. } => {} // 交给下面做
            _ => unreachable!()
        }

        let name = match decl.get_name() {
            Some(x) => x.symbol,
            None => return Ok(())
        };

        // 不能出现重定义
        if self.lookup(name).is_some() {
            let kind = parser_error::ErrorKind::redefinition(name);
            let error = ParserError::new(kind, decl.span);
            return Err(error);
        };

        self.decls.insert(name, decl);
        
        Ok(())
    }
    
}

#[derive(Clone)]
pub struct EnumDeclContext {
    decls: FxHashMap<Symbol, Rc<Decl>>,
    parent: DeclContextRef,
}

impl EnumDeclContext {
    pub fn new(parent: DeclContextRef) -> Self {
        Self {
            decls: FxHashMap::default(),
            parent,
        }
    }
}

impl Debug for EnumDeclContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ decls: {:?} kind: {:?} }}", self.decls, self.get_kind())
    }
}

impl DeclContext for EnumDeclContext {
    fn get_decls(&self) -> &FxHashMap<Symbol, Rc<Decl>> {
        &self.decls
    }

    fn get_kind(&self) -> DeclContextKind {
        DeclContextKind::Enum
    }

    fn get_parent(&self) -> Option<DeclContextRef> {
        Some(Rc::clone(&self.parent))
    }

    fn insert(&mut self, decl: Rc<Decl>) -> ParserResult<()> {
        let name = decl.get_name().unwrap().symbol; // 一定有名字
        
        // 不能出现重定义
        if self.lookup(name).is_some() {
            let kind = parser_error::ErrorKind::redefinition(name);
            let error = ParserError::new(kind, decl.span);
            return Err(error);
        };

        self.decls.insert(name, decl);
        Ok(())
    }
}

fn lookup_chain(name: Symbol, decls: &FxHashMap<Symbol, Rc<Decl>>, parent: Option<DeclContextRef>) -> Option<Rc<Decl>> {
    // 查找当前表
    if let Some(v) = decls.get(&name) {
        return Some(Rc::clone(v));
    }
    // 递归父作用域
    if let Some(ref parent) = parent {
        return parent.borrow().lookup_chain(name);
    }
    None
}

