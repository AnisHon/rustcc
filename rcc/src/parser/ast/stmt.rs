use super::{Declaration, Expression, StaticAssertion};
use crate::source::SourceRange;

#[derive(Debug, Clone, PartialEq)]
/// Statement payload paired with the full source range that produced it.
pub struct Statement {
    pub kind: StatementKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
/// The two grammar alternatives allowed in the first clause of a `for` loop.
pub enum ForInit {
    Expression(Option<Expression>),
    Declaration(Vec<Declaration>),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
/// C11 statement forms after condition and return conversions are inserted.
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
/// Items that may occur directly inside a compound statement.
pub enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
    StaticAssert(StaticAssertion),
}
