use crate::err::parser_error;
use crate::err::parser_error::{ParserError, ParserResult};
use crate::lex::types::token_kind::{IntSuffix, LiteralKind, Symbol};
use crate::parser::ast::decl::DeclKind;
use crate::parser::ast::exprs::{AssignOpKind, BinOpKind, Expr, ExprKey, ExprKind, MemberAccessKind, UnaryOpKind};
use crate::parser::ast::types::{IntegerSize, Qualifier, Type, TypeKey, TypeKind};
use crate::parser::common::Ident;
use crate::parser::comp_ctx::CompCtx;
use crate::parser::semantic::sema::expr::value_type::ValueType::LValue;
use crate::types::span::Span;
use crate::parser::semantic::sema::expr::value_type::ValueType;


/// 构建expression 折叠表达式
pub fn make_expr(ctx: &mut CompCtx, kind: ExprKind, span: Span) -> ParserResult<ExprKey> {
    let ty = expr_type(ctx, &kind, span)?;

    let expr = Expr { kind, ty, span };

    let expr = fold_expr(ctx, expr)?;

    Ok(expr)
}

/// 检查和计算当前表达式的类型
fn expr_type(ctx: &CompCtx, kind: &ExprKind, span: Span) -> ParserResult<TypeKey> {
    let ty = match kind {
        ExprKind::DeclRef(x) => var_expr_type(ctx, x)?,
        ExprKind::Constant(x) => type_context.get_constant_type(x),
        // ExprKind::Paren { expr, .. } => Rc::clone(&expr.ty),
        ExprKind::ArraySubscript { base, index, .. } => {
            let base = ctx.get_expr(*base);
            let index = ctx.get_expr(*index);
            let index_ty = ctx.get_type(index.ty);
            let base_ty = ctx.get_type(base.ty);
            if !index_ty.kind.is_integer() {
                todo!("数组索引非整数")
            }
            match &base_ty.kind {
                TypeKind::Pointer { elem_ty }
                | TypeKind::Array { elem_ty, .. } => *elem_ty,
                _ => return Err(ParserError::new(parser_error::ErrorKind::NonSubscripted, span))
            }
        }
        ExprKind::Call { base, params, .. } => {
            let base = ctx.get_expr(*base);
            call_expr_type(ctx, base.ty, &params.exprs, span)?
        }
        ExprKind::MemberAccess { base, field, kind, .. } => {
            let base = ctx.get_expr(*base);
            member_access_expr_type(ctx, base.ty, kind.clone(), *field, span)?
        }
        ExprKind::SizeofType { .. } | ExprKind::SizeofExpr { .. } => {
            type_context.get_int_type(IntegerSize::Long, false)
        }
        ExprKind::Unary { op, rhs } => {
            let rhs = ctx.get_expr(*rhs);  
            let valuety = ValueType::value_type(rhs)
            unary_type(ctx, op.kind.clone(), rhs.ty, valuety, span)?
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let lhs = ctx.get_expr(*lhs);
            let rhs = ctx.get_expr(*rhs); 
            binary_type(ctx, lhs, op.kind.clone(), rhs.ty.clone(), span)?
        }
        ExprKind::Assign { lhs, op, rhs } => {
            let lhs = ctx.get_expr(*lhs);
            let rhs = ctx.get_expr(*rhs);
            assign_type(ctx, lhs, op.kind.clone(), rhs, span)?
        }
        ExprKind::Cast { ty, expr, .. } => {
            let from = ctx.get_expr(*expr).ty;
            let to = *ty;
            cast_expr_type(ctx, from, to, span)?
        }
        ExprKind::Ternary { cond, then_expr, else_expr, .. } =>
            ternary_type(ctx, cond.ty.clone(), then_expr.ty.clone(), else_expr.ty.clone(), span)?,
    };

    Ok(ty)
}

fn var_expr_type(ctx: &CompCtx, ident: &Ident) -> ParserResult<TypeKey> {
    let decl = lookup_chain(ident.symbol).ok_or(ParserError::undefined_symbol(ident))?;
    let decl = ctx.get_decl(decl);
    let ty = match &decl.kind {
        DeclKind::EnumField { .. }
        | DeclKind::VarInit { .. }
        | DeclKind::ParamVar => decl.ty,
        DeclKind::Func { .. }
        | DeclKind::FuncRef => {
            let type_kind = TypeKind::Pointer { elem_ty: decl.ty };
            let ty = Type::new_qual(Qualifier::default(), type_kind);
            let ty = type_context.get_or_set(ty);
            ty
        }
        _ => return Err(ParserError::undefined_symbol(ident))
    };
    Ok(ty)
}

