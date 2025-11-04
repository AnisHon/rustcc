use std::rc::Rc;
use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::LiteralKind;
use crate::parser::ast::decl::DeclKind;
use crate::parser::common::Ident;
use crate::parser::semantic::ast::expr::{Expr, ExprKind};
use crate::parser::semantic::sema::Sema;
use crate::parser::semantic::sema::sema_type::{IntegerSize, Qualifier, Type, TypeKind};
use crate::types::span::Span;

impl Sema {

    /// 构建expression 折叠表达式
    pub fn make_expr(&mut self, kind: ExprKind, span: Span) -> ParserResult<Box<Expr>> {
        // todo 需要实现，目前默认unknown
        let ty = self.expr_type(&kind, span)?;

        Ok(Box::new(Expr {
            kind,
            ty,
            span
        }))
    }

    /// 检查和计算当前表达式的类型
    fn expr_type(&mut self, kind: &ExprKind, span: Span) -> ParserResult<Rc<Type>> {
        let ty = match kind {
            ExprKind::DeclRef(x) => self.var_expr_type(x)?,
            ExprKind::Constant(x) => self.type_context.get_constant_type(x),
            ExprKind::Paren { expr, .. } => Rc::clone(&expr.ty),
            ExprKind::ArraySubscript { base, index, .. } => match &base.ty.kind {
                TypeKind::Pointer { elem_ty }
                | TypeKind::Array { elem_ty, .. } => elem_ty.upgrade().unwrap(),
                _ => return Err(ParserError::new(parser_error::ErrorKind::NonSubscripted, span))
            }
            ExprKind::Call { base, params, .. } =>
                self.call_expr_type(&base.ty, &params.exprs, span)?,
            ExprKind::MemberAccess { base, op, field, .. } =>
                self.member_access_expr_type(base, span)?,
            ExprKind::SizeofType { .. } | ExprKind::SizeofExpr { .. } =>
                self.type_context.get_int_type(IntegerSize::Long, false),
            ExprKind::Unary { .. } => { todo!() }
            ExprKind::Binary { .. } => { todo!() }
            ExprKind::Assign { .. } => { todo!() }
            ExprKind::Cast { ty, expr, .. } => self.cast_expr_type(&expr.ty, ty, span)?,
            ExprKind::Ternary { then_expr, else_expr, .. } => { todo!() }
        };

        Ok(ty)
    }

    fn var_expr_type(&mut self, ident: &Ident) -> ParserResult<Rc<Type>> {
        let decl = self.lookup_chain(ident.symbol).ok_or(ParserError::undefined_symbol(ident))?;
        let decl = decl.borrow();
        let ty = match &decl.kind {
            DeclKind::EnumField { .. }
            | DeclKind::VarInit { .. }
            | DeclKind::ParamVar => {
                Rc::clone(&decl.ty)
            }
            DeclKind::Func { .. }
            | DeclKind::FuncRef => {
                let type_kind = TypeKind::Pointer { elem_ty: Rc::downgrade(&decl.ty) };
                let ty = Type::new(Qualifier::default(), type_kind);
                let ty = self.type_context.get_or_set(ty);
                ty
            }
            _ => return Err(ParserError::undefined_symbol(ident))
        };
        Ok(ty)
    }

    fn call_expr_type(&mut self, ty: &Type, call_params: &[Box<Expr>], span: Span) -> ParserResult<Rc<Type>> {
        let ty = match &ty.kind {
            TypeKind::Pointer { elem_ty } => {
                let elem_ty = elem_ty.upgrade().unwrap();
                self.call_expr_type(&elem_ty, call_params, span)?
            }
            TypeKind::Function { ret_ty, params, .. } => {
                let call = call_params.iter()
                    .map(|x| Rc::clone(&x.ty));
                let params = params.iter()
                    .map(|x| x.upgrade().unwrap());

                // 参数不对
                if !call.eq(params) {
                    todo!()
                }
                ret_ty.upgrade().unwrap()
            },
            _ => return Err(ParserError::new(parser_error::ErrorKind::UnCallable, span))
        };
        Ok(ty)
    }

    fn member_access_expr_type(&mut self, expr: &Expr, span: Span) -> ParserResult<Rc<Type>> {
        todo!()
    }

    fn cast_expr_type(&mut self, from: &Type, to: &Type, span: Span) -> ParserResult<Rc<Type>> {
        todo!()
    }

}