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
