/// 表达式类型推导
use crate::{err::parser_error::{self, ParserError, ParserResult}, lex::types::token_kind::{LiteralKind, Symbol}, parser::{ast::{ExprKey, TypeKey, exprs::{AssignOpKind, BinOpKind, ExprKind, MemberAccessKind, UnaryOpKind}, types::{IntegerSize, Qualifier, Type, TypeKind}}, common::Ident, comp_ctx::CompCtx, semantic::sema::expr::value_type::ValueType}, types::span::Span};


/// 检查和计算当前表达式的类型，要做 表达式类型 decay
pub(crate) fn expr_type(ctx: &mut CompCtx, kind: &ExprKind, span: Span) -> ParserResult<TypeKey> {
    use ExprKind::*;
    let ty = match kind {
        DeclRef(x) => var_expr_type(ctx, x)?,
        Literal(x) => literal_expr_type(ctx, x),
        // ExprKind::Paren { expr, .. } => Rc::clone(&expr.ty),
        ArraySubscript { base, index, .. } => 
            array_subscript_type(ctx, *base, *index)?,
        Call { base, params, .. } => {
            let base = ctx.get_expr(*base);
            call_expr_type(ctx, base.ty, &params.exprs, span)?
        }
        MemberAccess { base, field, kind, .. } => {
            let base = ctx.get_expr(*base);
            member_access_expr_type(ctx, base.ty, kind.clone(), *field, span)?
        }
        SizeofType { .. } | ExprKind::SizeofExpr { .. } => {
            type_context.get_int_type(IntegerSize::Long, false)
        }
        Unary { op, rhs } => {
            let rhs = ctx.get_expr(*rhs);  
            let valuety = ValueType::value_type(rhs)
            unary_type(ctx, op.kind.clone(), rhs.ty, valuety, span)?
        }
        Binary { op, lhs, rhs } => {
            let lhs = ctx.get_expr(*lhs);
            let rhs = ctx.get_expr(*rhs); 
            binary_type(ctx, lhs, op.kind.clone(), rhs.ty.clone(), span)?
        }
        Assign { lhs, op, rhs } => {
            let lhs = ctx.get_expr(*lhs);
            let rhs = ctx.get_expr(*rhs);
            assign_type(ctx, lhs, op.kind.clone(), rhs, span)?
        }
        Cast { ty, expr, .. } => {
            let from = ctx.get_expr(*expr).ty;
            let to = *ty;
            cast_expr_type(ctx, from, to, span)?
        }
        Ternary { cond, then_expr, else_expr, .. } => {
            let cond = ctx.get_expr(*cond).ty;
            let then_expr = ctx.get_expr(*then_expr).ty;
            let else_expr = ctx.get_expr(*else_expr).ty;
            ternary_expr_type(ctx, cond, else_expr, then_expr, span)?
        }
    };

    Ok(ty)
}

/// 获取literal的类型
fn literal_expr_type(ctx: &mut CompCtx, literal: &LiteralKind) -> TypeKey {
    use LiteralKind::*;
    match literal {
        Integer { suffix, .. } => 
            ctx.type_ctx.get_by_int_sfx(suffix.clone()),
        Float { suffix, ..} => 
            ctx.type_ctx.get_by_float_sfx(suffix.clone()),
        Char { .. } => ctx.type_ctx.get_char(),
        String { value } => // 假定已经处理转义
            ctx.type_ctx.get_string_type(value.get().len() as u64),
    }
}

/// 获取变量表达式类型
fn var_expr_type(ctx: &CompCtx, ident: &Ident) -> ParserResult<TypeKey> {
    // 去符号表lookup ident，获取类型
    let decl = ctx.scope_mgr.must_lookup_ident(ident.symbol)
        .map_err(|err| ParserError::from_scope_error(err, ident.span))?;

    let decl = ctx.get_decl(decl);
    let ty = decl.ty;
    Ok(ty)
}

