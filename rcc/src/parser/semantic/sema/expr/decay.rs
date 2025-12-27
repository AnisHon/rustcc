// use crate::parser::{ast::{TypeKey, exprs::{BinOpKind, ExprKind, UnaryOpKind}}, comp_ctx::CompCtx};

// /// 处理类型衰变，也可以直接放到type里计算
// pub(crate) fn decay_expr(ctx: &mut CompCtx, kind: &mut ExprKind) {
//     use ExprKind::*;
//     let ty = match kind {
//         // 不类型衰变
//         DeclRef(_)
//         | Literal(_)
//         | ArraySubscript { .. }
//         | Call { .. }
//         | MemberAccess { base, field, kind, .. }
//         | SizeofType { .. } 
//         | ExprKind::SizeofExpr { .. } => {} 


//         Unary { op, rhs } => {
//             let rhs = ctx.get_expr_mut(*rhs);
//             match op.kind {
//                 // 不衰变
//                 UnaryOpKind::AddrOf | UnaryOpKind::PostInc
//                 | UnaryOpKind::PostDec | UnaryOpKind::PreInc 
//                 | UnaryOpKind::PreDec => return,
//                 // array function 衰变
//                 UnaryOpKind::Deref | UnaryOpKind::Plus
//                 | UnaryOpKind::Minus | UnaryOpKind::Not 
//                 | UnaryOpKind::BitNot => {
//                     rhs.ty = dacay_if_function_or_array(ctx, rhs.ty);
//                 }
//             }
//         }
//         Binary { op, lhs, rhs } => {
//             use BinOpKind::*;
//             let lhs = ctx.get_expr_mut(*lhs);
//             let rhs = ctx.get_expr_mut(*rhs); 
//             match op.kind {
//                  Lt | Gt | Eq | Ne | Le | Ge | Plus | Minus | And | Or | Comma => {
//                     rhs.ty = dacay_if_function_or_array(ctx, rhs.ty);
//                     lhs.ty = dacay_if_function_or_array(ctx, lhs.ty);
//                  },
//                  _ => {}
//             }
//         }
//         Assign { lhs, op, rhs } => {
//             let lhs = ctx.get_expr_mut(*lhs);
//             let rhs = ctx.get_expr_mut(*rhs);
//         }
//         Cast { expr, .. } => {
//             let from = ctx.get_expr(*expr).ty;
//         }
//         Ternary { cond, then_expr, else_expr, .. } => {
//             let cond = ctx.get_expr(*cond).ty;
//             let then_expr = ctx.get_expr(*then_expr).ty;
//             let else_expr = ctx.get_expr(*else_expr).ty;
//         }
//     };
// }


// /// 将 function 衰变为指针，只有sizeof不会衰变指针
// fn decay_function(ctx: &mut CompCtx, func_ty: TypeKey) -> TypeKey {
//     assert!(ctx.type_ctx.get_type(func_ty).kind.is_function(), "not a function");
//     ctx.type_ctx.get_pointer(func_ty)
// }

// // 将 array 衰变为指针
// fn decay_array(ctx: &mut CompCtx, arr_ty: TypeKey) -> TypeKey {
//     let ty = ctx.type_ctx.get_type(arr_ty);
//     let (elem_ty, _) = ty.kind.as_array().expect("not an array type");
//     ctx.type_ctx.get_pointer(*elem_ty)
// }

// // 如果是 function 则衰变，否则返回自身
// fn decay_if_function(ctx: &mut CompCtx, key: TypeKey) -> TypeKey {
//     let ty = ctx.type_ctx.get_type(key);
//     if ty.kind.is_function() {
//         decay_function(ctx, key)
//     } else {
//         key
//     }
// }

// fn dacay_if_function_or_array(ctx: &mut CompCtx, key: TypeKey) -> TypeKey {
//     let ty = ctx.type_ctx.get_type(key);
//     match &ty.kind {
//         TypeKind::Function{ .. } => decay_function(ctx, key),
//         TypeKind::Array{ .. } => decay_array(ctx, key),
//         _ => key,
//     }
// }