fn call_expr_type(ctx: &CompCtx, ty: TypeKey, call_params: &[ExprKey], span: Span) -> ParserResult<TypeKey> {
    let ty = ctx.get_type(ty);
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
    let ty = ctx.get_type(ty_key);
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
    let from = ctx.get_type(from_key);
    let to = ctx.get_type(to_key);
    if cast_compatible(from, to) {
        Ok(to_key)
    } else {
        Err(ParserError::error("Wrong Cast".to_owned(), span))
    }
}

/// 三元运算符类型
fn ternary_type(
    ctx: &CompCtx,
    cond: TypeKey,
    a_key: TypeKey,
    b_key: TypeKey,
    span: Span,
) -> ParserResult<TypeKey>
{
    use TypeKind::*;

    let cond = ctx.get_type(cond);
    let a = ctx.get_type(a_key);
    let b = ctx.get_type(b_key);

    // cond 必须是可转换为 bool/整数的类型
    match &cond.kind {
        Integer { .. } | Floating { .. } | Pointer { .. } => {}
        Void => todo!("条件表达式不能为 void"),
        _ => todo!("条件表达式类型不合法"),
    }

    // a 和 b 完全相同 —— 直接返回
    if a == b {
        return Ok(a_key);
    }

    // 都是算术类型 → usual arithmetic conversion
    if a.is_arithmetic() && b.is_arithmetic() {
        return arith_promote(ctx, a, a_key, b, b_key, span);
    }

    // ============================================================
    // 4. 两者都是结构体/联合，名字必须一致
    // ============================================================
    match (&a.kind, &b.kind) {
        (StructRef { name: na }, StructRef { name: nb }) if na == nb => {
            return Ok(a_key);
        }
        (UnionRef { name: na }, UnionRef { name: nb }) if na == nb => {
            return Ok(a_key);
        }
        _ => {}
    }

    // void + void → void
    if matches!(a.kind, Void) && matches!(b.kind, Void) {
        return Ok(type_context.get_void_type());
    }

    // 指针处理 cast_compatible
    if let Pointer { elem_ty: ae } = &a.kind {
        if let Pointer { elem_ty: be } = &b.kind {
            // 指向同元素类型 → 返回该指针
            if ae == be {
                return Ok(a_key);
            }

            // void* + T* → void*
            if a.is_void_ptr(ctx) {
                return Ok(a_key);
            }
            if b.is_void_ptr(ctx) {
                return Ok(b_key);
            }

            // 不兼容
            todo!("三元运算符的两个指针类型不兼容")
        }
    }
    // 尝试转换为二者其一
    if cast_compatible(&a, &b) {
        return Ok(b_key)
    }
    if cast_compatible(&b, &a) {
        return Ok(a_key);
    }

    todo!("三元运算符两侧类型无法进行转换")
}

/// 算数时类型提升
fn arith_promote(
    ctx: &CompCtx,
    a_key: TypeKey,
    b_key: TypeKey,
    span: Span
) -> ParserResult<TypeKey> {
    use TypeKind::*;

    let a = ctx.get_type(a_key);
    let b = ctx.get_type(b_key);

    // 1. 两者都是浮点，返回较宽浮点
    if let (Floating { size: sa }, Floating { size: sb }) = (&a.kind, &b.kind) {
        let ty = match sa.rank() > sb.rank() {
            true => a_key,
            false => b_key,
        };
        return Ok(ty);
    }

    // 2. 浮点 + 整数 → 浮点类型，整数先转浮点
    match (&a.kind, &b.kind) {
        (Floating { .. }, Integer { .. }) => return Ok(a_key),
        (Integer { .. }, Floating { size: fs }) => return Ok(b_key),
        _ => {}
    }

    // 3. 两者都是整数 → integer promotion + rank 比较
    if let (Integer { is_signed: sa, size: ra },
        Integer { is_signed: sb, size: rb }) = (&a.kind, &b.kind)
    {
        // (1) 做 integer promotion
        let sza = int_promote(*ra);
        let szb = int_promote(*rb);

        // let pa = ctx.get_int_type(sza, *sa);
        // let pb = ctx.get_int_type(szb, *sb);

        // (2) 比较 rank
        if ra.rank() != rb.rank() {
            let ty = match ra.rank() > rb.rank() {
                true => type_context.get_int_type(sza, *sa),
                false => type_context.get_int_type(szb, *sb)
            };
            return Ok(ty)
        }

        // (3) rank 相同，处理 signed/unsigned 混合情况
        match (sa, sb) {
            // 都是无符号
            (false, false) => {
                return Ok(type_context.get_int_type(sza, *sa))
            }
            // 都是有符号
            (true, true) => {
                return Ok(type_context.get_int_type(sza, *sa))
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
                    return Ok(type_context.get_int_type(*unsigned_side.1, false))
                } else {
                    // 构造 signed
                    return Ok(type_context.get_int_type(*signed_side.1, true))
                }
            }
        }
    }

    Err(ParserError::error("Not a arithmetic type".to_owned(), span))
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

    let a = ctx.get_type(a_key);
    let b = ctx.get_type(b_key);

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

    let aty = ctx.get_type(aty_key);
    let bty = ctx.get_type(bty_key);

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
    let result_ty = ctx.get_type(result_key);

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
    let a = ctx.get_type(a_key);
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

        // ===== Struct / StructRef =====
        (Struct { name: Some(a), .. }, Struct { name: Some(b), .. }) => a == b,
        (StructRef { name: a }, StructRef { name: b }) => a == b,

        // 定义体和引用互相兼容（名字一致）
        (Struct { name: Some(a), .. }, StructRef { name: b })
        | (StructRef { name: a }, Struct { name: Some(b), .. })
        | (Union { name: Some(a), .. }, Union { name: Some(b), .. })
        | (UnionRef { name: a }, UnionRef { name: b }) => a == b,

        (Union { name: Some(a), .. }, UnionRef { name: b }) => a == b,
        (UnionRef { name: a }, Union { name: Some(b), .. }) => a == b,

        // ===== Enum / EnumRef =====
        (Enum { name: Some(a), .. }, Enum { name: Some(b), .. }) => a == b,
        (EnumRef { name: a }, EnumRef { name: b }) => a == b,

        (Enum { name: Some(a), .. }, EnumRef { name: b }) => a == b,
        (EnumRef { name: a }, Enum { name: Some(b), .. }) => a == b,

        // enum 定义与 enum 定义（无名枚举）不可兼容，也不允许互转换
        (Enum { name: None, .. }, Enum { name: None, .. }) => false,

        // ===== Unknown 不兼容任何 =====
        (Unknown, _) | (_, Unknown) => false,

        // 其余组合不兼容
        _ => false,
    }
}