/// 数组访问表达式类型，不衰变函数
fn array_subscript_type(ctx: &CompCtx, base: ExprKey, index: ExprKey) -> ParserResult<TypeKey> {
    let base = ctx.get_expr(base);
    let index = ctx.get_expr(index);
    let index_ty = ctx.type_ctx.get_type(index.ty);
    let base_ty = ctx.type_ctx.get_type(base.ty);

    // 非整数索引
    if !index_ty.kind.is_integer() {
        let err = ParserError::not_int_constant(index.span);
        return Err(err)
    }

    let ty = match &base_ty.kind {
        TypeKind::Pointer { elem_ty }
        | TypeKind::Array { elem_ty, .. } => *elem_ty,
        // 不是可索引的类型
        _ => return Err(ParserError::non_subscripted(base.span))
    };

    Ok(ty)
}

/// 函数调用类型
fn call_expr_type(ctx: &CompCtx, ty: TypeKey, call_params: &[ExprKey], span: Span) -> ParserResult<TypeKey> {
    let ty = ctx.type_ctx.get_type(ty);
    let ty = match &ty.kind {
        TypeKind::Pointer { elem_ty } => {
            call_expr_type(ctx, *elem_ty, call_params, span)?
        }
        TypeKind::Function { ret_ty, params, .. } => {
            let call = call_params.iter().copied()
                .map(|x| ctx.get_expr(x))
                .map(|x| x.ty);

            // 检查参数
            if !call.eq(params.iter().cloned()) {
                todo!()
            }
            *ret_ty
        },
        _ => return Err(ParserError::new(parser_error::ErrorKind::UnCallable, span))
    };
    Ok(ty)
}

fn member_access_expr_type(ctx: &CompCtx, ty_key: TypeKey, op: MemberAccessKind, field: Symbol, span: Span) -> ParserResult<TypeKey> {
    let ty = ctx.type_ctx.get_type(ty_key);
    match op {
        MemberAccessKind::Arrow => {
            let elem_ty=  match ty.kind.as_pointer() {
                None => todo!("不是指针错误"),
                Some(x) => *x,
            };
            member_access_expr_type(ctx, elem_ty, MemberAccessKind::Dot, field, span)
        }
        MemberAccessKind::Dot => {
            let (name, fields) = match &ty.kind {
                TypeKind::Struct { name, fields, .. }
                | TypeKind::Union { name, fields, .. } => {
                    (name, fields)
                }
                _ => {
                    let kind = parser_error::ErrorKind::NotStructOrUnion { ty: ty_key };
                    let error = ParserError::new(kind, span);
                    return Err(error)
                },
            };
            fields.iter()
                .find(|x| x.name.as_ref().map(|x| x.symbol == field).unwrap_or_default())
                .map(|x| x.ty)
                .ok_or_else(|| { // 找不到出错
                    let field = field.get().to_string();
                    let ty = name.as_ref().map(|x| x.symbol.get().to_string()).unwrap_or_default();
                    let kind = parser_error::ErrorKind::NoMember { field, ty };
                    ParserError::new(kind, span)
                })

        }
    }
}

fn cast_expr_type(ctx: &CompCtx, from_key: TypeKey, to_key: TypeKey, span: Span) -> ParserResult<TypeKey> {
    let from = ctx.type_ctx.get_type(from_key);
    let to = ctx.type_ctx.get_type(to_key);
    if cast_compatible(from, to) {
        Ok(to_key)
    } else {
        Err(ParserError::error("Wrong Cast".to_owned(), span))
    }
}

