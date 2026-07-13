# rcc C11 front end

`rcc` exposes a typed C11 abstract syntax tree. The old generated-parser
prototype remains in the repository for reference but is not part of the
crate's compilation graph.

```rust
let ast = rcc::compile("int main(void) { return 0; }")?;
let ast = rcc::compile_file("program.c")?;
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

Run the CLI to print the complete typed AST:

```text
cargo run -p rcc -- rcc/resources/example/c11.c
```
