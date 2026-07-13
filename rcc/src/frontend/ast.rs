#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn join(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
    pub is_atomic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CType {
    pub kind: TypeKind,
    pub qualifiers: Qualifiers,
}

impl CType {
    pub fn new(kind: TypeKind) -> Self {
        Self {
            kind,
            qualifiers: Qualifiers::default(),
        }
    }
    pub fn int() -> Self {
        Self::new(TypeKind::Int { signed: true })
    }
    pub fn uint() -> Self {
        Self::new(TypeKind::Int { signed: false })
    }
    pub fn void() -> Self {
        Self::new(TypeKind::Void)
    }
    pub fn pointer(to: CType) -> Self {
        Self::new(TypeKind::Pointer(Box::new(to)))
    }
    pub fn is_integer(&self) -> bool {
        matches!(
            self.kind,
            TypeKind::Bool
                | TypeKind::Char { .. }
                | TypeKind::Short { .. }
                | TypeKind::Int { .. }
                | TypeKind::Long { .. }
                | TypeKind::LongLong { .. }
                | TypeKind::Enum { .. }
        )
    }
    pub fn is_arithmetic(&self) -> bool {
        self.is_integer()
            || matches!(
                self.kind,
                TypeKind::Float
                    | TypeKind::Double
                    | TypeKind::LongDouble
                    | TypeKind::Complex(_)
                    | TypeKind::Imaginary(_)
            )
    }
    pub fn is_scalar(&self) -> bool {
        self.is_arithmetic() || matches!(self.kind, TypeKind::Pointer(_))
    }
    pub fn decay(&self) -> Self {
        match &self.kind {
            TypeKind::Array { element, .. } => CType::pointer((**element).clone()),
            TypeKind::Function { .. } => CType::pointer(self.clone()),
            _ => self.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Void,
    Bool,
    Char {
        signed: Option<bool>,
    },
    Short {
        signed: bool,
    },
    Int {
        signed: bool,
    },
    Long {
        signed: bool,
    },
    LongLong {
        signed: bool,
    },
    Float,
    Double,
    LongDouble,
    Complex(Box<CType>),
    Imaginary(Box<CType>),
    Pointer(Box<CType>),
    Array {
        element: Box<CType>,
        size: ArraySize,
    },
    Function {
        return_type: Box<CType>,
        params: Vec<Parameter>,
        variadic: bool,
        has_prototype: bool,
    },
    Struct {
        name: Option<String>,
        fields: Option<Vec<Field>>,
    },
    Union {
        name: Option<String>,
        fields: Option<Vec<Field>>,
    },
    Enum {
        name: Option<String>,
        variants: Option<Vec<EnumVariant>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArraySize {
    Constant(usize),
    Variable(Box<Expression>),
    Unspecified,
    Star,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Option<String>,
    pub ty: CType,
    pub bit_width: Option<u32>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: i64,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: Option<String>,
    pub ty: CType,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageClass {
    Typedef,
    Extern,
    Static,
    Auto,
    Register,
    ThreadLocal,
    StaticThreadLocal,
    ExternThreadLocal,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FunctionSpecifiers {
    pub is_inline: bool,
    pub is_noreturn: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationUnit {
    pub declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExternalDeclaration {
    Declaration(Declaration),
    Function(FunctionDefinition),
    StaticAssert(StaticAssertion),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticAssertion {
    pub condition: Expression,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: Option<String>,
    pub ty: CType,
    pub storage: StorageClass,
    pub function_specifiers: FunctionSpecifiers,
    pub initializer: Option<Initializer>,
    pub alignment: Option<usize>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub ty: CType,
    pub storage: StorageClass,
    pub function_specifiers: FunctionSpecifiers,
    pub parameters: Vec<Parameter>,
    pub body: Statement,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Initializer {
    Expression(Expression),
    List(Vec<InitializerItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitializerItem {
    pub designators: Vec<Designator>,
    pub value: Initializer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Designator {
    Field(String),
    Index(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Expression(Option<Expression>),
    Declaration(Vec<Declaration>),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StatementKind {
    Empty,
    Expression(Expression),
    Compound(Vec<BlockItem>),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Switch {
        expression: Expression,
        body: Box<Statement>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    For {
        init: ForInit,
        condition: Option<Expression>,
        step: Option<Expression>,
        body: Box<Statement>,
    },
    Label {
        name: String,
        statement: Box<Statement>,
    },
    Case {
        value: Expression,
        statement: Box<Statement>,
    },
    Default {
        statement: Box<Statement>,
    },
    Goto(String),
    Continue,
    Break,
    Return(Option<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
    StaticAssert(StaticAssertion),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCategory {
    LValue,
    RValue,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub ty: CType,
    pub category: ValueCategory,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Integer(i128),
    Floating(f64),
    Character {
        value: i64,
        encoding: StringEncoding,
    },
    String {
        value: String,
        encoding: StringEncoding,
    },
    Identifier(String),
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Assignment {
        op: AssignOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Subscript {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    Member {
        base: Box<Expression>,
        field: String,
        indirect: bool,
    },
    Cast {
        target: CType,
        expression: Box<Expression>,
    },
    SizeofType(CType),
    SizeofExpression(Box<Expression>),
    Alignof(CType),
    CompoundLiteral {
        ty: CType,
        initializer: Box<Initializer>,
    },
    GenericSelection {
        controlling: Box<Expression>,
        selected: Box<Expression>,
    },
    PostIncrement {
        operand: Box<Expression>,
        decrement: bool,
    },
    Comma(Vec<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Narrow,
    Utf8,
    Utf16,
    Utf32,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    LogicalNot,
    BitNot,
    AddressOf,
    Dereference,
    PreIncrement,
    PreDecrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BitAnd,
    BitXor,
    BitOr,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitXor,
    BitOr,
}