/// 三元运算符类型
fn ternary_expr_type(
    ctx: &CompCtx,
    cond_key: TypeKey,
    a_key: TypeKey,
    b_key: TypeKey,
    span: Span,
) -> ParserResult<TypeKey>
{
    use TypeKind::*;

    // a 和 b 完全相同 —— 直接返回
    if a_key == b_key {
        return Ok(a_key);
    }

    let cond = ctx.type_ctx.get_type(cond_key);
    let a = ctx.type_ctx.get_type(a_key);
    let b = ctx.type_ctx.get_type(b_key);
        



    // cond 必须是可转换为 bool/整数的类型
    match &cond.kind {
        Integer { .. } | Floating { .. } | Pointer { .. } => {}
        Array { .. } | Function { .. } => {
            todo!("无意义的代码，这些地址永远为true，所以发一个warning")
        }
        Void => unreachable!("got void expression, weird"), // 这个理论上不会出现
        Record { .. } | Enum { .. } | Unknown =>  // 不是 scalar 类型出错
            return Err(ParserError::not_scalar_type(cond_key, span)),
    }

    

    // 都是算术类型 → usual arithmetic conversion
    if a.is_arithmetic() && b.is_arithmetic() {
        return arith_promote(ctx, a_key, b_key, span);
    }

    // ============================================================
    // 4. 
    // ============================================================
    match (&a.kind, &b.kind) {
        (Record { id: id1, .. }, Record { id: id2, .. }) if id1 == id2 =>  // record id必须一致
            return Ok(a_key), 
        (Pointer { elem_ty: ae, .. }, Pointer { elem_ty: be, .. }) => { // 指针特殊处理
            if ae != be {
                todo!("指针不一致，报错，gcc/clang都只是警告") 
            }
            return Ok(a_key)
        }
        (Void, Void) => return Ok(a_key),  // 理论上不会出现

        _ => {}
    }

    // // 指针处理 cast_compatible
    // if let Pointer { elem_ty: ae } = &a.kind {
    //     if let Pointer { elem_ty: be } = &b.kind {
    //         // 指向同元素类型 → 返回该指针
    //         if ae == be {
    //             return Ok(a_key);
    //         }

    //         // void* + T* → void*
    //         if a.is_void_ptr(ctx) {
    //             return Ok(a_key);
    //         }
    //         if b.is_void_ptr(ctx) {
    //             return Ok(b_key);
    //         }

    //         // 不兼容
    //         todo!("三元运算符的两个指针类型不兼容")
    //     }
    // }

    // 尝试转换为二者其一
    if cast_compatible(&a, &b) {
        return Ok(b_key)
    }
    if cast_compatible(&b, &a) {
        return Ok(a_key);
    }

    // 都不行出错
    Err(ParserError::incompatable(a_key, b_key, span))
}

