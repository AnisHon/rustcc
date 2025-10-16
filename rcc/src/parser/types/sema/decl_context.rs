use std::cell::RefCell;
use std::rc::Rc;
use crate::err::parser_error::{ParserResult};
use crate::parser::types::decl_spec::{DeclSpec, TypeQualKind, TypeQualType, TypeSpec};
use crate::parser::types::declarator::Declarator;
use crate::parser::types::sema::sema_context::DeclContextType;
use crate::parser::types::sema::sema_type::{Qualifier, Type};

pub trait DeclContext {
    fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Type>>;

    fn act_on_type_quals(&self, type_qual: &TypeQualType) -> Qualifier {
        Qualifier {
            is_const: type_qual[TypeQualKind::Const as usize].is_some(),
            is_volatile: type_qual[TypeQualKind::Volatile as usize].is_some(),
            is_restrict:type_qual[TypeQualKind::Restrict as usize].is_some()
        }
    }

    fn act_on_decl_spec(&self, decl_spec: Rc<DeclSpec>) -> ParserResult<Type> {
        match (&decl_spec.type_base, &decl_spec.type_size, &decl_spec.signed) {
            (Some(x), Some(a), Some(b)) => {

            }
            (_, _, _) => {}
        }

        let qualifier = self.act_on_type_quals(&decl_spec.type_quals);

        todo!()
    }


    fn enter(&mut self, context_type: DeclContextType) -> ParserResult<()>;

    fn exit(&mut self) -> Rc<RefCell<dyn DeclContext>>;

}

pub struct GlobalDeclContext {}

impl DeclContext for GlobalDeclContext {

    fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Type>> {
        todo!()
    }

    fn enter(&mut self, context_type: DeclContextType) -> ParserResult<()> {
        todo!()
    }

    fn exit(&mut self) -> Rc<RefCell<dyn DeclContext>> {
        todo!()
    }
}

pub struct StructDeclContext {
    curr_decl: Rc<RefCell<dyn DeclContext>>
}

pub struct EnumDeclContext {
    curr_decl: Rc<RefCell<dyn DeclContext>>,
}

impl DeclContext for EnumDeclContext {
    fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Type>> {
        todo!()
    }

    fn enter(&mut self, context_type: DeclContextType) -> ParserResult<()> {
        todo!()
    }

    fn exit(&mut self) -> Rc<RefCell<dyn DeclContext>> {
        todo!()
    }
}

pub struct ParamDeclContext {
    curr_decl: Rc<RefCell<dyn DeclContext>>
}

impl DeclContext for ParamDeclContext {
    fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Type>> {
        todo!()
    }

    fn enter(&mut self, context_type: DeclContextType) -> ParserResult<()> {
        todo!()
    }

    fn exit(&mut self) -> Rc<RefCell<dyn DeclContext>> {
        todo!()
    }
}

pub struct FuncDeclContext {
    curr_decl: Rc<RefCell<dyn DeclContext>>
}

impl DeclContext for FuncDeclContext {
    fn act_on_declarator(&mut self, declarator: Declarator) -> ParserResult<Rc<Type>> {
        todo!()
    }

    fn enter(&mut self, context_type: DeclContextType) -> ParserResult<()> {
        todo!()
    }

    fn exit(&mut self) -> Rc<RefCell<dyn DeclContext>> {
        todo!()
    }
}