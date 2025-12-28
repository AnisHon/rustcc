use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
    LongLong,
}

impl Display for IntegerSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IntegerSize::*;
        let str = match self {
            Char => "char",
            Short => "short",
            Int => "int",
            Long => "long",
            LongLong => "long long",
        };
        write!(f, "{}", str)
    }
}

impl IntegerSize {
    pub fn rank(self) -> usize {
        use IntegerSize::*;
        match self {
            Char => 0x1,
            Short => 0x2,
            Int => 0x3,
            Long => 0x4,
            LongLong => 0x5,
        }
    }

    pub fn sizeof(self) -> usize {
        use IntegerSize::*;
        match self {
            Char => 1,
            Short => 2,
            Int => 4,
            Long => 8,
            LongLong => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum FloatSize {
    Float,
    Double,
    LongDouble,
}

impl FloatSize for IntegerSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FloatSize::*;
        let str = match self {
            Float => "float",
            Double => "double",
            LongDouble => "long double",
        };
        write!(f, "{}", str)
    }
}

impl FloatSize {
    /// a > b?
    pub fn rank(&self) -> usize {
        use FloatSize::*;
        match self {
            Float => 0x1,
            Double => 0x10,
            LongDouble => 0x100,
        }
    }

    pub fn sizeof(self) -> usize {
        use FloatSize::*;
        match self {
            Float => 4,
            Double => 8,
            LongDouble => 8,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArraySize {
    Static(usize), // int a[10]
    VLA,           // int a[var]
    Incomplete,    // int a[]
}

impl Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArraySize::*;
        match self {
            Static(x) => write!(f, "[{}]", x),
            VLA => write!(f, "[...]"),
            Incomplete => write!(f, "[?]"),
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
