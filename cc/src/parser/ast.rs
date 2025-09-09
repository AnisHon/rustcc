use std::rc::Rc;
use enum_as_inner::EnumAsInner;
use crate::lex::lex_yy::TokenType;
use crate::parser::span::Span;
use crate::types::symbol_table::{Symbol};
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

            impl Into<Option<$ty>> for ASTNode {
                fn into(self) -> Option<$ty> {
                    match self {
                        ASTNode::None => None,
                        _ => Some(self.into()),
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
    None,
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


impl TranslationUnit {
    pub fn make_translation_unit(ext_decl: ExternalDeclaration) -> ASTNode {
        let span = ext_decl.unwrap_span();
        TranslationUnit {
            ext_decls: vec![ext_decl],
            span
        }.into()
    }

    pub fn insert_ext_decl(mut translation_unit: TranslationUnit, ext_decl: ExternalDeclaration) {
        translation_unit.span.merge_self(&ext_decl.unwrap_span());
        translation_unit.ext_decls.push(ext_decl);
    }

}

// 外部声明：函数或变量
#[derive(Debug, Clone)]
pub enum ExternalDeclaration {
    Function(FunctionDefinition, Span),
    Variable(Declaration, Span),
}

impl ExternalDeclaration {

    pub fn make_func(func_def: FunctionDefinition) -> ASTNode {
        let span = func_def.span;
        Self::Function(func_def, span).into()
    }

    pub fn make_variable(decl: Declaration) -> ASTNode {
        let span = decl.span;
        Self::Variable(decl, span).into()
    }

    pub fn unwrap_span(&self) -> Span {
        match self {
            ExternalDeclaration::Function(_, x) => *x,
            ExternalDeclaration::Variable(_, x) => *x
        }
    }
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

impl FunctionDefinition {


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

impl Declaration {

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
    pub fn rank(&self) -> u8 {
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

    pub fn set_span(&mut self, set: Span) {
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

impl StorageClass {
    pub fn make(token: Token) -> ASTNode {
        let span = Span::from_token(&token);
        let result = match token.as_type().unwrap() {
            TokenType::KeywordTypedef => StorageClass::Typedef(span),
            TokenType::KeywordExtern => StorageClass::Extern(span),
            TokenType::KeywordStatic => StorageClass::Static(span),
            TokenType::KeywordAuto => StorageClass::Auto(span),
            TokenType::KeywordRegister => StorageClass::Register(span),
            _ => unreachable!()
        };
        result.into()
    }
}

// 类型限定符
#[derive(Debug, Clone, Default)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    // pub is_static: bool, // ?
}

impl Qualifiers {

    pub fn new(is_const: bool, is_volatile: bool) -> Self {
        Self {
            is_const,
            is_volatile
        }
    }
    pub fn make(decl: Option<Qualifiers>, qual: Token) -> ASTNode {
        let result = match (decl, qual.as_type().unwrap()) {
            (None, TokenType::KeywordConst) => Qualifiers::new(true, false),
            (Some(mut decl), TokenType::KeywordConst) => {decl.set_const(); decl},
            (None, TokenType::KeywordVolatile) => Qualifiers::new(false, true),
            (Some(mut decl), TokenType::KeywordVolatile) => {decl.set_volatile(); decl},
            _ => unreachable!(),
        };

        result.into()
    }

    pub fn set_const(&mut self) {
        // todo 检查
        self.is_const = true;
    }

    pub fn set_volatile(&mut self) {
        // todo 检查
        self.is_volatile = true;
    }
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

impl Field {

    pub fn make(name: String, ty: Type, span: Span) {

    }


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

    pub fn unwrap_span(&self) -> Span {
        match self {
            Statement::Labeled { span, .. }
            | Statement::Case { span, .. }
            | Statement::Default { span, .. }
            | Statement::Block(_, span)
            | Statement::Expression(_, span)
            | Statement::If { span, .. }
            | Statement::Switch { span, .. }
            | Statement::While { span, .. }
            | Statement::DoWhile { span, .. }
            | Statement::For { span, .. }
            | Statement::Goto { span, .. }
            | Statement::Continue(span)
            | Statement::Break(span)
            | Statement::Return(_, span) => span.clone()
        }
    }

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

    pub fn make_expression(expr: Option<Expression>, semi: Token) -> ASTNode {
        let span = Span::from_token(&semi);
        let span = expr.as_ref().map(|x| span.merge(&x.span)).unwrap_or(span);

        Statement::Expression(expr, span).into()
    }

    pub fn make_if(if_token: Token, cond: Expression, then_stmt: Statement, else_stmt: Option<Statement>) -> ASTNode {
        let span = Span::from_token(&if_token);
        let span = match &else_stmt {
            None => span,
            Some(x) => span.merge(&x.unwrap_span())
        };

        Statement::If { cond, then_stmt: Box::new(then_stmt), else_stmt: else_stmt.map(Box::new), span }.into()
    }

    pub fn make_switch(switch_token: Token, cond: Expression, body: Statement) -> ASTNode {
        let span = Span::from_token(&switch_token).merge(&body.unwrap_span());
        Statement::Switch { cond, body: Box::new(body), span }.into()
    }
    pub fn make_while(while_token: Token, cond: Expression, body: Statement, rparen: Option<Token>) -> ASTNode {
        let span = Span::from_token(&while_token).merge(&body.unwrap_span());
        let span = match rparen {
            None => span,
            Some(x) => Span::from_token(&x).merge(&span)
        };
        let result = match while_token.as_type().unwrap() {
            TokenType::KeywordWhile => Statement::While { cond, body: Box::new(body), span },
            TokenType::KeywordDo => Statement::DoWhile { cond, body: Box::new(body), span },
            _ => unreachable!()
        };
        result.into()
    }

    pub fn make_for(for_token: Token, init_opt: Option<Expression>, cond_opt: Option<Expression>, step_opt: Option<Expression>, body: Statement) -> ASTNode {
        let span = Span::from_token(&for_token).merge(&body.unwrap_span());
        Statement::For { init: init_opt, cond: cond_opt, step: step_opt, body: Box::new(body), span }.into()
    }

    /// 第一个token是goto
    pub fn make_goto(goto: Token, label: Token) -> ASTNode {
        let goto_span = Span::from_token(&goto);
        let label_span = Span::from_token(&label);

        let span = goto_span.merge(&label_span);
        let label = label.value.into_string().unwrap();
        Statement::Goto { label, span }.into()
    }


    pub fn make_continue_break(token: Token) -> ASTNode {
        let span = Span::from_token(&token);
        let result = match token.as_type().unwrap() {
            TokenType::KeywordContinue => Statement::Continue(span),
            TokenType::KeywordBreak => Statement::Break(span),
            _ => unreachable!()
        };

        result.into()
    }

    /// 第一个token是return
    pub fn make_return(ret: Token, expr: Option<Expression>) -> ASTNode {
        let ret_span = Span::from_token(&ret);
        let span = match &expr {
            None => ret_span,
            Some(expr) => ret_span.merge(&expr.span)
        };

        Statement::Return(expr, span).into()
    }
}

// 表达式
#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: Option<Type>, // 类型交给后期多次遍历时填充，
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
        let kind = ExpressionKind::Literal(constant, span.clone());
        Expression { kind, ty: None, span }.into()
    }

    pub fn make_id(token: Token) -> ASTNode {
        let name = token.value.into_string().unwrap();
        ExpressionKind::Id {name, decl_ref: None}.into()
    }

    /// 最后的token是 arr[...] <-这个字符，用来精确确定位置
    pub fn make_array_access(base: Expression, index: Expression, token: Token) -> ASTNode {
        let span = base.span.merge(&Span::from_token(&token));
        let kind = ExpressionKind::ArrayAccess { base: Box::new(base), index: Box::new(index) };
        Expression { kind, ty: None, span }.into()
    }

    /// 最后的token是 foo(...) <-这个字符，用来精确确定位置
    pub fn make_call(func: Expression, args: Vec<Expression>, token: Token) -> ASTNode {
        let span = func.span.merge(&Span::from_token(&token));
        let kind = ExpressionKind::Call {func: Box::new(func), args};

        Expression { kind, ty: None, span }.into()
    }

    pub fn make_field_access(base: Expression, field: Token) -> ASTNode {
        let span = base.span.merge(&Span::from_token(&field));
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::FieldAccess { base: Box::new(base), field };

        Expression { kind, ty: None, span }.into()
    }

    pub fn make_arrow(base: Expression, field: Token) -> ASTNode {
        let span = base.span.merge(&Span::from_token(&field));
        let field = field.value.into_string().unwrap();
        let kind = ExpressionKind::Arrow { base: Box::new(base), field };

        Expression { kind, ty: None, span }.into()

    }

    ///
    /// 构建 前后置 ++ --
    /// # Arguments
    /// expr:
    /// token:
    /// post: 是否是后置
    ///
    pub fn make_update(expr: Expression, token: Token, post: bool) -> ASTNode {
        let span = Span::from_token(&token).merge(&expr.span);

        let kind = match (token.as_type().unwrap(), post) {
            (TokenType::OpDec, true) => ExpressionKind::PostDec,
            (TokenType::OpDec, false) => ExpressionKind::PreDec,
            (TokenType::OpInc, true) => ExpressionKind::PostInc,
            (TokenType::OpInc, false) => ExpressionKind::PreInc,
            _ => unreachable!()
        };

        let kind = kind(Box::new(expr));
        Expression { kind, ty: None, span }.into()
    }

    pub fn make_unary(token: Token, expr: Expression) -> ASTNode {
        let token_span = Span::from_token(&token);
        let span = token_span.merge(&expr.span);
        let op = match token.as_type().unwrap() {
            TokenType::OpBitand => UnaryOp::AddressOf(token_span),
            TokenType::OpTimes => UnaryOp::Deref(token_span),
            TokenType::OpPlus => UnaryOp::Plus(token_span),
            TokenType::OpMinus => UnaryOp::Minus(token_span),
            TokenType::OpBitNot => UnaryOp::BitNot(token_span),
            TokenType::OpNot => UnaryOp::LogicalNot(token_span),
            _ => unreachable!()
        };
        let kind = ExpressionKind::Unary {op, expr: Box::new(expr)};

        Expression { kind , ty: None, span }.into()
    }

    /// 第一个token是sizeof的值 -> sizeof expr
    pub fn make_sizeof_expr(sizeof: Token, expr: Expression) -> ASTNode {
        let span = expr.span.clone();
        let kind = ExpressionKind::SizeofExpr(Box::new(expr));

        Expression { kind, ty: None, span }.into()
    }

    /// 第一个token是sizeof的值 -> sizeof(type) <- 第二个是第二个括号
    pub fn make_sizeof_type(sizeof: Token, typ: Type, rparen: Token) -> ASTNode {
        let span = Span::from_token(&sizeof).merge(&Span::from_token(&rparen));
        let kind = ExpressionKind::SizeofType(typ);

        Expression { kind, ty: None, span }.into()
    }


    /// 第一个token 是类型转换的第一个括号-> (X)X
    pub fn make_cast(token: Token, typ: Type, expr: Expression) -> ASTNode {
        let span = Span::from_token(&token).merge(&expr.span);
        let kind = ExpressionKind::Cast { ty: typ, expr: Box::new(expr) };

        Expression { kind, ty: None, span }.into()
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
        let kind = ExpressionKind::Binary { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };

        Expression { kind, ty: None, span }.into()
    }


    pub fn make_conditional(cond: Expression, then_expr: Expression, else_expr: Expression) -> ASTNode {
        let span = cond.span.merge(&else_expr.span);
        let kind = ExpressionKind::Conditional {
            cond: Box::new(cond),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr)
        };

        Expression { kind, ty: None, span }.into()
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
        let kind = ExpressionKind::Assign { lhs: Box::new(lhs), op, rhs: Box::new(rhs) };
        Expression { kind, ty: None, span }.into()
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

    pub fn insert_str(mut constant: Constant, token: Token) -> ASTNode {
        let token_str = token.value.as_string().unwrap();
        let token_span = Span::from_token(&token);
        let (str, span) = match &mut constant {
            Constant::String(str, span) => (str, span),
            _ => unreachable!(),
        };

        span.merge_self(&token_span);
        str.push_str(token_str);
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
