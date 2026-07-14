use rcc::{AstWriter, compile_file};
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: rcc <source.c>");
        return ExitCode::from(2);
    };
    match compile_file(&path) {
        Ok(ast) => {
            print!("{}", AstWriter::render(&ast));
            ExitCode::SUCCESS
        }
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{path}:{diagnostic}");
            }
            ExitCode::FAILURE
        }
    }
}
