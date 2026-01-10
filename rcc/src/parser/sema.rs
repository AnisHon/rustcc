use crate::parser::comp_ctx::CompCtx;

pub mod decl;
pub mod expr;
pub mod scope;
pub mod type_ctx;


pub struct Sema<'a> {
   pub(crate) ctx: &'a mut CompCtx,
}
impl Sema<'_> {
    pub fn new(ctx: &mut CompCtx) -> Sema<'_> {
        Sema { ctx }
    }
}