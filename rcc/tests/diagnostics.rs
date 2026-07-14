use rcc::{ExpressionKind, ExternalDeclaration, Initializer, compile};

#[test]
fn compilation_keeps_source_handles_resolvable() {
    let compilation = compile("int value = 42;\n").unwrap();
    let ExternalDeclaration::Declaration(declaration) = &compilation.ast.declarations[0] else {
        panic!()
    };
    let Some(Initializer::Expression(expression)) = &declaration.initializer else {
        panic!()
    };
    assert!(matches!(expression.kind, ExpressionKind::Integer(42)));
    let location = compilation
        .source_manager
        .presumed_location(expression.range.begin)
        .unwrap();
    assert_eq!((location.filename.as_str(), location.line), ("<memory>", 1));
}

#[test]
fn diagnostics_render_presumed_locations_and_macro_expansion_notes() {
    let failure =
        compile("#define BAD missing\n#line 42 \"virtual.c\"\nint f(void) { return BAD; }\n")
            .unwrap_err();
    let diagnostic = failure
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("undeclared identifier"))
        .unwrap();
    let rendered = diagnostic.render(&failure.source_manager);
    assert!(rendered.contains("virtual.c:42:"), "{rendered}");
    assert!(rendered.contains("expanded from macro 'BAD'"), "{rendered}");
    assert!(rendered.contains("return BAD;"), "{rendered}");
}
