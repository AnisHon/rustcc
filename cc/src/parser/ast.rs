use std::rc::Rc;
use enum_as_inner::EnumAsInner;
use crate::lex::lex_yy::TokenType;
use crate::parser::span::Span;
use crate::types::symbol_table::{Symbol, SymbolTable};
use crate::types::token::{Token, TokenValue};

// =============================
// 宏定义
// =============================
macro_rules! impl_from_variants {
    ($enum:ident { $($variant:ident($ty:ty)),* $(,)? }) => {
        $(
            impl From<$ty> for $enum {
                fn from(v: $ty) -> Self {
                    $enum::$variant(v)
                }
            }

            impl Into<$ty> for $enum {
                fn into(self) -> $ty {
                    match self {
                        $enum::$variant(inner) => inner,
                        _ => panic!("failed to convert {} to {}", stringify!($enum), stringify!($variant)),
                    }
                }
            }
        )*
    }
}


#[derive(Debug)]
pub enum ASTNode {
    TranslationUnit(TranslationUnit),
    ExternalDeclaration(ExternalDeclaration),
    FunctionDefinition(FunctionDefinition),
    Declaration(Declaration),
    Type(Type),
    IntegerSize(IntegerSize),
    FloatSize(FloatSize),
    StorageClass(StorageClass),
    Qualifiers(Qualifiers),
    Field(Field),
    Parameter(Parameter),
    Initializer(Initializer),
    Block(Block),
    BlockItem(BlockItem),
    Statement(Statement),
    Expression(Expression),
    ExpressionList(Vec<Expression>),
    ExpressionKind(ExpressionKind),
    Constant(Constant),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
    AssignOp(AssignOp),
    Token(Token),
}

// =============================
//  Into From 实现
// =============================
impl_from_variants!(ASTNode {
    TranslationUnit(TranslationUnit),
    ExternalDeclaration(ExternalDeclaration),
    FunctionDefinition(FunctionDefinition),
    Declaration(Declaration),
    Type(Type),
    IntegerSize(IntegerSize),
    FloatSize(FloatSize),
    StorageClass(StorageClass),
    Qualifiers(Qualifiers),
    Field(Field),
    Parameter(Parameter),
    Initializer(Initializer),
    Block(Block),
    BlockItem(BlockItem),
    Statement(Statement),
    Expression(Expression),
    ExpressionList(Vec<Expression>),
    ExpressionKind(ExpressionKind),
    Constant(Constant),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
    AssignOp(AssignOp),
    Token(Token),
});


// 顶层翻译单元
#[derive(Debug, Clone)]
pub struct TranslationUnit {
    pub ext_decls: Vec<ExternalDeclaration>,
    pub span: Span,
}

// 外部声明：函数或变量
#[derive(Debug, Clone)]
pub enum ExternalDeclaration {
    Function(FunctionDefinition, Span),
    Variable(Declaration, Span),
}

// 函数定义
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub ret_ty: Type,
    pub params: Vec<Parameter>,
    pub is_variadic: bool,
    pub body: Option<Block>, // None for extern declarations
    pub span: Span,
}

// 变量声明
#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub ty: Type,
    pub storage: Option<StorageClass>,
    pub qualifiers: Qualifiers,
    pub init: Option<Initializer>,
    pub span: Span,
}

// 类型系统
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void(Span),
    Integer { signed: bool, size: IntegerSize, span: Span},
    Floating { size: FloatSize, span: Span},
    Pointer(Box<Type>, Span),
    Array { elem_ty: Box<Type>, size: Option<u64>, span: Span}, // size is constant-evaluated
    Function { ret_ty: Box<Type>, params: Vec<Type>, is_variadic: bool, span: Span },
    Struct { name: Option<String>, fields: Vec<Field>, span: Span },
    Union { name: Option<String>, fields: Vec<Field>, span: Span },
    Enum { name: Option<String>, values: Vec<(String, i64)>, span: Span },
    NamedType { name: String, decl_ref: Option<Rc<Symbol>>,span: Span, }
}

impl Type {
    fn string_type(len: u64, span: Span) -> Type {
        Type::Array { elem_ty: Box::new(Type::Integer { signed: false, size: IntegerSize::Char, span }), size: Some(len), span }
    }

