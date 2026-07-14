use crate::ap::ap_int::APInt;
use crate::ap::ap_value::APValue;
use crate::parser::ast::ExprKey;
use enum_as_inner::EnumAsInner;

/// 值
#[derive(Debug, Clone, EnumAsInner, Default)]
pub enum ExprEval {
    #[default]
    NotConst,
    ConstExpr {
        value: APValue,
    },
    ICE {
        value: APInt,
    }, // integer const expression
    AddressConst {
        // symbol: SymbolID,
        offset: i64,
    },
}

/// 尝试解析为 ICE
pub fn try_as_integer(expr: ExprKey) -> ExprEval {
    todo!()
}
