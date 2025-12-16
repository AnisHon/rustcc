#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum IntegerSize {
    Char,
    Short,
    Int,
    Long,
    LongLong,
}

impl IntegerSize {
    pub fn to_code(self) -> &'static str {
        match self {
            IntegerSize::Char => "char",
            IntegerSize::Short => "short",
            IntegerSize::Int => "int",
            IntegerSize::Long => "long",
            IntegerSize::LongLong => "long long"
        }
    }

    pub fn rank(self) -> usize {
        match self {
            IntegerSize::Char => 0x1,
            IntegerSize::Short => 0x2,
            IntegerSize::Int => 0x3,
            IntegerSize::Long => 0x4,
            IntegerSize::LongLong => 0x5,
        }
    }

    pub fn sizeof(self) -> u64 {
        match self {
            IntegerSize::Char => 1,
            IntegerSize::Short => 2,
            IntegerSize::Int => 4,
            IntegerSize::Long => 8,
            IntegerSize::LongLong => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub enum FloatSize {
    Float,
    Double,
    LongDouble,
}

impl FloatSize {
    pub fn to_code(self) -> &'static str {
        match self {
            FloatSize::Float => "float",
            FloatSize::Double => "double",
            FloatSize::LongDouble => "long double",
        }
    }

    /// a > b?
    pub fn rank(&self) -> usize {
        match self {
            FloatSize::Float => 0x1,
            FloatSize::Double => 0x10,
            FloatSize::LongDouble => 0x100,
        }
    }

    pub fn sizeof(self) -> u64 {
        match self {
            FloatSize::Float => 4,
            FloatSize::Double => 8,
            FloatSize::LongDouble => 8
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ArraySize {
    Static(u64),    // int a[10]
    VLA, // int a[var]
    Incomplete,     // int a[]
}

impl ArraySize {
    pub fn get_static(&self) -> u64 {
        match self {
            ArraySize::Static(x) => *x,
            _ => unreachable!()
        }
    }

    pub fn to_code(&self) -> String {
        match self {
            ArraySize::Static(x) => format!("[{}]", x),
            ArraySize::VLA => "[]".to_owned(),
            ArraySize::Incomplete => "[]".to_owned(),
        }
    }
}