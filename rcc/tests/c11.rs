use rcc::{ExpressionKind, ExternalDeclaration, StatementKind, TypeKind, compile};

fn parse(source: &str) -> rcc::TranslationUnit {
    compile(source)
        .unwrap_or_else(|errors| panic!("{errors:#?}"))
        .ast
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
    let ExpressionKind::ImplicitCast { expression, .. } = &expr.kind else {
        panic!()
    };
    assert!(matches!(
        expression.kind,
        ExpressionKind::GenericSelection { .. }
    ));
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
fn rejects_invalid_derived_types_without_panicking() {
    for source in [
        "void values[2];",
        "int function(void)[2];",
        "_Atomic(int (void)) value;",
    ] {
        let errors = compile(source).unwrap_err();
        assert!(!errors.diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn validates_c11_declaration_specifier_combinations() {
    for source in [
        "short long value;",
        "long long long value;",
        "signed unsigned value;",
        "int int value;",
        "auto int file_object;",
        "_Thread_local int function(void);",
        "typedef int Alias = 1;",
        "inline int object;",
        "struct S { static int field; };",
        "int function(static int parameter);",
    ] {
        assert!(compile(source).is_err(), "{source}");
    }
    parse("char unsigned byte; long double real; const const int qualified;");
}

#[test]
fn validates_record_members_bitfields_and_flexible_arrays() {
    parse(
        "struct Bits { unsigned int flag : 1; unsigned int : 0; int values[]; }; struct Outer { struct { int promoted; }; }; int f(void) { struct Outer value; return value.promoted; }",
    );
    for source in [
        "struct Bad { int named : 0; };",
        "struct Bad { int wide : 100; };",
        "struct Bad { double field : 1; };",
        "struct Bad { int duplicate; int duplicate; };",
        "struct Node { struct Node child; };",
        "struct Bad { int values[]; };",
        "union Bad { int fixed; int values[]; };",
        "struct Bad { int values[]; int after; };",
    ] {
        assert!(compile(source).is_err(), "{source}");
    }
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

#[test]
fn typed_ast_preserves_implicit_c_conversions() {
    let ast = parse("int left; short right; int value = left + right;");
    let ExternalDeclaration::Declaration(value) = &ast.declarations[2] else {
        panic!()
    };
    let Some(rcc::Initializer::Expression(expression)) = &value.initializer else {
        panic!()
    };
    let ExpressionKind::Binary { left, right, .. } = &expression.kind else {
        panic!()
    };
    assert!(matches!(
        left.kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::LValueToRValue,
            ..
        }
    ));
    assert!(matches!(
        right.kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::IntegralPromotion,
            ..
        }
    ));
    let ExpressionKind::ImplicitCast { expression, .. } = &right.kind else {
        panic!()
    };
    assert!(matches!(
        expression.kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::LValueToRValue,
            ..
        }
    ));
}

#[test]
fn typed_ast_records_decay_control_and_conditional_conversions() {
    let ast = parse(
        "short condition; int array[2]; int function(int); long selected = condition ? +condition : 1L; int *pointer = array; int (*callback)(int) = function;",
    );
    let initializer = |index: usize| {
        let ExternalDeclaration::Declaration(declaration) = &ast.declarations[index] else {
            panic!()
        };
        let Some(rcc::Initializer::Expression(expression)) = &declaration.initializer else {
            panic!()
        };
        expression
    };
    let ExpressionKind::Conditional {
        condition,
        then_expr,
        ..
    } = &initializer(3).kind
    else {
        panic!()
    };
    assert!(matches!(
        condition.kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::LValueToRValue,
            ..
        }
    ));
    assert!(matches!(
        then_expr.kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::IntegralConversion,
            ..
        }
    ));
    assert!(matches!(
        initializer(4).kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::ArrayToPointerDecay,
            ..
        }
    ));
    let ExpressionKind::ImplicitCast {
        kind: rcc::ImplicitCastKind::PointerConversion,
        expression,
    } = &initializer(5).kind
    else {
        panic!()
    };
    assert!(matches!(
        expression.kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::FunctionToPointerDecay,
            ..
        }
    ));
}

#[test]
fn calls_apply_default_argument_promotions() {
    let ast = parse(
        "int old_style(); int variadic(int, ...); int first = old_style(1.0f); int second = variadic(0, 1.0f);",
    );
    let call = |index: usize| {
        let ExternalDeclaration::Declaration(declaration) = &ast.declarations[index] else {
            panic!()
        };
        let Some(rcc::Initializer::Expression(expression)) = &declaration.initializer else {
            panic!()
        };
        let ExpressionKind::Call { arguments, .. } = &expression.kind else {
            panic!()
        };
        arguments
    };
    assert!(matches!(
        call(2)[0].kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::FloatingConversion,
            ..
        }
    ));
    assert!(matches!(
        call(3)[1].kind,
        ExpressionKind::ImplicitCast {
            kind: rcc::ImplicitCastKind::FloatingConversion,
            ..
        }
    ));
    assert!(matches!(call(2)[0].ty.kind, TypeKind::Double));
}

