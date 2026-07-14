use rcc::SourceManager;
use rcc::lex::{PPTokenKind, Punctuator, RawLexer};

#[test]
fn raw_lexer_produces_preprocessing_tokens_with_source_handles() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer("raw.c", "int value = 0x1.fp+2;\n#define X value\n")
        .unwrap();
    let mut lexer = RawLexer::new(&mut sources, file).unwrap();
    let keyword = lexer.next_token().unwrap();
    assert_eq!(keyword.kind, PPTokenKind::Identifier);
    assert_eq!(keyword.spelling, "int");
    assert!(keyword.start_of_line);
    let identifier = lexer.next_token().unwrap();
    assert_eq!(identifier.kind, PPTokenKind::Identifier);
    assert!(identifier.leading_space);
    assert_eq!(
        lexer.next_token().unwrap().kind,
        PPTokenKind::Punctuator(Punctuator::Assign)
    );
    assert_eq!(lexer.next_token().unwrap().spelling, "0x1.fp+2");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        PPTokenKind::Punctuator(Punctuator::Semi)
    );
    assert_eq!(lexer.next_token().unwrap().kind, PPTokenKind::NewLine);
    let hash = lexer.next_token().unwrap();
    assert!(hash.start_of_line);
    assert_eq!(hash.kind, PPTokenKind::Punctuator(Punctuator::Hash));
    drop(lexer);
    let location = sources.file_position(keyword.range.begin).unwrap();
    assert_eq!((location.file_id, location.byte_offset), (file, 0));
}

#[test]
fn block_comments_preserve_directive_newlines() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer("comment.c", "a/* first\nsecond */b")
        .unwrap();
    let mut lexer = RawLexer::new(&mut sources, file).unwrap();
    assert_eq!(lexer.next_token().unwrap().spelling, "a");
    assert_eq!(lexer.next_token().unwrap().kind, PPTokenKind::NewLine);
    let b = lexer.next_token().unwrap();
    assert_eq!(b.spelling, "b");
    assert!(b.leading_space);
}

#[test]
fn unknown_characters_are_invalid_instead_of_becoming_dots() {
    let mut sources = SourceManager::new();
    let file = sources.add_memory_buffer("invalid.c", "`\n").unwrap();
    let mut lexer = RawLexer::new(&mut sources, file).unwrap();
    assert_eq!(lexer.next_token().unwrap().kind, PPTokenKind::Invalid);
}

#[test]
fn encoding_prefixes_are_part_of_literal_tokens() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer("literal.c", "u\"text\" L'x'\n")
        .unwrap();
    let mut lexer = RawLexer::new(&mut sources, file).unwrap();
    let string = lexer.next_token().unwrap();
    let character = lexer.next_token().unwrap();
    assert_eq!(
        (string.kind, string.spelling.as_str()),
        (PPTokenKind::String, "u\"text\"")
    );
    assert_eq!(
        (character.kind, character.spelling.as_str()),
        (PPTokenKind::Character, "L'x'")
    );
}

#[test]
fn applies_trigraphs_before_line_splicing_without_losing_physical_ranges() {
    let mut sources = SourceManager::new();
    let file = sources
        .add_memory_buffer("phases.c", "hel??/\nlo ??= define\n")
        .unwrap();
    let mut lexer = RawLexer::new(&mut sources, file).unwrap();
    let identifier = lexer.next_token().unwrap();
    let hash = lexer.next_token().unwrap();
    drop(lexer);
    assert_eq!(identifier.spelling, "hello");
    assert_eq!(
        sources
            .file_position(identifier.range.end)
            .unwrap()
            .byte_offset,
        9
    );
    assert_eq!(
        (hash.kind, hash.spelling.as_str()),
        (PPTokenKind::Punctuator(rcc::lex::Punctuator::Hash), "#")
    );
}
