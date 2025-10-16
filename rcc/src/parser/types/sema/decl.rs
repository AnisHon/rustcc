use crate::err::parser_error::ParserResult;
use crate::parser::types::ast::decl::{Decl, EnumField};
use crate::parser::types::decl_spec::{EnumSpec, Enumerator, ParamDecl, ParamList, StructDeclarator, StructSpec};
use crate::parser::types::declarator::{Declarator, InitDeclarator};
use crate::parser::types::sema::sema_context::SemaContext;

impl SemaContext {

    pub fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Decl> {
        todo!()
    }

    pub fn act_on_init_declarator(&mut self, init: InitDeclarator) -> ParserResult<Decl> {
        todo!()
    }
    pub fn act_on_enum_field(&mut self, field: Enumerator) -> ParserResult<EnumField> {
        todo!()
    }

    pub fn act_on_finish_enum(&mut self, spec: EnumSpec) -> ParserResult<Decl> {
        todo!()
    }

    pub fn act_on_struct_declarator(&mut self, declarator: StructDeclarator) -> ParserResult<Decl> {
        todo!()
    }

    pub fn act_on_finish_struct(&mut self, spec: StructSpec) -> ParserResult<Decl> {
        self.exit()?;
        todo!()
    }
    
    pub fn act_on_finish_parameter(&mut self) -> ParserResult<()> {
        todo!()
    }
}