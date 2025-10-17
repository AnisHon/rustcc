use std::rc::Rc;
use crate::err::parser_error::ParserResult;
use crate::parser::types::ast::decl::{Decl, DeclKind};
use crate::parser::types::common::Ident;
use crate::parser::types::declarator::Declarator;
use crate::parser::types::sema::decl::decl_context::DeclContextKind;
use crate::parser::types::sema::Sema;
use crate::parser::types::sema::sema_type::Type;

impl Sema {

    pub fn act_on_enum(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        match kind {
            DeclKind::Enum { .. } => {}
            DeclKind::EnumRef { .. } => {}
            _ => unreachable!()
        }
        todo!()
    }
    
    pub fn act_on_field(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        let (decl, colon, bit_field) = kind.as_record_field().unwrap();
        
        todo!()
    }
    
    pub fn act_on_record(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        todo!()
    }

    pub fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        let kind = self.curr_decl.borrow().get_kind();
        let decl = match kind {
            DeclContextKind::Global => self.act_on_global_declarator(declarator)?,
            DeclContextKind::Block => self.act_on_block_declarator(declarator)?,
            DeclContextKind::Record => self.act_on_struct_declarator(declarator)?,
            DeclContextKind::Enum => self.act_on_enum_declarator(declarator)?,
            DeclContextKind::Param => self.act_on_param_declarator(declarator)?,
        };
        self.add_decl(Rc::clone(&decl))?;

        Ok(decl)
    }

    fn add_decl(&mut self, decl: Rc<Decl>) -> ParserResult<()> {
        let mut context = self.curr_decl.borrow_mut();

        // 没有name退出
        let name = match decl.get_name() {
            Some(x) => x.symbol,
            None => return Ok(()),
        };

        // 如果没有声明直接插入，并退出
        let lookup = match context.lookup(name) {
            Some(x) => x,
            None => {
                context.insert(decl)?;
                return Ok(())
            },
        };

        // 出现冲突
        

        Ok(())
    }

    pub fn act_on_var_init(&mut self, kind: &DeclKind) -> ParserResult<Rc<Type>> {
        todo!()
    }

    fn act_on_global_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        todo!()
    }

    fn act_on_block_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        todo!()
    }

    fn act_on_struct_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        todo!()
    }

    fn act_on_enum_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        todo!()
    }

    fn act_on_param_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Decl>> {
        todo!()
    }

}