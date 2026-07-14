use std::collections::hash_map::Entry;

use crate::parser::ast::common::RecordKind;
use crate::parser::ast::types::type_builder::TagTypeBuilder;
use crate::parser::ast::types::{ArrayType, BuildInType, CharSign, IntegerSign, PtrType, QualType};
use crate::types::lex::token_kind::{FloatSuffix, IntSuffix};
use crate::types::parser::ast::types::type_builder::TypeBuilder;
use crate::types::parser::ast::types::{
    ArraySize, EnumID, FloatingType, IntegerType, RecordID, Type,
};
use crate::types::parser::ast::TypeKey;
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
        Self {
            types,
            pool,
            enum_counter: 0,
            record_counter: 0,
        }
    }

    pub fn get_type(&self, key: TypeKey) -> &Type {
        self.pool.get(key).expect("Type not exist")
    }

    pub fn get_type_mut(&mut self, key: TypeKey) -> &mut Type {
        self.pool.get_mut(key).expect("Type not exists")
    }

    /// 单例获取type
    pub fn build_type(&mut self, ty: TypeBuilder) -> TypeKey {
        let entry = self.types.entry(ty);
        match entry {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(v) => {
                let value = v.key().clone().build();
                let id = self.pool.insert(value);
                *v.insert(id)
            }
        }
    }

    // int 类型
    pub fn new_int_type(&mut self, size: IntegerType, sign: IntegerSign) -> TypeKey {
        let ty = TypeBuilder::new_int(sign, size);
        self.build_type(ty)
    }

    // float 类型
    pub fn new_float_type(&mut self, size: FloatingType) -> TypeKey {
        let ty = TypeBuilder::new_float(size);
        self.build_type(ty)
    }

    // 通过 int 的 suffix 获取类型
    pub fn new_by_int_sfx(&mut self, sfx: Option<IntSuffix>) -> TypeKey {
        use IntSuffix::*;
        use IntegerType::*;
        sfx.map(|x| match x {
            U => self.new_int_type(Int, IntegerSign::Unsigned),
            L => self.new_int_type(Long, IntegerSign::Signed),
            UL => self.new_int_type(Long, IntegerSign::Unsigned),
            LL => self.new_int_type(LongLong, IntegerSign::Signed),
            ULL => self.new_int_type(LongLong, IntegerSign::Unsigned),
        })
        .unwrap_or(self.new_int_type(Int, IntegerSign::Signed))
    }

    // 通过 float 的 suffix 获取类型
    pub fn new_by_float_sfx(&mut self, sfx: Option<FloatSuffix>) -> TypeKey {
        use FloatSuffix::*;
        use FloatingType::*;
        let size = sfx
            .map(|x| match x {
                F => Float,
                L => LongDouble,
            })
            .unwrap_or(Double);
        self.new_float_type(size)
    }

    // char 类型
    pub fn new_char(&mut self, signedness: CharSign) -> TypeKey {
        let builder = TypeBuilder::new_char(signedness);
        self.build_type(builder)
    }

    // 获取 void type
    pub fn new_void(&mut self) -> TypeKey {
        let build = TypeBuilder::BuildIn(BuildInType::Void);
        self.build_type(build)
    }

    // 字符串类型
    pub fn new_string(&mut self, sz: usize) -> TypeKey {
        // c 的 string 似乎不是 const 类型
        let char_ty = self.new_char(CharSign::Plain);
        let elem_ty = QualType::from(char_ty);
        let size = ArraySize::Static(sz);
        let array_ty = ArrayType { elem_ty, size };
        let builder = TypeBuilder::Array(array_ty);

        self.build_type(builder)
    }

    pub fn new_ptr(&mut self, elem_ty: TypeKey) -> TypeKey {
        let elem_ty = QualType::from(elem_ty);
        let ptr_ty = PtrType { elem_ty };
        let builder = TypeBuilder::Ptr(ptr_ty);
        self.build_type(builder)
    }

    fn next_record_id(&mut self) -> RecordID {
        let record_id = RecordID(self.record_counter);
        self.record_counter += 1;
        record_id
    }

    fn next_enum_id(&mut self) -> EnumID {
        let enum_id = EnumID(self.enum_counter);
        self.enum_counter += 1;
        enum_id
    }

    /// 构建一个空 record 对象，分配一个 record id
    pub fn new_record(&mut self, kind: RecordKind) -> TypeKey {
        let builder = self.new_record_builder(kind);
        self.build_type(builder)
    }

    /// 构建一个 record builder kind，分配一个 record id
    pub fn new_record_builder(&mut self, kind: RecordKind) -> TypeBuilder {
        let record_id = self.next_record_id();
        let tag = TagTypeBuilder::Record {
            kind,
            id: record_id,
        };
        TypeBuilder::Tag(tag)
    }

    /// 构建一个空 enum 对象，分配一个 enum id
    pub fn new_enum(&mut self) -> TypeKey {
        let builder = self.new_enum_builder();
        self.build_type(builder)
    }

    /// 构建一个全新的 enum builder kind，分配一个 enum_id
    pub fn new_enum_builder(&mut self) -> TypeBuilder {
        let enum_id = self.next_enum_id();
        let tag = TagTypeBuilder::Enum { id: enum_id };
        TypeBuilder::Tag(tag)
    }
}
