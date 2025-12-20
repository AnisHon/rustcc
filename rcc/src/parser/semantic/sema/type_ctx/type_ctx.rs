use crate::lex::types::token_kind::{FloatSuffix, IntSuffix};
use crate::parser::ast::types::{ArraySize, FloatSize, IntegerSize, Type, TypeKey, TypeKind};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;

pub struct TypeCtx {
    types: FxHashMap<Type, TypeKey>,
    pool: SlotMap<TypeKey, Type>,
}
impl TypeCtx {
    pub fn new() -> Self {
        let types = FxHashMap::default();
        let pool = SlotMap::with_key();
        let mut ctx = Self { types, pool };

        Self::init(&mut ctx);

        ctx
    }

    // 初始化一些常用类型
    pub fn init(ctx: &mut Self) {
        use IntegerSize::*;
        let char_ = Type::new_int(true, Char);
        let uchar = Type::new_int(false, Char);
        let short = Type::new_int(true, Short);
        let ushort = Type::new_int(false, Short);
        let int = Type::new_int(true, Int);
        let uint = Type::new_int(false, Int);
        let long = Type::new_int(true, Long);
        let ulong = Type::new_int(false, Long);
        let ll = Type::new_int(true, LongLong);
        let ull = Type::new_int(false, LongLong);

        let float = Type::new_float(FloatSize::Float);
        let double = Type::new_float(FloatSize::Double);
        let long_double = Type::new_float(FloatSize::LongDouble);

        let void = Type::new(TypeKind::Void);
        let unknown = Type::new(TypeKind::Unknown);

        let types = vec![
            char_,
            uchar,
            short,
            ushort,
            int,
            uint,
            long,
            ulong,
            ll,
            ull,
            void,
            unknown,
            float,
            double,
            long_double,
        ];

        for ele in types {
            let _ = ctx.get_or_set(ele);
        }
    }

    pub fn get_type(&self, key: TypeKey) -> &Type {
        self.pool.get(key).expect("Type Not Exist")
    }

    /// 单例获取type
    pub fn get_or_set(&mut self, ty: Type) -> TypeKey {
        use TypeKind::*;
        let key = match &ty.kind {
            // 普通的类型使用自身作为键
            Void
            | Unknown
            | Integer { .. }
            | Floating { .. }
            | Pointer { .. }
            | Array { .. }
            | Function { .. } => {
                // 单例 entry insert with
                *self
                    .types
                    .entry(ty)
                    .or_insert_with_key(|x| self.pool.insert(x.clone()))
            }

            Struct { .. }
            | Union { .. }
            | Enum { .. }
            | StructRef { .. }
            | UnionRef { .. }
            | EnumRef { .. } => {
                // 不单例
                self.pool.insert(ty)
            }
        };

        key
    }

    // int 类型
    pub fn get_int_type(&self, size: IntegerSize, is_signed: bool) -> TypeKey {
        let ty = Type::new_int(is_signed, size);
        *self.types.get(&ty).expect("already initialized")
    }

    // float 类型
    pub fn get_float_type(&self, size: FloatSize) -> TypeKey {
        let ty = Type::new_float(size);
        *self.types.get(&ty).expect("already initialized")
    }

    // 通过 int 的 suffix 获取类型
    pub fn get_by_int_sfx(&self, sfx: Option<IntSuffix>) -> TypeKey {
        use IntSuffix::*;
        use IntegerSize::*;
        sfx.map(|x| match x {
            U => self.get_int_type(Int, false),
            L => self.get_int_type(Long, true),
            UL => self.get_int_type(Long, false),
            LL => self.get_int_type(LongLong, true),
            ULL => self.get_int_type(LongLong, false),
        })
        .unwrap_or(self.get_int_type(Int, true))
    }

    // 通过 float 的 suffix 获取类型
    pub fn get_by_float_sfx(&mut self, sfx: Option<FloatSuffix>) -> TypeKey {
        use FloatSize::*;
        use FloatSuffix::*;
        let size = sfx
            .map(|x| match x {
                F => Float,
                L => LongDouble,
            })
            .unwrap_or(Double);
        self.get_float_type(size)
    }

    // char 类型
    pub fn get_char(&self) -> TypeKey {
        self.get_int_type(IntegerSize::Char, true)
    }

    // 获取 void type
    pub fn get_void_type(&self) -> TypeKey {
        let kind = TypeKind::Void;
        let ty = Type::new(kind);

        *self.types.get(&ty).expect("already initialized")
    }

    // 字符串类型, 无法保证 immutable
    pub fn get_string_type(&mut self, sz: u64) -> TypeKey {
        let elem_ty = self.get_char();
        let size = ArraySize::Static(sz);
        let kind = TypeKind::Array { elem_ty, size };
        let ty = Type::new(kind);

        self.get_or_set(ty)
    }

    /// 获取未知类型
    pub fn get_unknown_type(&mut self) -> TypeKey {
        let ty = Type::new(TypeKind::Unknown);
        *self.types.get(&ty).expect("already initialized")
    }
}
