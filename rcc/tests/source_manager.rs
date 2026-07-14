use rcc::{SourceLocation, SourceManager, SourceRange};

#[test]
fn locations_are_compact_handles_and_ranges_are_half_open() {
    assert_eq!(std::mem::size_of::<SourceLocation>(), 4);
    let mut sources = SourceManager::new();
    let file = sources.add_memory_buffer("main.c", "one\ntwo\n").unwrap();
    let begin = sources.file_location(file, 4).unwrap();
    let end = sources.file_location(file, 7).unwrap();
    let range = SourceRange::new(begin, end);
    assert_eq!(sources.file_position(range.begin).unwrap().byte_offset, 4);
    assert_eq!(sources.file_position(range.end).unwrap().byte_offset, 7);
    let presumed = sources.presumed_location(begin).unwrap();
    assert_eq!(
        (presumed.filename.as_str(), presumed.line, presumed.column),
        ("main.c", 2, 1)
    );
}

#[test]
fn line_directives_include_stack_and_macro_locations_are_resolved() {
    let mut sources = SourceManager::new();
    let main = sources
        .add_memory_buffer("main.c", "#include \"a.h\"\nUSE\n")
        .unwrap();
    let include_location = sources.file_location(main, 0).unwrap();
    let header = sources
        .add_memory_buffer("a.h", "#define VALUE 42\n")
        .unwrap();

    // Re-add through a file-like child buffer to record an include parent.
    let header_text = sources.buffer(header).unwrap().to_string();
    let child = sources
        .add_included_buffer("generated-a.h", header_text, include_location)
        .unwrap();
    assert_eq!(
        sources.include_stack(child).unwrap()[0].include_location,
        include_location
    );

    sources
        .add_line_directive(main, 17, 200, Some("virtual.c".into()))
        .unwrap();
    let use_location = sources.file_location(main, 17).unwrap();
    let presumed = sources.presumed_location(use_location).unwrap();
    assert_eq!(
        (presumed.filename.as_str(), presumed.line),
        ("virtual.c", 200)
    );

    let spelling = sources.file_location(header, 14).unwrap();
    let expansion = sources
        .expansion_location(
            spelling,
            SourceRange::new(use_location, use_location),
            "VALUE",
        )
        .unwrap();
    assert_eq!(sources.spelling_location(expansion).unwrap(), spelling);
    assert_eq!(
        sources.expansion_location_of(expansion).unwrap(),
        use_location
    );
    assert_eq!(
        sources.expansion_stack(expansion).unwrap()[0].macro_name,
        "VALUE"
    );
}
