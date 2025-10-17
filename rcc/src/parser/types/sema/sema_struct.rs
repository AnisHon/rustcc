use crate::parser::types::sema::decl::decl_context::{DeclContext, DeclContextKind, DeclContextRef};
use std::cell::RefCell;
use std::rc::Rc;


pub struct Sema {
    curr_decl: DeclContextRef
}
impl Sema {
    pub fn new() -> Self {
        let curr_decl = Rc::new(RefCell::new(DeclContext::new(DeclContextKind::Global, None)));
        Self {
            curr_decl
        }
    }
    
    pub fn decl_enter(&mut self, kind: DeclContextKind) {
        self.curr_decl = Rc::new(RefCell::new(DeclContext::new(kind, Some(Rc::clone(&self.curr_decl)))));
    }
    
    pub fn decl_exit(&mut self) -> DeclContextRef {
        let curr_decl = Rc::clone(&self.curr_decl);
        let curr_decl_mut = curr_decl.borrow();
        assert_ne!(curr_decl_mut.kind, DeclContextKind::Global); // 不允许退出全局
        let parent = Rc::clone(curr_decl_mut.parent.as_ref().unwrap());
        drop(curr_decl_mut);
        self.curr_decl = parent;
        curr_decl
    }
    
}