/// 算数时类型提升，只支持 Float 和 Integer
fn arith_promote(
    ctx: &CompCtx,
    a_key: TypeKey,
    b_key: TypeKey,
    span: Span
) -> ParserResult<TypeKey> {
    use TypeKind::*;

    let a = ctx.type_ctx.get_type(a_key);
    let b = ctx.type_ctx.get_type(b_key);
    assert!(a.kind.is_integer() || a.kind.is_floating(), "expact integer, floating");
    assert!(b.kind.is_integer() || b.kind.is_floating(), "expact interger, floating");

    // 1. 两者都是浮点，返回较宽浮点
    if let (Floating { size: sa }, Floating { size: sb }) = (&a.kind, &b.kind) {
        let ty = match sa.rank() > sb.rank() {
            true => a_key,
            false => b_key,
        };
        return Ok(ty);
    }

   
    match (&a.kind, &b.kind) {

         // 2. 浮点 + 整数 → 浮点类型，整数先转浮点
        (Floating { .. }, Integer { .. }) => return Ok(a_key),
        (Integer { .. }, Floating { size: fs }) => return Ok(b_key), 


        // 3. 两者都是整数 → integer promotion + rank 比较
        (
            Integer { is_signed: sa, size: ra }, 
            Integer { is_signed: sb, size: rb }
        ) => { 
             // (1) 做 integer promotion
            let sza = int_promote(*ra);
            let szb = int_promote(*rb);

            // let pa = ctx.get_int_type(sza, *sa);
            // let pb = ctx.get_int_type(szb, *sb);

            // (2) 比较 rank
            if ra.rank() != rb.rank() {
                let ty = match ra.rank() > rb.rank() {
                    true => ctx.type_ctx.get_int_type(sza, *sa),
                    false => ctx.type_ctx.get_int_type(szb, *sb)
                };
                return Ok(ty)
            }

            // (3) rank 相同，处理 signed/unsigned 混合情况
            match (sa, sb) {
                // 都是无符号
                (false, false) => {
                    return Ok(ctx.type_ctx.get_int_type(sza, *sa))
                }
                // 都是有符号
                (true, true) => {
                    return Ok(ctx.type_ctx.get_int_type(sza, *sa))
                }
                // 一个 signed 一个 unsigned
                (true, false) | (false, true) => {
                    // 规则：
                    // 若 unsigned rank >= signed rank → unsigned
                    // 否则 → signed 的类型（若能表示所有 unsigned）
                    // C 的完整逻辑：
                    //
                    // 1. 如果 unsigned 的 rank >= signed 的 rank：
                    //        转成 unsigned 同 rank
                    // 2. 否则：
                    //    如果 signed 类型能表示 unsigned 的所有值：
                    //         转成 signed 类型
                    //    否则：
                    //         转成 unsigned same-rank

                    let unsigned_side = if *sa == false { (false, ra) } else { (false, rb) };
                    let signed_side   = if *sa == true  { (true,  ra) } else { (true,  rb) };

                    // rank 比较
                    if unsigned_side.1.rank() >= signed_side.1.rank() {
                        return Ok(ctx.type_ctx.get_int_type(*unsigned_side.1, false))
                    } else {
                        // 构造 signed
                        return Ok(ctx.type_ctx.get_int_type(*signed_side.1, true))
                    }
                }
            }
        }

        _ => {}
    }

    // 
    // if let (Integer { is_signed: sa, size: ra },
    //     Integer { is_signed: sb, size: rb }) = (&a.kind, &b.kind)
    // {
    //     // (1) 做 integer promotion
    //     let sza = int_promote(*ra);
    //     let szb = int_promote(*rb);

    //     // let pa = ctx.get_int_type(sza, *sa);
    //     // let pb = ctx.get_int_type(szb, *sb);

    //     // (2) 比较 rank
    //     if ra.rank() != rb.rank() {
    //         let ty = match ra.rank() > rb.rank() {
    //             true => ctx.type_ctx.get_int_type(sza, *sa),
    //             false => ctx.type_ctx.get_int_type(szb, *sb)
    //         };
    //         return Ok(ty)
    //     }

    //     // (3) rank 相同，处理 signed/unsigned 混合情况
    //     match (sa, sb) {
    //         // 都是无符号
    //         (false, false) => {
    //             return Ok(ctx.type_ctx.get_int_type(sza, *sa))
    //         }
    //         // 都是有符号
    //         (true, true) => {
    //             return Ok(ctx.type_ctx.get_int_type(sza, *sa))
    //         }
    //         // 一个 signed 一个 unsigned
    //         (true, false) | (false, true) => {
    //             // 规则：
    //             // 若 unsigned rank >= signed rank → unsigned
    //             // 否则 → signed 的类型（若能表示所有 unsigned）
    //             // C 的完整逻辑：
    //             //
    //             // 1. 如果 unsigned 的 rank >= signed 的 rank：
    //             //        转成 unsigned 同 rank
    //             // 2. 否则：
    //             //    如果 signed 类型能表示 unsigned 的所有值：
    //             //         转成 signed 类型
    //             //    否则：
    //             //         转成 unsigned same-rank

    //             let unsigned_side = if *sa == false { (false, ra) } else { (false, rb) };
    //             let signed_side   = if *sa == true  { (true,  ra) } else { (true,  rb) };

    //             // rank 比较
    //             if unsigned_side.1.rank() >= signed_side.1.rank() {
    //                 return Ok(type_context.get_int_type(*unsigned_side.1, false))
    //             } else {
    //                 // 构造 signed
    //                 return Ok(type_context.get_int_type(*signed_side.1, true))
    //             }
    //         }
    //     }
    // }

    Err(ParserError::incompatable(a_key, b_key, span))
}

