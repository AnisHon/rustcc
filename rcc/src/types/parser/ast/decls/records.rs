// 函数声明 函数定义

pub struct RecordField {
    pub bit_field: Option<ExprKey>,
}

pub struct RecordDecl {
    pub kind: StructOrUnion,
    pub def: Option<DeclKey>,
}

pub struct RecordDef {
    pub kind: StructOrUnion,
    pub fields: Vec<DeclGroup>, // 当 fields 为 none 时为不完全类型
}

pub struct EnumField {
    pub expr: Option<ExprKey>,
}

pub struct EnumDecl {
    pub def: Option<DeclKey>,
}

pub struct EnumDef {
    pub enums: Option<Vec<DeclKey>>, // 当 enums 为 none 时为不完全类型
}
