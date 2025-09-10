use crate::parser::ast::*;

fn type_binary(lhs: &Type, op: &BinaryOp, rhs: &Type) -> Type {
    match (lhs, rhs) {
        (Type::NamedType {name, ..}, Type::NamedType {..}) => {return todo!()}
        (Type::NamedType {name, ..}, _) => {return todo!()}
        (_, Type::NamedType {..}) => {return todo!()}
        (_, _) => {}
    }

    let span = op.unwrap_span();
    match op {
        // 逻辑运算 & 比较运算
        BinaryOp::LogicalAnd(_) | BinaryOp::LogicalOr(_) |
        BinaryOp::Lt(_) | BinaryOp::Gt(_) | BinaryOp::Le(_) |
        BinaryOp::Ge(_) | BinaryOp::Eq(_) | BinaryOp::Ne(_) => {
            Type::Integer { signed: true, size: IntegerSize::Int, span }
        }

        // 算术运算
        BinaryOp::Add(_) | BinaryOp::Sub(_) | BinaryOp::Mul(_) |
        BinaryOp::Div(_) | BinaryOp::Mod(_) => {
            arithmetic_result(lhs, op, rhs)
        }

        // 位运算
        BinaryOp::Shl(_) | BinaryOp::Shr(_) | BinaryOp::BitAnd(_) |
        BinaryOp::BitXor(_) | BinaryOp::BitOr(_) => {
            if lhs.is_integer() && rhs.is_integer() {
                Type::Integer { signed: true, size: IntegerSize::Int, span }
            } else {
                panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs)
            }
        }
    }
}

/// 会抛出panic，以后做成错误处理
fn arithmetic_result(lhs: &Type, op: &BinaryOp, rhs: &Type) -> Type {
    let span = op.unwrap_span();
    // 指针运算
    match (lhs, rhs) {
        (Type::Pointer(base, _), t) | (t, Type::Pointer(base, _)) => {
            match op {
                BinaryOp::Add(_) | BinaryOp::Sub(_) => {},
                _ => panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs),
            }
            if t.is_integer() {
                // 指针 + 整数 → 指针
                Type::Pointer(base.clone(), span)
            } else if let (Type::Pointer(_, _), Type::Pointer(_, _)) = (lhs, rhs) {
                if let BinaryOp::Sub(_) = op {
                    // 指针 - 指针 → ptrdiff_t (简化用 long)
                    Type::Integer { signed: true, size: IntegerSize::Long, span }
                } else {
                    panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs)
                }
            } else {
                panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs)
            }
        }

        // 普通算术运算
        _ if lhs.is_arithmetic() && rhs.is_arithmetic() => {
            // 整数/浮点提升
            if lhs.is_floating() || rhs.is_floating() {
                let size = match (lhs, rhs) {
                    (Type::Floating { size: ls, .. }, Type::Floating { size: rs, .. }) => std::cmp::max(*ls, *rs),
                    (Type::Floating { size: ls, .. }, _) => *ls,
                    (_, Type::Floating { size: rs, .. }) => *rs,
                    _ => FloatSize::Double,
                };
                Type::Floating { size, span }
            } else {
                // 整数提升
                let rank = std::cmp::max(lhs.rank(), rhs.rank());
                let size = match rank {
                    1 => IntegerSize::Char,
                    2 => IntegerSize::Short,
                    3 => IntegerSize::Int,
                    4 => IntegerSize::Long,
                    _ => IntegerSize::Int,
                };
                Type::Integer { signed: true, size, span }
            }
        }

        _ => panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs)
    }
}