    /// 返回类型等级（越大精度越高）
    fn rank(&self) -> u8 {
        match self {
            Type::Integer { size, .. } => match size {
                IntegerSize::Char => 1,
                IntegerSize::Short => 2,
                IntegerSize::Int => 3,
                IntegerSize::Long => 4,
            },
            Type::Floating { size, .. } => match size {
                FloatSize::Float => 6,
                FloatSize::Double => 7,
            },
            Type::Pointer(_, _) => 9, // 指针类型等级最高，用于 +,- 可合法
            _ => 0,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Integer { .. })
    }

    pub fn is_floating(&self) -> bool {
        matches!(self, Type::Floating { .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_, _))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }

    pub fn is_arithmetic(&self) -> bool {
        self.is_integer() || self.is_floating()
    }

    pub fn is_named_type(&self) -> bool {
        matches!(self, Type::NamedType { .. })
    }
}

impl Type {
    pub fn unwarp_span(&self) -> Span {
        match self {
            Type::Void(x) => *x,
            Type::Integer { span, .. } => *span,
            Type::Floating { span, .. } => *span,
            Type::Pointer(_, x) => *x,
            Type::Array { span, .. } => *span,
            Type::Function { span, .. } => *span,
            Type::Struct { span, .. } => *span,
            Type::Union { span, .. } => *span,
            Type::Enum { span, .. } => *span,
            Type::NamedType { span, .. } => *span,
        }
    }

    fn set_span(&mut self, set: Span) {
        match self {
            Type::Void(span)
            | Type::Integer { span, .. }
            | Type::Floating { span, .. }
            | Type::Pointer(_, span)
            | Type::Array { span, .. }
            | Type::Function { span, .. }
            | Type::Struct { span, .. }
            | Type::Union { span, .. }
            | Type::Enum { span, .. }
            | Type::NamedType { span, .. } => {
                *span = set;
            }
        }
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum FloatSize {
    Float = 4,
    Double = 8,
}

// 存储类
#[derive(Debug, Clone)]
pub enum StorageClass {
    Typedef(Span),
    Extern(Span),
    Static(Span),
    Auto(Span),
    Register(Span),
}

// 类型限定符
#[derive(Debug, Clone, Default)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_static: bool,
}

// 结构体/联合体字段
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub bit_width: Option<u32>, // for bitfields
    pub span: Span,
}

// 函数参数
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Option<String>,
    pub ty: Type,
    pub span: Span,
}

// 初始化器
#[derive(Debug, Clone)]
pub enum Initializer {
    Scalar(Expression, Span),
    List(Vec<Initializer>, Span),
}

// 语句块
#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Declaration(Declaration, Span),
    Statement(Statement, Span),
}

// 语句
#[derive(Debug, Clone)]
pub enum Statement {
    Labeled { label: String, stmt: Box<Statement>, span: Span },
    Case { value: i64, stmt: Box<Statement>, span: Span }, // constant-evaluated
    Default { stmt: Box<Statement>, span: Span },
    Block(Block, Span),
    Expression(Option<Expression>, Span),
    If { cond: Expression, then_stmt: Box<Statement>, else_stmt: Option<Box<Statement>>, span: Span },
    Switch { cond: Expression, body: Box<Statement>, span: Span },
    While { cond: Expression, body: Box<Statement>, span: Span },
    DoWhile { body: Box<Statement>, cond: Expression, span: Span },
    For { init: Option<Expression>, cond: Option<Expression>, step: Option<Expression>, body: Box<Statement>, span: Span },
    Goto { label: String, span: Span },
    Continue(Span),
    Break(Span),
    Return(Option<Expression>, Span),
}

impl Statement {
    /// constexpr应该会被归并成为一个常量表达式，最终被计算
    pub fn make_case(constexpr: Expression, stmt: Statement) -> ASTNode {
        let (constant, span) = match constexpr.kind {
            ExpressionKind::Literal(constant, span) => (constant, span),
            _ => unreachable!()
        };

        let value = match constant {
            Constant::Int(x, _) => x,
            Constant::Char(x, _) => x as i64,
            _ => panic!("Clangd: Integer constant expression must have integer type, not '{:?}'", constexpr.ty)
        };

        Statement::Case {value, stmt: Box::new(stmt), span}.into()
    }
}

