# rcc C11 front end

`rcc` exposes a typed C11 abstract syntax tree through the original compiler
crate structure. Each compiler phase owns its domain model:

- `lex` owns preprocessing, tokens, and lexical analysis.
- `parser` owns semantic state and the typed AST.
- `types` contains only source positions shared across phases.
- `err` owns diagnostics, `compiler` coordinates the pipeline, and `writer`
  serializes compiler output.

```rust
let ast = rcc::compile("int main(void) { return 0; }")?;
let ast = rcc::compile_file("program.c")?;

let compiler = rcc::CCompiler::new("int answer = 42;");
let ast = compiler.compile()?;
```

`compile` preprocesses an in-memory translation unit. `compile_file` also
resolves quoted includes relative to the including file and system includes
through `CPATH` and common platform include directories. Both return all
lexical, syntax, or semantic diagnostics accumulated during recovery.

The AST preserves source spans, C types and qualifiers, storage and function
specifiers, value categories, declarations, initializers, statements, and
expressions. C11-specific nodes include generic selections, atomic-qualified
types, alignment, static assertions, thread-local storage, complex types,
Unicode literals, compound literals, designated initializers, and variable
length arrays. Standard C11 mode is the default; GNU-only syntax is rejected.

The previous AP-number and content-manager prototypes were not retained as
dead compatibility code: both were incomplete and were not connected to the
working parser. Arbitrary-precision constant values and a multi-file source
manager remain explicit follow-up subsystems; current C11 integer semantics use
the target's supported integer widths and spans refer to preprocessed source.

Run the CLI to print the complete typed AST:

```text
cargo run -p rcc -- rcc/resources/example/c11.c
```
