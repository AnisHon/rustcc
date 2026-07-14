use crate::parser::TranslationUnit;
use std::io::{self, Write};

pub struct AstWriter;

impl AstWriter {
    pub fn render(ast: &TranslationUnit) -> String {
        format!("{ast:#?}")
    }

    pub fn write(ast: &TranslationUnit, mut output: impl Write) -> io::Result<()> {
        writeln!(output, "{}", Self::render(ast))
    }
}
