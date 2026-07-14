use rcc::{ExpressionKind, ExternalDeclaration, StatementKind, TypeKind, compile};

fn parse(source: &str) -> rcc::TranslationUnit {
    compile(source).unwrap_or_else(|errors| panic!("{errors:#?}"))
}

#[test]
fn parses_and_types_a_translation_unit() {
    let ast = parse(
        r#"
        typedef unsigned long size_t;
        struct Point { int x; int y; };
        int sum(const int *values, size_t count);
        int main(void) {
            struct Point p = { .x = 2, .y = 3 };
            int values[2] = { p.x, p.y };
            int *cursor = values;
            return cursor[0] + *(cursor + 1);
        }
    "#,
    );
    assert_eq!(ast.declarations.len(), 4);
    let ExternalDeclaration::Function(main) = &ast.declarations[3] else {
        panic!()
    };
    let StatementKind::Compound(items) = &main.body.kind else {
        panic!()
    };
    let rcc::BlockItem::Statement(ret) = items.last().unwrap() else {
        panic!()
    };
    let StatementKind::Return(Some(expr)) = &ret.kind else {
        panic!()
    };
    assert!(matches!(expr.ty.kind, TypeKind::Int { signed: true }));
}

#[test]
fn preserves_c_declarator_binding() {
    let ast = parse("int *returns_pointer(void); int (*pointer_to_function)(int);");
    let ExternalDeclaration::Declaration(first) = &ast.declarations[0] else {
        panic!()
    };
    let TypeKind::Function { return_type, .. } = &first.ty.kind else {
        panic!()
    };
    assert!(matches!(return_type.kind, TypeKind::Pointer(_)));
    let ExternalDeclaration::Declaration(second) = &ast.declarations[1] else {
        panic!()
    };
    let TypeKind::Pointer(pointee) = &second.ty.kind else {
        panic!()
    };
    assert!(matches!(pointee.kind, TypeKind::Function { .. }));
}

#[test]
fn supports_c11_generic_compound_literal_and_static_assert() {
    let ast = parse(
        r#"
        struct Pair { int x; int y; };
        _Static_assert(_Alignof(int) >= 1, "alignment");
        int main(void) {
            struct Pair p = (struct Pair){ .x = 1, .y = 2 };
            return _Generic(p.x, int: p.x, default: 0);
        }
    "#,
    );
    assert!(matches!(
        ast.declarations[1],
        ExternalDeclaration::StaticAssert(_)
    ));
    let ExternalDeclaration::Function(main) = &ast.declarations[2] else {
        panic!()
    };
    let StatementKind::Compound(items) = &main.body.kind else {
        panic!()
    };
    let rcc::BlockItem::Statement(ret) = items.last().unwrap() else {
        panic!()
    };
    let StatementKind::Return(Some(expr)) = &ret.kind else {
        panic!()
    };
    assert!(matches!(expr.kind, ExpressionKind::GenericSelection { .. }));
}

#[test]
fn reports_semantic_errors() {
    let errors = compile("int main(void) { const int x = 1; x = 2; return missing; }").unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("modifiable lvalue"))
    );
}

#[test]
fn preprocesses_c11_macros_and_conditionals() {
    let ast = parse(
        r#"
        #define SUM(a, b) ((a) + (b))
        #define ENABLED 1
        #define CAT(a, b) a ## b
        #define TEXT(x) #x
        #define FIRST(x, ...) x
        #if defined(ENABLED) && (3 * 4 == 12)
        int CAT(val, ue) = SUM(2, 3);
        char *text = TEXT(hello world);
        int first = FIRST(7, 8, 9);
        #else
        invalid branch
        #endif
    "#,
    );
    assert_eq!(ast.declarations.len(), 3);
    let ExternalDeclaration::Declaration(value) = &ast.declarations[0] else {
        panic!()
    };
    let rcc::Initializer::Expression(expression) = value.initializer.as_ref().unwrap() else {
        panic!()
    };
    assert!(matches!(expression.kind, ExpressionKind::Binary { .. }));
}

#[test]
fn file_entry_resolves_active_includes_only() {
    let directory = std::env::temp_dir().join(format!("rcc-c11-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("value.h"), "#define HEADER_VALUE 42\n").unwrap();
    std::fs::write(
        directory.join("main.c"),
        "#if 0\n#include \"missing.h\"\n#endif\n#include \"value.h\"\nint value = HEADER_VALUE;\n",
    )
    .unwrap();
    let ast = rcc::compile_file(directory.join("main.c")).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn types_vlas_hex_floats_enums_and_c11_storage() {
    let ast = parse(
        r#"
        enum Scale { BASE = 2, FACTOR = BASE * 3 };
        _Static_assert(FACTOR == 6, "enum constants");
        _Thread_local _Atomic int shared;
        double hex_value = 0x1.8p+1;
        int work(int count) {
            double values[count];
            return count;
        }
    "#,
    );
    let ExternalDeclaration::Declaration(shared) = &ast.declarations[2] else {
        panic!()
    };
    assert_eq!(shared.storage, rcc::StorageClass::ThreadLocal);
    assert!(shared.ty.qualifiers.is_atomic);
    let ExternalDeclaration::Function(work) = &ast.declarations[4] else {
        panic!()
    };
    let StatementKind::Compound(items) = &work.body.kind else {
        panic!()
    };
    let rcc::BlockItem::Declaration(values) = &items[0] else {
        panic!()
    };
    assert!(matches!(
        values.ty.kind,
        TypeKind::Array {
            size: rcc::ArraySize::Variable(_),
            ..
        }
    ));
}

#[test]
fn preserves_wide_literals_and_old_style_function_types() {
    let ast = parse(
        r#"
        unsigned short text[] = u"hello";
        int add(left, right)
            int left;
            short right;
        { return left + right; }
    "#,
    );
    let ExternalDeclaration::Declaration(text) = &ast.declarations[0] else {
        panic!()
    };
    let Some(rcc::Initializer::Expression(expression)) = &text.initializer else {
        panic!()
    };
    assert!(matches!(
        expression.kind,
        ExpressionKind::String {
            encoding: rcc::StringEncoding::Utf16,
            ..
        }
    ));
    let ExternalDeclaration::Function(add) = &ast.declarations[1] else {
        panic!()
    };
    let TypeKind::Function {
        has_prototype,
        params,
        ..
    } = &add.ty.kind
    else {
        panic!()
    };
    assert!(!has_prototype);
    assert!(matches!(params[1].ty.kind, TypeKind::Short { .. }));
}
