use crate::parser::ast::ExprKey;
use enum_as_inner::EnumAsInner;
use std::fmt::Display;

/// char 默认有三种形式，signed unsigned plain
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum CharSign {
    Signed,
    Unsigned,
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum IntegerSign {
    Signed,
    Unsigned,
}

/// char 被独立出去了
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum IntegerType {
    Short,
    Int,
    Long,
    LongLong,
}

impl Display for IntegerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IntegerType::*;
        let str = match self {
            Short => "short",
            Int => "int",
            Long => "long",
            LongLong => "long long",
        };
        write!(f, "{}", str)
    }
}

impl IntegerType {
    pub fn rank(self) -> usize {
        match self {
            IntegerType::Short => 0x2,
            IntegerType::Int => 0x3,
            IntegerType::Long => 0x4,
            IntegerType::LongLong => 0x5,
        }
    }

    pub fn sizeof(self) -> usize {
        match self {
            IntegerType::Short => 2,
            IntegerType::Int => 4,
            IntegerType::Long => 8,
            IntegerType::LongLong => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum FloatingType {
    Float,
    Double,
    LongDouble,
}

impl Display for FloatingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            FloatingType::Float => "float",
            FloatingType::Double => "double",
            FloatingType::LongDouble => "long double",
        };
        write!(f, "{}", str)
    }
}

impl FloatingType {
    /// a > b?
    pub fn rank(&self) -> usize {
        match self {
            FloatingType::Float => 0x1,
            FloatingType::Double => 0x10,
            FloatingType::LongDouble => 0x100,
        }
    }

    pub fn sizeof(self) -> usize {
        match self {
            FloatingType::Float => 4,
            FloatingType::Double => 8,
            FloatingType::LongDouble => 8,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EnumAsInner)]
pub enum ArraySize {
    Static(usize), // int a[10]
    VLA(ExprKey),  // int a[var]
    Incomplete,    // int a[]
}

impl Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySize::Static(x) => write!(f, "[{}]", x),
            ArraySize::VLA(_) => write!(f, "[...]"),
            ArraySize::Incomplete => write!(f, "[?]"),
        }
    }
}

impl ArraySize {
    pub fn get_static(&self) -> usize {
        match self {
            ArraySize::Static(x) => *x,
            _ => unreachable!(),
        }
    }
}
