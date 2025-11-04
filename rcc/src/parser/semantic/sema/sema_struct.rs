use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::Symbol;
use crate::parser::semantic::sema::decl::decl_context::*;
use crate::parser::semantic::sema::ty::type_context::TypeContext;
use std::cell::RefCell;
use std::rc::Rc;
pub(crate) use crate::parser::ast::decl::DeclRef;

pub struct Sema {
    pub(crate) curr_decl: DeclContextRef,
    pub(crate) type_context: TypeContext,
}
impl Sema {
    pub fn new() -> Self {
        let curr_decl = Rc::new(RefCell::new(CommonDeclContext::new(DeclContextKind::File, None)));
        let type_context = TypeContext::new();
        
        Self {
            curr_decl,
            type_context,
        }
    }
    
    /// 进入decl
    /// # Arguments
    /// - `kind`: 类型
    /// - `context`: 继承上下文
    pub fn enter_decl(&mut self, kind: DeclContextKind) {
        use DeclContextKind::*;
        let curr = Rc::clone(&self.curr_decl);

        let context: DeclContextRef = match kind {
            File | Block => Rc::new(RefCell::new(CommonDeclContext::new(kind, Some(curr)))),
            Record => Rc::new(RefCell::new(RecordDeclContext::new(curr))),
            Enum => Rc::new(RefCell::new(EnumDeclContext::new(curr))),
        };
        self.curr_decl = context;
        // println!("into decl: {:?}", self.curr_decl);
    }

    pub fn exit_decl(&mut self) -> DeclContextRef {
        let curr_decl = Rc::clone(&self.curr_decl);
        let curr_decl_mut = curr_decl.borrow();
        assert_ne!(curr_decl_mut.get_kind(), DeclContextKind::File); // 不允许退出全局
        let parent = curr_decl_mut.get_parent().unwrap();
        drop(curr_decl_mut);
        self.curr_decl = parent;
        curr_decl
    }

    pub fn get_decl_context(&self) -> DeclContextRef {
        Rc::clone(&self.curr_decl)
    }
    
    pub fn insert_decl(&mut self, decl: DeclRef) -> ParserResult<()> {
        self.curr_decl.borrow_mut().insert(decl)
    }

    pub fn insert_parent(&mut self, decl: DeclRef) -> ParserResult<()> {
        let curr_decl = self.curr_decl.borrow_mut();
        let parent_decl = curr_decl.get_parent().unwrap();
        parent_decl.borrow_mut().insert(decl)
    }

    pub fn lookup_chain(&self, symbol: Symbol) -> Option<DeclRef> {
        self.curr_decl.borrow().lookup_chain(symbol)
    }
    
}