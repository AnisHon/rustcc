use enum_as_inner::EnumAsInner;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum Signedness {
    Signed,
    Unsigned,
    Plain,
}
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
pub enum FloatType {
    Float,
    Double,
    LongDouble,
}

impl Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            FloatType::Float => "float",
            FloatType::Double => "double",
            FloatType::LongDouble => "long double",
        };
        write!(f, "{}", str)
    }
}

impl FloatType {
    /// a > b?
    pub fn rank(&self) -> usize {
        match self {
            FloatType::Float => 0x1,
            FloatType::Double => 0x10,
            FloatType::LongDouble => 0x100,
        }
    }

    pub fn sizeof(self) -> usize {
        match self {
            FloatType::Float => 4,
            FloatType::Double => 8,
            FloatType::LongDouble => 8,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, EnumAsInner)]
pub enum ArraySize {
    Static(usize), // int a[10]
    VLA,           // int a[var]
    Incomplete,    // int a[]
}

impl Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySize::Static(x) => write!(f, "[{}]", x),
            ArraySize::VLA => write!(f, "[...]"),
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