/// 算数类型提升
fn binary_type(
    ctx: &CompCtx,
    a_key: TypeKey,
    op: BinOpKind,
    b_key: TypeKey,
    span: Span
) -> ParserResult<TypeKey> {
    use BinOpKind::*;

    let a = ctx.type_ctx.get_type(a_key);
    let b = ctx.type_ctx.get_type(b_key);


    match op {
        // ======================================
        // 加法：整数+整数，浮点+浮点，
        // 指针 + 整数
        // ======================================
        Plus => {
            if a.is_arithmetic() && b.is_arithmetic() {
                arith_promote(ctx, a_key, b_key, span)
            } else if a.is_pointer() && b.kind.is_integer() {
                // pointer + int  → pointer
                return Ok(a_key);
            } else if a.kind.is_integer() && b.is_pointer() {
                // int + pointer → pointer
                return Ok(b_key);
            } else {
                todo!("Plus 类型错误")
            }
        }

        // ======================================
        // 减法：整数/浮点
        // 指针 - 整数
        // 指针 - 指针 → ptrdiff_t
        // ======================================
        Minus => {
            if a.is_arithmetic() && b.is_arithmetic() {
                arith_promote(ctx, a_key, b_key, span)
            } else if a.is_pointer() && b.kind.is_integer() {
                // pointer - int → pointer
                Ok(a_key)
            } else if a.is_pointer() && b.is_pointer() {
                // pointer - pointer → ptrdiff_t
                // ptrdiff_t 定义为 long long
                Ok(type_context.get_int_type(IntegerSize::LongLong, true))
            } else {
                todo!("Minus 类型错误")
            }
        }

        // 乘法 / 除法，只允许算术类型
        Mul | Div => {
            if a.is_arithmetic() && b.is_arithmetic() {
                arith_promote(ctx, a_key, b_key, span)
            } else {
                todo!("Mul/Div 类型错误")
            }
        }

        // 取模：仅整数
        Mod => {
            if a.kind.is_integer() && b.kind.is_integer() {
                arith_promote(ctx, a_key, b_key, span)
            } else {
                todo!("Mod 类型错误")
            }
        }

        // 移位运算：a << b, a >> b
        // 左操作数必须是整数，右操作数必须是整数
        // 返回左操作数类型
        Shl | Shr => {
            if a.kind.is_integer() && b.kind.is_integer() {
                Ok(a_key)
            } else {
                todo!("Shift 类型错误")
            }
        }

        // 比较：< > <= >= == !=
        // 返回 int（或 bool）
        Lt | Gt | Le | Ge | Eq | Ne => {
            if a.is_arithmetic() && b.is_arithmetic() {
                return Ok(type_context.get_int_type(IntegerSize::Int, true));
            }
            if a.is_pointer() && b.is_pointer() {
                return Ok(type_context.get_int_type(IntegerSize::Int, true));
            }
            todo!("比较运算类型错误")
        }

        // 逻辑与 && ，逻辑或 ||
        // 返回 int
        And | Or => {
            if a.is_scalar() && b.is_scalar() {
                return Ok(type_context.get_int_type(IntegerSize::Int, true));
            }
            todo!("逻辑运算类型错误")
        }

        // ======================================
        // 按位运算：整数 &  |  ^
        // ======================================
        BitAnd | BitOr | BitXor => {
            if a.kind.is_integer() && b.kind.is_integer() {
                return arith_promote(ctx, a_key, b_key, span);
            }
            todo!("位运算类型错误")
        }

        Xor => {
            if a.is_scalar() && b.is_scalar() {
                return Ok(type_context.get_int_type(IntegerSize::Int, true));
            }
            todo!("Xor 类型错误")
        }

        // 逗号表达式：返回右侧类型
        Comma => {
            Ok(b_key)
        }
    }
}


fn assign_type(
    ctx: &CompCtx,
    a: &Expr,          // 左值类型
    op: AssignOpKind,
    b: &Expr,
    span: Span
) -> ParserResult<TypeKey> {
    use AssignOpKind::*;

    let aty_key = a.ty;
    let bty_key = b.ty;

    let aty = ctx.type_ctx.get_type(aty_key);
    let bty = ctx.type_ctx.get_type(bty_key);

    // 不是左值
    if !a.is_lvalue() {
        todo!("赋值给右值出错")
    }

    let bin_op = match op {
        PlusEq  => BinOpKind::Plus,
        MinusEq => BinOpKind::Minus,
        StarEq  => BinOpKind::Mul,
        SlashEq => BinOpKind::Div,
        PercentEq => BinOpKind::Mod,

        ShlEq   => BinOpKind::Shl,
        ShrEq   => BinOpKind::Shr,

        AmpEq   => BinOpKind::BitAnd,
        CaretEq => BinOpKind::BitXor,
        PipeEq  => BinOpKind::BitOr,

        Assign => match cast_compatible(aty, bty) {
            true => return Ok(aty_key),
            false => todo!(),
        }
    };

    let result_key = binary_type(ctx, aty_key, bin_op, bty_key, span)?;
    let result_ty = ctx.type_ctx.get_type(result_key);

    if !cast_compatible(aty, result_ty) {
        todo!("不兼容类型错误")
    }
    // C规范返回lhs操作数
    Ok(aty_key)
}



