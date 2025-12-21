use std::collections::hash_map::Entry;

use crate::err::type_error::TypeError;
use crate::lex::types::token_kind::{FloatSuffix, IntSuffix};
use crate::parser::ast::TypeKey;
use crate::parser::ast::types::{ArraySize, EnumID, FloatSize, IntegerSize, RecordID, Type};
use crate::parser::semantic::sema::type_ctx::type_builder::{TypeBuilder, TypeBuilderKind};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;

pub struct TypeCtx {
    types: FxHashMap<TypeBuilder, TypeKey>,
    pool: SlotMap<TypeKey, Type>,

    enum_counter: usize,
    record_counter: usize,
}
impl TypeCtx {
    pub fn new() -> Self {
        let types = FxHashMap::default();
        let pool = SlotMap::with_key();
        let mut ctx = Self {
            types,
            pool,
            enum_counter: 0,
            record_counter: 0,
        };

        Self::init(&mut ctx);

        ctx
    }

    // 初始化一些常用类型
    pub fn init(ctx: &mut Self) {
        use IntegerSize::*;
        let char_ = TypeBuilder::new_int(true, Char);
        let uchar = TypeBuilder::new_int(false, Char);
        let short = TypeBuilder::new_int(true, Short);
        let ushort = TypeBuilder::new_int(false, Short);
        let int = TypeBuilder::new_int(true, Int);
        let uint = TypeBuilder::new_int(false, Int);
        let long = TypeBuilder::new_int(true, Long);
        let ulong = TypeBuilder::new_int(false, Long);
        let ll = TypeBuilder::new_int(true, LongLong);
        let ull = TypeBuilder::new_int(false, LongLong);

        let float = TypeBuilder::new_float(FloatSize::Float);
        let double = TypeBuilder::new_float(FloatSize::Double);
        let long_double = TypeBuilder::new_float(FloatSize::LongDouble);

        let void = TypeBuilder::new(TypeBuilderKind::Void);
        let unknown = TypeBuilder::new(TypeBuilderKind::Unknown);

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
            let _ = ctx.build_type(ele);
        }
    }

    pub fn get_type(&self, key: TypeKey) -> &Type {
        self.pool.get(key).expect("Type not exist")
    }

    pub fn get_type_mut(&mut self, key: TypeKey) -> &mut Type {
        self.pool.get_mut(key).expect("Type not exists")
    }

    /// 单例获取type
    pub fn build_type(&mut self, ty: TypeBuilder) -> Result<TypeKey, TypeError> {
        let entry = self.types.entry(ty);
        let key = match entry {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(v) => {
                let value = v.key().clone().build()?;
                let id = self.pool.insert(value);
                *v.insert(id)
            }
        };
        Ok(key)
    }

    // int 类型
    pub fn get_int_type(&self, size: IntegerSize, is_signed: bool) -> TypeKey {
        let ty = TypeBuilder::new_int(is_signed, size);
        *self.types.get(&ty).expect("already initialized")
    }

    // float 类型
    pub fn get_float_type(&self, size: FloatSize) -> TypeKey {
        let ty = TypeBuilder::new_float(size);
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
        let kind = TypeBuilderKind::Void;
        let ty = TypeBuilder::new(kind);

        *self.types.get(&ty).expect("already initialized")
    }

    // 字符串类型, 无法保证 immutable
    pub fn get_string_type(&mut self, sz: u64) -> Result<TypeKey, TypeError> {
        let elem_ty = self.get_char();
        let size = ArraySize::Static(sz);
        let kind = TypeBuilderKind::Array { elem_ty, size };
        let ty = TypeBuilder::new(kind);

        self.build_type(ty)
    }

    /// 获取未知类型
    pub fn get_unknown_type(&mut self) -> TypeKey {
        let ty = TypeBuilder::new(TypeBuilderKind::Unknown);
        *self.types.get(&ty).expect("already initialized")
    }

    pub fn next_record_id(&mut self) -> RecordID {
        let record_id = RecordID(self.record_counter);
        self.record_counter += 1;
        record_id
    }

    pub fn next_enum_id(&mut self) -> EnumID {
        let enum_id = EnumID(self.enum_counter);
        self.enum_counter += 1;
        enum_id
    }
}
