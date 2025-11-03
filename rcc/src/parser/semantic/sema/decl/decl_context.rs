use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::Symbol;
use crate::parser::semantic::ast::decl::DeclKind;
use crate::parser::semantic::sema::sema_struct::DeclRef;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub type DeclContextRef = Rc<RefCell<dyn DeclContext>>;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DeclContextKind {
    File,
    Block,
    Record,
    Enum,
}

pub trait DeclContext: Debug {
    fn get_decls(&self) -> &FxHashMap<Symbol, DeclRef>;
    // fn get_decls_mut(&mut self) -> &mut FxHashMap<Symbol, DeclKey>;
    fn get_kind(&self) -> DeclContextKind;
    fn get_parent(&self) -> Option<DeclContextRef>;

    fn insert(&mut self, decl: DeclRef) -> ParserResult<()>;

    fn lookup(&self, name: Symbol) -> Option<DeclRef> {
        let decls = self.get_decls();
        decls.get(&name).cloned()
    }

    fn lookup_chain(&self, name: Symbol) -> Option<DeclRef> {
        lookup_chain(name, self.get_decls(), self.get_parent())
    }
}

#[derive(Clone)]
pub struct CommonDeclContext {
    kind: DeclContextKind,
    decls: FxHashMap<Symbol, DeclRef>,
    parent: Option<DeclContextRef>,
}

impl CommonDeclContext {
    ///
    /// # Arguments
    /// - `kind`: 类型
    /// - `parent`: 父声明
    /// - `decls`: 定义
    ///
    pub fn new(
        kind: DeclContextKind,
        parent: Option<DeclContextRef>,
    ) -> Self {
        let decls = FxHashMap::default();
        Self { kind, decls, parent }
    }
}

impl DeclContext for CommonDeclContext {

    fn get_decls(&self) -> &FxHashMap<Symbol, DeclRef> {
        &self.decls
    }
    fn get_kind(&self) -> DeclContextKind {
        self.kind
    }

    fn get_parent(&self) -> Option<DeclContextRef> {
        self.parent.clone()
    }

    /// 添加decl定义，如果没有名字就忽略
    fn insert(&mut self, decl_ref: DeclRef) -> ParserResult<()> {
        let decl = decl_ref.borrow();
        use DeclKind::*;
        let name = match decl.get_name() {
            Some(x) => x.symbol,
            None => return Ok(())
        };

        let lookup_ref = match self.lookup(name) {
            Some(x) => x,
            None => {
                drop(decl);
                self.decls.insert(name, decl_ref);
                return Ok(())
            }
        };

        let mut lookup = lookup_ref.borrow_mut();

        // println!("{:?} \n\n {:?}\n---------\n", lookup.kind, decl.kind);
        match (&lookup.kind, &decl.kind) {
            (Enum { .. }, EnumRef { .. })
            | (EnumRef { .. }, EnumRef { .. })
            | (Record { .. }, RecordRef { .. })
            | (RecordRef { .. }, RecordRef { .. })
            => {}
            (EnumRef { .. }, Enum { .. })
            | (RecordRef { .. }, Record { .. })
            => {
                // 覆盖原来的声明
                *lookup = decl.clone();
            }
            (FuncRef { .. }, FuncRef { .. }, ) => {
                // 如果类型不同出错，
                if lookup.ty.ne(&decl.ty) {
                    // todo
                }
                // 覆盖原来的声明
                *lookup = decl.clone();
            }
            (_, _) => { // 重定义出错
                let kind = parser_error::ErrorKind::redefinition(name);
                let error = ParserError::new(kind, decl.span);
                return Err(error)
            }
        }
        Ok(())
    }

    fn lookup(&self, name: Symbol) -> Option<DeclRef> {
        self.decls.get(&name).cloned()
    }
}

impl Debug for CommonDeclContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ decls: {:?} kind: {:?} }}", self.decls, self.kind)
    }
}



#[derive(Clone)]
pub struct RecordDeclContext {
    decls: FxHashMap<Symbol, DeclRef>,
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
    fn get_decls(&self) -> &FxHashMap<Symbol, DeclRef> {
        &self.decls
    }

    fn get_kind(&self) -> DeclContextKind {
        DeclContextKind::Record
    }

    fn get_parent(&self) -> Option<DeclContextRef> {
        Some(Rc::clone(&self.parent))
    }

    fn insert(&mut self, decl_ref: DeclRef) -> ParserResult<()> {
        let decl = decl_ref.borrow();
        match &decl.kind {
            DeclKind::EnumField { .. }
            | DeclKind::RecordRef{ .. }
            | DeclKind::Record{ .. }
            | DeclKind::EnumRef { .. }
            | DeclKind::Enum{ .. } => {
                // 这些声明都放到父作用域
                drop(decl);
                return self.parent.borrow_mut().insert(decl_ref);
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

        drop(decl);
        self.decls.insert(name, decl_ref);
        
        Ok(())
    }
    
}

#[derive(Clone)]
pub struct EnumDeclContext {
    decls: FxHashMap<Symbol, DeclRef>,
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
    fn get_decls(&self) -> &FxHashMap<Symbol, DeclRef> {
        &self.decls
    }

    fn get_kind(&self) -> DeclContextKind {
        DeclContextKind::Enum
    }

    fn get_parent(&self) -> Option<DeclContextRef> {
        Some(Rc::clone(&self.parent))
    }

    fn insert(&mut self, decl: DeclRef) -> ParserResult<()> {
        // 直接插到父作用域
        self.parent.borrow_mut().insert(decl)
    }
}

// #[derive(Clone)]
// pub struct ParamDeclContext {
//     decls: FxHashMap<Symbol, DeclRef>,
//     parent: DeclContextRef,
// }
//
// impl ParamDeclContext {
//     pub fn new(parent: DeclContextRef) -> Self {
//         Self {
//             decls: FxHashMap::default(),
//             parent
//         }
//     }
// }

// impl Debug for ParamDeclContext {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{{ decls: {:?} kind: {:?} }}", self.decls, self.get_kind())
//     }
// }

// impl DeclContext for ParamDeclContext {
//     fn get_decls(&self) -> &FxHashMap<Symbol, DeclRef> {
//         &self.decls
//     }
//
//     fn get_kind(&self) -> DeclContextKind {
//         DeclContextKind::Enum
//     }
//
//     fn get_parent(&self) -> Option<DeclContextRef> {
//         Some(Rc::clone(&self.parent))
//     }
//
//     fn insert(&mut self, decl_ref: DeclRef) -> ParserResult<()> {
//         let decl = decl_ref.borrow();
//         if !decl.kind.is_param_var() {
//             // 不是函数参数声明不能
//             todo!()
//         }
//
//         match &decl.name {
//             None => {}
//             Some(x) => {
//                 let name = x.symbol;
//                 drop(decl);
//                 self.decls.insert(name, decl_ref);
//             }
//         }
//         Ok(())
//     }
// }

fn lookup_chain(name: Symbol, decls: &FxHashMap<Symbol, DeclRef>, parent: Option<DeclContextRef>) -> Option<DeclRef> {
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

