use rcc::SourceManager;
use rcc::lex::{PPTokenKind, Preprocessor};

#[test]
fn preprocessor_expands_token_macros_and_records_expansion_locations() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer(
            "macro.c",
            "#define ADD(x, y) x + y\n#if 1\nADD(2, 3)\n#endif\n",
        )
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    let mut tokens = Vec::new();
    loop {
        let token = preprocessor.next_token().unwrap();
        let eof = token.kind == PPTokenKind::EndOfFile;
        tokens.push(token);
        if eof {
            break;
        }
    }
    let expansion_location = tokens[0].range.begin;
    assert_eq!(
        tokens
            .iter()
            .map(|token| token.spelling.as_str())
            .collect::<Vec<_>>(),
        ["2", "+", "3", ""]
    );
    drop(preprocessor);
    let frames = sources.expansion_stack(expansion_location).unwrap();
    assert_eq!(frames[0].macro_name, "ADD");
    assert_eq!(
        sources
            .presumed_location(sources.expansion_location_of(expansion_location).unwrap())
            .unwrap()
            .line,
        3
    );
}

#[test]
fn inactive_include_does_not_open_a_file() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer(
            "conditional.c",
            "#if 0\n#include \"missing.h\"\n#endif\nint\n",
        )
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    let token = preprocessor.next_token().unwrap();
    assert_eq!(
        (token.kind, token.spelling.as_str()),
        (PPTokenKind::Identifier, "int")
    );
}

#[test]
fn evaluates_c11_preprocessing_integer_expressions() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer(
            "condition.c",
            "#define VERSION 11\n#if defined(VERSION) && (VERSION * 2 + 1 == 23) && ((8 >> 2) == 2)\naccepted\n#else\nrejected\n#endif\n",
        )
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    assert_eq!(preprocessor.next_token().unwrap().spelling, "accepted");
}

#[test]
fn stringifies_and_pastes_macro_arguments_as_tokens() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer(
            "operators.c",
            "#define STR(x) #x\n#define CAT(x, y) x ## y\nSTR(a   + b) CAT(long, long) CAT(, value)\n",
        )
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    let mut spellings = Vec::new();
    loop {
        let token = preprocessor.next_token().unwrap();
        if token.kind == PPTokenKind::EndOfFile {
            break;
        }
        spellings.push(token.spelling);
    }
    assert_eq!(spellings, ["\"a + b\"", "longlong", "value"]);
}

#[test]
fn expands_function_macros_in_if_expressions_without_expanding_defined_operands() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer(
            "directive.c",
            "#define VALUE 3\n#define TWICE(x) ((x) * 2)\n#if defined(VALUE) && TWICE(VALUE) == 6\nyes\n#else\nno\n#endif\n",
        )
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    assert_eq!(preprocessor.next_token().unwrap().spelling, "yes");
}

#[test]
fn include_operand_is_macro_expanded() {
    let directory = std::env::temp_dir().join(format!("rcc-pp-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let header = directory.join("value.h");
    std::fs::write(&header, "included\n").unwrap();
    let main = directory.join("main.c");
    std::fs::write(&main, "#define HEADER \"value.h\"\n#include HEADER\n").unwrap();

    let mut sources = SourceManager::new();
    let file = sources
        .add_file(&main, rcc::SourceLocation::INVALID)
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    assert_eq!(preprocessor.next_token().unwrap().spelling, "included");
    drop(preprocessor);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn permits_equivalent_macro_redefinitions_and_rejects_different_ones() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer("same.c", "#define F(x) x + 1\n#define F(y) y + 1\nF(2)\n")
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    assert_eq!(preprocessor.next_token().unwrap().spelling, "2");

    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer("different.c", "#define X 1\n#define X 2\nX\n")
        .unwrap();
    let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
    assert!(
        preprocessor
            .next_token()
            .unwrap_err()
            .message
            .contains("redefinition")
    );
}

#[test]
fn rejects_malformed_c11_directives() {
    for source in [
        "#if (1 + )\n#endif\n",
        "#ifdef X extra\n#endif\n",
        "#define F(x, x) x\n",
        "#line 0\nvalue\n",
        "#if 1\n#else extra\n#endif\n",
    ] {
        let mut sources = SourceManager::new();
        let file = sources.add_memory_buffer("bad.c", source).unwrap();
        let mut preprocessor = Preprocessor::new(&mut sources, file).unwrap();
        assert!(preprocessor.next_token().is_err(), "{source}");
    }
}