// 表达式
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Type, // 推导出的类型
    pub span: Span,
}

impl Expression {

    pub fn is_lvalue(&self) -> bool {
        match &self.kind {
            ExpressionKind::Id { .. } => true,                    // 变量
            ExpressionKind::ArrayAccess { .. } => true,             // a[i]
            ExpressionKind::FieldAccess { .. } => true,            // s.f 或 s->f
            ExpressionKind::Arrow { .. } => true,
            ExpressionKind::PostInc(_) => true,
            ExpressionKind::PostDec(_) => true,
            ExpressionKind::Unary { op, .. } => op.is_lvalue(), // *p
            ExpressionKind::Cast { expr, .. } => expr.is_lvalue(),
            ExpressionKind::Assign { .. } => true,                // a = b 是右值
            _ => false,
        }
    }

    pub fn is_rvalue(&self) -> bool {
        !self.is_lvalue()
    }

}

impl Expression {

    pub fn make_literal(constant: Constant) -> ASTNode {
        let span = constant.unwrap_span();
        let ty = constant.get_type();
        let kind = ExpressionKind::Literal(constant, span.clone());
        Expression { kind, ty, span }.into()
    }

    pub fn make_id(token: Token, symbol_table: SymbolTable) -> ASTNode {
        let name = token.value.into_string().unwrap();
        let symbol = match symbol_table.lookup(&name) {
            None => panic!("Undefined Symbol {}", name),
            Some(x) => x,
        };
        ExpressionKind::Id {name, decl_ref: Some(symbol)}.into()
    }

    /// 最后的token是 arr[...] <-这个字符，用来精确确定位置
    pub fn make_array_access(base: Expression, index: Expression, token: Token) -> ASTNode {
        let span = base.span.merge(&Span::from_token(&token));
        let ty = array_access_type(&base.ty, &index.ty);
        let kind = ExpressionKind::ArrayAccess { base: Box::new(base), index: Box::new(index) };
        Expression { kind, ty, span }.into()
    }

    /// 最后的token是 foo(...) <-这个字符，用来精确确定位置
    pub fn make_call(func: Expression, args: Vec<Expression>, token: Token) -> ASTNode {
        let span = func.span.merge(&Span::from_token(&token));
        let arg_refs: Vec<_> = args.iter().map(|x| &x.ty).collect();
        let ty = func_call_type(&func.ty, arg_refs);
        let kind = ExpressionKind::Call {func: Box::new(func), args};

        Expression { kind, ty, span }.into()
    }

    pub fn make_field_access(base: Expression, field: Token) -> ASTNode {
        let span = base.span.merge(&Span::from_token(&field));
        let field = field.value.into_string().unwrap();
        let ty = field_access(&base.ty, &field, false);
        let kind = ExpressionKind::FieldAccess { base: Box::new(base), field };

        Expression { kind, ty, span }.into()
    }

    pub fn make_arrow(base: Expression, field: Token) -> ASTNode {
        let span = base.span.merge(&Span::from_token(&field));
        let field = field.value.into_string().unwrap();
        let ty = field_access(&base.ty, &field, true);
        let kind = ExpressionKind::Arrow { base: Box::new(base), field };

        Expression { kind, ty, span }.into()

    }

    pub fn make_post_inc(expr: Expression) -> ASTNode {
        if expr.is_rvalue() {
            panic!("Cannot apply postfix operator++ to rvalue");
        }
        todo!()
    }

    pub fn make_post_dec(expr: Expression) -> ASTNode {
        if expr.is_rvalue() {
            panic!("Cannot apply postfix operator-- to rvalue");
        }
        todo!()
    }

    pub fn make_pre_inc(expr: Expression) -> ASTNode {
        if expr.is_rvalue() {
            panic!("Cannot apply prefix operator++ to rvalue");
        }
        todo!()
    }

    pub fn make_pre_dec(expr: Expression) -> ASTNode {
        if expr.is_rvalue() {
            panic!("Cannot apply prefix operator-- to rvalue");
        }
        todo!()
    }

    pub fn make_unary(token: Token, expr: Expression) -> ASTNode {
        todo!()
    }

    pub fn make_sizeof_expr(expr: Expression) -> ASTNode {
        todo!()
    }

    pub fn make_sizeof_type(typ: Type) -> ASTNode {
        todo!()
    }