#[test]
fn tag_types_use_declaration_identity() {
    let ast = parse(
        "struct Item; struct Item { int value; }; struct Item item; struct { int value; } a; struct { int value; } b;",
    );
    let ids = ast
        .declarations
        .iter()
        .filter_map(|declaration| {
            let ExternalDeclaration::Declaration(declaration) = declaration else {
                return None;
            };
            match declaration.ty.kind {
                TypeKind::Struct { id, .. } => Some(id),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(ids[0], ids[1]);
    assert_eq!(ids[1], ids[2]);
    assert_ne!(ids[3], ids[4]);
}

#[test]
fn tag_namespace_supports_self_reference_shadowing_and_kind_checks() {
    let ast = parse(
        "struct Node { struct Node *next; }; int f(void) { struct Node { double value; } local; return 0; } struct Node global;",
    );
    let ExternalDeclaration::Declaration(outer) = &ast.declarations[0] else {
        panic!()
    };
    let TypeKind::Struct {
        id: outer_id,
        fields: Some(fields),
        ..
    } = &outer.ty.kind
    else {
        panic!()
    };
    let TypeKind::Pointer(pointee) = &fields[0].ty.kind else {
        panic!()
    };
    assert!(matches!(pointee.kind, TypeKind::Struct { id, .. } if id == *outer_id));

    let ExternalDeclaration::Function(function) = &ast.declarations[1] else {
        panic!()
    };
    let StatementKind::Compound(items) = &function.body.kind else {
        panic!()
    };
    let rcc::BlockItem::Declaration(local) = &items[0] else {
        panic!()
    };
    let TypeKind::Struct { id: local_id, .. } = local.ty.kind else {
        panic!()
    };
    assert_ne!(local_id, *outer_id);

    let ExternalDeclaration::Declaration(global) = &ast.declarations[2] else {
        panic!()
    };
    assert!(matches!(global.ty.kind, TypeKind::Struct { id, .. } if id == *outer_id));
    assert!(compile("struct Conflict; union Conflict;").is_err());
}

#[test]
fn identifier_expressions_are_bound_to_declaration_ids() {
    let ast = parse("int source; int value = source;");
    let ExternalDeclaration::Declaration(source) = &ast.declarations[0] else {
        panic!()
    };
    let ExternalDeclaration::Declaration(value) = &ast.declarations[1] else {
        panic!()
    };
    let Some(rcc::Initializer::Expression(expression)) = &value.initializer else {
        panic!()
    };
    let ExpressionKind::ImplicitCast { expression, .. } = &expression.kind else {
        panic!()
    };
    assert!(matches!(
        expression.kind,
        ExpressionKind::Identifier { declaration, .. } if declaration == source.id
    ));
    assert_eq!(std::mem::size_of::<rcc::DeclId>(), 4);
}

#[test]
fn declarations_record_context_linkage_and_storage_duration() {
    let ast = parse(
        "static int internal; extern int external; _Thread_local int tls; int f(int p) { static int local; int automatic; return p; }",
    );
    let declarations = ast
        .declarations
        .iter()
        .take(3)
        .map(|declaration| match declaration {
            ExternalDeclaration::Declaration(declaration) => declaration,
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations[0].linkage, rcc::Linkage::Internal);
    assert_eq!(declarations[1].linkage, rcc::Linkage::External);
    assert_eq!(
        declarations[2].storage_duration,
        rcc::StorageDuration::Thread
    );

    let ExternalDeclaration::Function(function) = &ast.declarations[3] else {
        panic!()
    };
    assert_eq!(function.parameters[0].context, function.body_context);
    let StatementKind::Compound(items) = &function.body.kind else {
        panic!()
    };
    let rcc::BlockItem::Declaration(local) = &items[0] else {
        panic!()
    };
    let rcc::BlockItem::Declaration(automatic) = &items[1] else {
        panic!()
    };
    assert_eq!(local.context, function.body_context);
    assert_eq!(local.storage_duration, rcc::StorageDuration::Static);
    assert_eq!(automatic.storage_duration, rcc::StorageDuration::Automatic);
}

#[test]
fn compatible_redeclarations_form_a_decl_chain() {
    let ast = parse("int function(int); int function(int value) { return value; }");
    let ExternalDeclaration::Declaration(prototype) = &ast.declarations[0] else {
        panic!()
    };
    let ExternalDeclaration::Function(definition) = &ast.declarations[1] else {
        panic!()
    };
    assert_eq!(definition.previous_declaration, Some(prototype.id));

    let errors = compile("int function(int); int function(double);").unwrap_err();
    assert!(
        errors
            .iter()
            .any(|diagnostic| diagnostic.message.contains("incompatible redeclaration"))
    );
}

#[test]
fn rejects_duplicate_definitions_and_block_scope_redefinitions() {
    for source in [
        "int value = 1; int value = 2;",
        "int function(void) { return 1; } int function(void) { return 2; }",
        "int function(int parameter) { int parameter; return parameter; }",
        "int function(void) { int local; int local; return 0; }",
    ] {
        let errors = compile(source).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|diagnostic| diagnostic.message.contains("redefinition")),
            "{source}: {errors:#?}"
        );
    }
    parse("int tentative; int tentative; extern int object; int object = 1;");
}

#[test]
fn validates_function_labels_and_switch_labels() {
    parse("int f(void) { goto done; done: return 0; }");
    for source in [
        "int f(void) { goto missing; return 0; }",
        "int f(void) { label: ; label: return 0; }",
        "int f(int x) { switch (x) { case 1: ; case 1: return 0; } }",
        "int f(int x) { switch (x) { default: ; default: return 0; } }",
        "int f(void) { case 1: return 0; }",
    ] {
        assert!(compile(source).is_err(), "{source}");
    }
}

#[test]
fn semantic_state_is_restored_when_function_parsing_recovers() {
    let errors = compile(
        "int first(void) { return missing_first; } int second(void) { return missing_second; }",
    )
    .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|diagnostic| diagnostic.message.contains("missing_first"))
    );
    assert!(
        errors
            .iter()
            .any(|diagnostic| diagnostic.message.contains("missing_second"))
    );
}