/// 根据左值类型和右值类型推导赋值运算结果类型
pub fn assign_type(lhs: &Type, op : &AssignOp, rhs: &Type) -> Type {
    match op {
        AssignOp::Assign(_) => {
            // 普通赋值：右侧表达式隐式转换为左侧类型
            if lhs.is_arithmetic() && rhs.is_arithmetic() {
                // 整数/浮点类型互转
                lhs.clone()
            } else if lhs.is_pointer() && rhs.is_pointer() {
                // 指针赋值，必须同类型或兼容
                lhs.clone()
            } else if lhs.is_pointer() && rhs.is_integer() {
                // 允许 rhs 为 0（NULL）
                // 可以在语义分析阶段检查 rhs 是否为 0
                lhs.clone()
            } else {
                panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs)
            }
        }

        // 其他复合赋值
        AssignOp::AddAssign(_) | AssignOp::SubAssign(_) |
        AssignOp::MulAssign(_) | AssignOp::DivAssign(_) |
        AssignOp::ModAssign(_) | AssignOp::ShlAssign(_) |
        AssignOp::ShrAssign(_) | AssignOp::AndAssign(_) |
        AssignOp::XorAssign(_) | AssignOp::OrAssign(_) => {
            // 先做 lhs op rhs 类型检查
            let op = match op {
                AssignOp::AddAssign(s) => BinaryOp::Add(*s),
                AssignOp::SubAssign(s) => BinaryOp::Sub(*s),
                AssignOp::MulAssign(s) => BinaryOp::Mul(*s),
                AssignOp::DivAssign(s) => BinaryOp::Div(*s),
                AssignOp::ModAssign(s) => BinaryOp::Mod(*s),
                AssignOp::ShlAssign(s) => BinaryOp::Shl(*s),
                AssignOp::ShrAssign(s) => BinaryOp::Shr(*s),
                AssignOp::AndAssign(s) => BinaryOp::BitAnd(*s),
                AssignOp::XorAssign(s) => BinaryOp::BitXor(*s),
                AssignOp::OrAssign(s) => BinaryOp::BitOr(*s),
                _ => unreachable!(),
            };

            let result_type = type_binary(lhs, &op, rhs);
            // 赋值时类型必须可转换为左值类型
            if lhs.is_arithmetic() && result_type.is_arithmetic() {
                lhs.clone()
            } else if lhs.is_pointer() && result_type.is_pointer() {
                lhs.clone()
            } else {
                panic!("InvalidOperands {:?} {:?} {:?}", lhs, op, rhs)
            }
        }
    }
}


pub fn array_access_type(base: &Type, index: &Type) -> Type {
    match index {
        Type::Integer { .. } => {}
        Type::NamedType { .. } => {
            todo!()
            // 引用类型单独处理
        }
        _ => panic!("Cannot apply subscript to {:?} and {:?}", base, index)
    }
    match base {
        Type::Pointer(elem_ty, span)
        | Type::Array { elem_ty, span, .. } => {
            let mut new_typ = (**elem_ty).clone();
            let span = span.merge(&new_typ.unwarp_span());
            new_typ.set_span(span);
            new_typ
        }
        Type::NamedType { .. } => {
            todo!()
            // 引用类型单独实现
        }
        _ => panic!("Cannot apply subscript to {:?} and {:?}", base, index)
    }

}

pub fn func_call_type(func: &Type, args: Vec<&Type>) -> Type {
    match func {
        Type::Pointer(typ, _) if typ.is_function() => { // 只管解一层引用
            func_call_type(typ, args)
        }
        Type::Function { ret_ty, .. } => {
            (**ret_ty).clone()
        }
        Type::NamedType { .. } => {
            // todo 符号表
            todo!()
        }
        _ => panic!("Called object type '{:?}' is not a function or function pointer", func)
    }
}

pub fn field_access(base: &Type, field: &String, arrow: bool) -> Type {
    let error_msg = if arrow {"Left side of 'operator ->' has non-pointer type"} else {"Left side of member access has non-class type"};

    match base {
        Type::Pointer(ty, _) => field_access(ty, field, true),
        Type::Array { elem_ty, .. } => field_access(elem_ty, field, true),
        Type::Struct { .. } => {
            // todo 等待符号表
            todo!()
        }
        Type::Union { .. } => {
            // todo 等待符号表
            todo!()
        }
        Type::NamedType { .. } => {
            // todo 等待符号表
            todo!()
        }
        _ => panic!("{} {:?}", error_msg, base)
    }
}

/// 类型提升
fn promote_type(typ: Type) -> Type {
    match typ {
        Type::Integer { signed, size, span } => {
            let (signed, size) = match size {
                IntegerSize::Char => (true, IntegerSize::Int),
                IntegerSize::Short => (true, IntegerSize::Int),
                IntegerSize::Int => (signed, IntegerSize::Int),
                IntegerSize::Long => (signed, IntegerSize::Long),
            };
            Type::Integer { signed, size, span }
        },
        _ => typ,
    }
}

// fn resolve_named_type()


/// 解引用类型
fn deref_type(typ: Type) -> Type {
    match typ {
        Type::Pointer(inner, _) => *inner,
        Type::Array { elem_ty, .. } => *elem_ty,
        Type::Function { .. } => typ, // 据我所知函数解引用多少次都没用
        Type::NamedType { .. } => {
            // todo 符号表处理
            todo!()
        }
        _ => panic!("deref type is not a pointer"), // 可以交给后期错误处理
    }
}