    /// 第一个token 是类型转换的第一个括号-> (X)X
    pub fn make_cast(token: Token, typ: Type, expr: Expression) -> ASTNode {
        todo!()
    }

    pub fn make_binary(lhs: Expression, token: Token, rhs: Expression) -> ASTNode {
        let span = lhs.span.merge(&rhs.span);
        let span_token = Span::from_token(&token);


        let op = match token.as_type().unwrap() {
            TokenType::OpPlus => BinaryOp::Add(span_token),
            TokenType::OpMinus => BinaryOp::Sub(span_token),
            TokenType::OpTimes => BinaryOp::Mul(span_token),
            TokenType::OpDivide => BinaryOp::Div(span_token),
            TokenType::OpMod => BinaryOp::Mod(span_token),
            TokenType::OpLShift => BinaryOp::Shl(span_token),
            TokenType::OpRShift => BinaryOp::Shr(span_token),
            TokenType::OpLt => BinaryOp::Lt(span_token),
            TokenType::OpGt => BinaryOp::Gt(span_token),
            TokenType::OpLe => BinaryOp::Le(span_token),
            TokenType::OpGe => BinaryOp::Ge(span_token),
            TokenType::OpEq => BinaryOp::Eq(span_token),
            TokenType::OpNe => BinaryOp::Ne(span_token),
            TokenType::OpBitand => BinaryOp::BitAnd(span_token),
            TokenType::OpXor => BinaryOp::BitXor(span_token),
            TokenType::OpBitor => BinaryOp::BitOr(span_token),
            TokenType::OpAnd => BinaryOp::LogicalAnd(span_token),
            TokenType::OpOr => BinaryOp::LogicalOr(span_token),
            _ => unreachable!()
        };
        let ty = type_binary(&lhs.ty, &op, &rhs.ty);
        let kind = ExpressionKind::Binary { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };

        Expression { kind, ty, span }.into()
    }

    pub fn make_conditional(cond: Expression, then_expr: Expression, else_expr: Expression) -> ASTNode {
        todo!()
    }

    pub fn make_assign(lhs: Expression, token: Token, rhs: Expression) -> ASTNode {
        if lhs.is_rvalue() {
            panic!("Cannot assign to rvalue");
        }
        let span_token = Span::from_token(&token);

        let op = match token.as_type().unwrap() {
            TokenType::OpAssign => AssignOp::Assign(span_token),
            TokenType::OpMulAssign => AssignOp::MulAssign(span_token),
            TokenType::OpDivAssign => AssignOp::DivAssign(span_token),
            TokenType::OpModAssign => AssignOp::ModAssign(span_token),
            TokenType::OpAddAssign => AssignOp::AddAssign(span_token),
            TokenType::OpSubAssign => AssignOp::SubAssign(span_token),
            TokenType::OpLShiftAssign => AssignOp::ShlAssign(span_token),
            TokenType::OpRShiftAssign => AssignOp::ShrAssign(span_token),
            TokenType::OpAndAssign => AssignOp::AndAssign(span_token),
            TokenType::OpOrAssign => AssignOp::OrAssign(span_token),
            _ => unreachable!(),
        };
        let span = lhs.span.merge(&rhs.span);
        let ty = assign_type(&lhs.ty, &op, &rhs.ty);
        let kind = ExpressionKind::Assign { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };
        Expression { kind, ty, span }.into()
    }

    pub fn make_comma(mut exprs: Vec<Expression>, expr: Expression) -> ASTNode {
        exprs.push(expr);
        exprs.into()
    }


}


#[derive(Debug, Clone, EnumAsInner)]
pub enum ExpressionKind {
    Literal(Constant, Span),
    Id { name: String, decl_ref: Option<Rc<Symbol>> }, // decl_ref 指向符号表索引
    ArrayAccess { base: Box<Expression>, index: Box<Expression> },
    Call { func: Box<Expression>, args: Vec<Expression> },
    FieldAccess { base: Box<Expression>, field: String },
    Arrow { base: Box<Expression>, field: String },
    PostInc(Box<Expression>),
    PostDec(Box<Expression>),
    PreInc(Box<Expression>),
    PreDec(Box<Expression>),
    Unary { op: UnaryOp, expr: Box<Expression> },
    SizeofExpr(Box<Expression>),
    SizeofType(Type),
    Cast { ty: Type, expr: Box<Expression> },
    Binary { op: BinaryOp, lhs: Box<Expression>, rhs: Box<Expression> },
    Conditional { cond: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
    Assign { lhs: Box<Expression>, op: AssignOp, rhs: Box<Expression> },
    Comma { exprs: Vec<Expression> },
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64, Span),
    Float(f64, Span),
    Char(u8, Span),
    String(String, Span), // 合并后的字符串字面量
}

