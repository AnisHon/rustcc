use rcc::{
    ArrayBound, BuiltinType, FunctionType, QualType, Qualifiers, RecordKind, TypeContext,
    TypeError, TypeId,
};

#[test]
fn type_ids_are_compact_and_structural_types_are_interned() {
    assert_eq!(size_of::<TypeId>(), 4);

    let mut types = TypeContext::new();
    let int = types.builtin(BuiltinType::Int);
    let const_int = int.with_qualifiers(Qualifiers::CONST);
    let first = types.pointer(const_int);
    let second = types.pointer(const_int);

    assert_eq!(first, second);
    assert_ne!(types.pointer(int), first);
    assert!(types.compatible(int, const_int));
    assert_eq!(types.canonical(const_int), const_int);
}

#[test]
fn tags_have_declaration_identity_and_constructors_enforce_c_constraints() {
    let mut types = TypeContext::new();
    let int = types.builtin(BuiltinType::Int);
    let void = types.builtin(BuiltinType::Void);
    let record_a = types.fresh_record(RecordKind::Struct);
    let record_b = types.fresh_record(RecordKind::Struct);
    assert_ne!(record_a, record_b);

    assert_eq!(
        types.array(void, ArrayBound::Constant(1)),
        Err(TypeError::ArrayOfVoid)
    );
    let array = types.array(int, ArrayBound::Constant(4)).unwrap();
    let function = types
        .function(FunctionType {
            result: int,
            parameters: vec![int],
            variadic: false,
            has_prototype: true,
            calling_convention: Default::default(),
        })
        .unwrap();
    assert_eq!(
        types.array(function, ArrayBound::Incomplete),
        Err(TypeError::ArrayOfFunction)
    );
    assert_eq!(
        types.function(FunctionType {
            result: array,
            parameters: Vec::new(),
            variadic: false,
            has_prototype: true,
            calling_convention: Default::default(),
        }),
        Err(TypeError::FunctionReturnsArray)
    );
    assert_eq!(
        types.atomic(QualType::unqualified(function.ty)),
        Err(TypeError::AtomicFunction)
    );
    assert_eq!(
        types.atomic(int.with_qualifiers(Qualifiers::CONST)),
        Err(TypeError::AtomicQualified)
    );
}

#[test]
fn compilation_attaches_canonical_type_ids_to_ast_types() {
    let compilation = rcc::compile("const int *left; const int *right;").unwrap();
    let declarations = &compilation.ast.declarations;
    let rcc::ExternalDeclaration::Declaration(left) = &declarations[0] else {
        panic!()
    };
    let rcc::ExternalDeclaration::Declaration(right) = &declarations[1] else {
        panic!()
    };
    assert_ne!(left.ty.canonical.ty, TypeId::INVALID);
    assert_eq!(left.ty.canonical, right.ty.canonical);
    assert!(matches!(
        compilation.types.kind(left.ty.canonical.ty),
        rcc::CanonicalTypeKind::Pointer(_)
    ));
}
