use rcc::{AstWriter, compile_file};
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: rcc <source.c>");
        return ExitCode::from(2);
    };
    match compile_file(&path) {
        Ok(compilation) => {
            print!("{}", AstWriter::render(&compilation.ast));
            ExitCode::SUCCESS
        }
        Err(diagnostics) => {
            for diagnostic in &diagnostics.diagnostics {
                eprintln!("{}", diagnostic.render(&diagnostics.source_manager));
            }
            ExitCode::FAILURE
        }
    }
}