fn make_unsigned_long_long(n: u64, ty: TypeKey, span: Span) -> ExprKind {
    let kind = ExprKind::Constant(LiteralKind::Integer { suffix: Some(IntSuffix::ULL), value: n });
    todo!()
}

/// 折叠常量表达式
fn fold_expr(ctx: &mut CompCtx, expr: Expr) -> ParserResult<ExprKey> {
    let kind: ExprKind = match expr.kind {
        // ExprKind::Paren { expr, .. } => return Ok(expr), // 折叠括号
        ExprKind::SizeofExpr { expr: sizeof_expr, .. } => {
            let sizeof_expr = ctx.get_expr(sizeof_expr);
            let ty = ctx.get_type(sizeof_expr.ty);
            let size = ty.sizeof();
            make_unsigned_long_long(size, expr.ty.clone(), expr.span) // 折叠sizeof
        }
        ExprKind::SizeofType { ty, .. } => { // 折叠sizeof
            let ty = ctx.get_type(ty);
            let size = ty.sizeof();
            make_unsigned_long_long(size, expr.ty.clone(), expr.span)
        }
        ExprKind::Unary { op, rhs } => // 折叠运算
            fold_unary(ctx, op.kind, rhs),
        ExprKind::Binary { lhs, op, rhs } =>  // 折叠运算
            fold_binary(ctx, lhs, op.kind, rhs),
        ExprKind::Cast { expr, .. } => return Ok(expr),  // 折叠类型转换
        ExprKind::Ternary { cond: cond_key, then_expr, else_expr} => { // 折叠三元运算
            let cond = ctx.get_expr(cond_key);
            match &cond.kind.as_constant() {
                Some(x) => match x.is_true()? {
                    true => return Ok(then_expr),
                    false => return Ok(else_expr),
                }
                None => ExprKind::Ternary { cond: cond_key, then_expr, else_expr },
            }
        }
        _ => return Ok(ctx.insert_expr(expr)), // 不折叠
    };

    let expr = Expr::new(kind, expr.ty, expr.span);
    let expr_key= ctx.insert_expr(expr);

    Ok(expr_key)
}


fn fold_unary(ctx: &CompCtx, op: UnaryOpKind, rhs_key: ExprKey) -> ExprKind {
    let rhs = ctx.get_expr(rhs_key);
    if !rhs.kind.is_constant() {
        return 
    }

    match op {
        UnaryOpKind::Plus => ,
        UnaryOpKind::Minus =>  ,
        UnaryOpKind::Not => ,
        UnaryOpKind::BitNot => ,
        _ => rhs.kind,
    }
}

fn fold_binary(ctx: &CompCtx, lhs: ExprKey, op: BinOpKind, rhs: ExprKey) -> ExprKind {
    todo!()
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