fn unary_type(
    ctx: &CompCtx,
    op: UnaryOpKind,
    a_key: TypeKey,
    value_type: ValueType,
    span: Span
) -> ParserResult<TypeKey> {
    let a = ctx.type_ctx.get_type(a_key);
    let ty = match op {
        UnaryOpKind::AddrOf => {
            if value_type != LValue {
                // return Err(ParserError::)
                todo!()
            } else if matches!(a.kind, TypeKind::Unknown) {
                todo!()
            }

            let kind = TypeKind::Pointer { elem_ty: a_key };
            let ty = Type::new_qual(Qualifier::default(), kind);
            type_context.get_or_set(ty)
        }
        UnaryOpKind::Deref => {
            match &a.kind {
                TypeKind::Pointer { elem_ty } => *elem_ty,
                TypeKind::Array { elem_ty, .. } => *elem_ty,
                _ => {
                    todo!()
                }
            }
        }

        // 返回自己
        UnaryOpKind::PostInc
        | UnaryOpKind::PostDec
        | UnaryOpKind::PreInc
        | UnaryOpKind::PreDec => match &a.kind {
            TypeKind::Integer{ .. }
            | TypeKind::Floating{ .. }
            | TypeKind::Pointer{ .. } => a,
            _ => todo!()
        }


        //
        UnaryOpKind::Plus
        | UnaryOpKind::Minus
        | UnaryOpKind::Not => match &a.kind {
            TypeKind::Integer{ .. }
            | TypeKind::Floating{ .. } => a,
            _ => todo!()
        },

        UnaryOpKind::BitNot => match &a.kind {
            TypeKind::Integer{ .. } => a,
            _ => todo!()
        },

    };
    Ok(ty)
}

/// 显示cast兼容性
fn cast_compatible(a: &Type, b: &Type) -> bool {
    use TypeKind::*;

    match (&a.kind, &b.kind) {
        // 基础类型，我不知道什么情况能出现这种情况
        (Void, Void) => true,

        // 整型 <-> 整型
        (Integer { .. }, Integer { .. }) => true,

        // 浮点 <-> 浮点
        (Floating { .. }, Floating { .. }) => true,

        // 整型 <-> 浮点
        (Integer { .. }, Floating { .. }) |
        (Floating { .. }, Integer { .. }) => true,

        // 指针 <-> 指针
        (Pointer { .. }, Pointer { .. }) => true,

        // 整数 <-> 指针
        (Pointer { .. }, Integer { .. }) |
        (Integer { .. }, Pointer { .. }) => true,

        // 数组衰变到指针
        (Array { elem_ty: aelem, .. }, Pointer { elem_ty: belem })
        | (Pointer { elem_ty: aelem }, Array { elem_ty: belem, .. }) // 指针 <-> 数组
        | (Array { elem_ty: aelem, .. }, Array { elem_ty: belem, .. }) // 数组 <-> 数组：要求元素类型一致
        => aelem == belem,

        // 函数类型
        (
            Function {
                ret_ty: ar,
                params: ap,
                is_variadic: av,
            },
            Function {
                ret_ty: br,
                params: bp,
                is_variadic: bv,
            }
        ) => {
            ar == br && av == bv
                && ap.len() == bp.len()
                && ap.iter().zip(bp.iter()).all(|(x, y)| x == y)
        }

        // todo 这里还差了一些 enum 之间的兼容
        // ===== Struct / StructRef =====
        (Record { id: id1, .. }, Record { id: id2, .. }) => id1 == id2, 
        (Enum { id: id1, .. }, Enum { id: id2, .. }) => id1 == id2,

        // ===== Unknown 不兼容任何 =====
        (Unknown, _) | (_, Unknown) => false,

        // 其余组合不兼容
        _ => false,
    }
}



pub fn int_promote(sz: IntegerSize) -> IntegerSize {
    match sz {
        IntegerSize::Char => IntegerSize::Int,
        IntegerSize::Short => IntegerSize::Int,
        IntegerSize::Int => IntegerSize::Int,
        IntegerSize::Long => sz,
        IntegerSize::LongLong => sz,
    }
}