impl Constant {
    pub fn make(token: Token) -> ASTNode {
        let span = Span::from_token(&token);

        let constant = match token.value {
            TokenValue::Number { value, .. } => Self::Int(value as i64, span),
            TokenValue::Float(value) => Self::Float(value, span),
            TokenValue::String(value) => Self::String(value, span),
            TokenValue::Char(value) => Self::Char(value, span),
            TokenValue::Other => unreachable!(),
        };

        constant.into()
    }

    pub fn unwrap_span(&self) -> Span {
        match self {
            Constant::Int(_, x) => *x,
            Constant::Float(_, x) => *x,
            Constant::Char(_, x) => *x,
            Constant::String(_, x) => *x,
        }
    }

    pub fn get_type(&self) -> Type {
        let span = self.unwrap_span();
        match self {
            Constant::Int(_, _) => Type::Integer {signed: true, size: IntegerSize::Int, span},
            Constant::Float(_, _) => Type::Floating {size: FloatSize::Float, span},
            Constant::Char(_, _) => Type::Integer {signed: true, size: IntegerSize::Char, span},
            Constant::String(x, _) => Type::string_type(x.len() as u64, span),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    AddressOf(Span),
    Deref(Span),
    Plus(Span),
    Minus(Span),
    BitNot(Span),
    LogicalNot(Span),
}

impl UnaryOp {
    pub fn unwrap_span(&self) -> Span {
        match self {
            UnaryOp::AddressOf(x) => *x,
            UnaryOp::Deref(x) => *x,
            UnaryOp::Plus(x) => *x,
            UnaryOp::Minus(x) => *x,
            UnaryOp::BitNot(x) => *x,
            UnaryOp::LogicalNot(x) => *x,
        }
    }

    pub fn is_lvalue(&self) -> bool {
        match self {
            UnaryOp::AddressOf(_) => false,
            UnaryOp::Deref(_) => true,
            UnaryOp::Plus(_) => false,
            UnaryOp::Minus(_) => false,
            UnaryOp::BitNot(_) => true,
            UnaryOp::LogicalNot(_) => false
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add(Span),
    Sub(Span),
    Mul(Span),
    Div(Span),
    Mod(Span),
    Shl(Span),
    Shr(Span),
    Lt(Span),
    Gt(Span),
    Le(Span),
    Ge(Span),
    Eq(Span),
    Ne(Span),
    BitAnd(Span),
    BitXor(Span),
    BitOr(Span),
    LogicalAnd(Span),
    LogicalOr(Span),
}

impl BinaryOp {
    pub fn unwrap_span(&self) -> Span {
        match self {
            BinaryOp::Add(s) => *s,
            BinaryOp::Sub(s) => *s,
            BinaryOp::Mul(s) => *s,
            BinaryOp::Div(s) => *s,
            BinaryOp::Mod(s) => *s,
            BinaryOp::Shl(s) => *s,
            BinaryOp::Shr(s) => *s,
            BinaryOp::Lt(s) => *s,
            BinaryOp::Gt(s) => *s,
            BinaryOp::Le(s) => *s,
            BinaryOp::Ge(s) => *s,
            BinaryOp::Eq(s) => *s,
            BinaryOp::Ne(s) => *s,
            BinaryOp::BitAnd(s) => *s,
            BinaryOp::BitXor(s) => *s,
            BinaryOp::BitOr(s) => *s,
            BinaryOp::LogicalAnd(s) => *s,
            BinaryOp::LogicalOr(s) => *s,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Assign(Span),
    MulAssign(Span),
    DivAssign(Span),
    ModAssign(Span),
    AddAssign(Span),
    SubAssign(Span),
    ShlAssign(Span),
    ShrAssign(Span),
    AndAssign(Span),
    XorAssign(Span),
    OrAssign(Span),
}


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