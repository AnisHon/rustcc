use super::{Diagnostic, ErrorKind, Position, Span};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Macro {
    params: Option<Vec<String>>,
    variadic: bool,
    replacement: String,
}

#[derive(Clone, Copy)]
struct Conditional {
    parent_active: bool,
    active: bool,
    branch_taken: bool,
    saw_else: bool,
}

/// Run C11 translation phases one through four.
///
/// Includes need a file-aware source provider and are rejected by this
/// string-only entry point instead of being silently ignored.
pub fn preprocess(source: &str) -> Result<String, Vec<Diagnostic>> {
    Preprocessor::new(source, None).run()
}

/// Preprocess a file after recursively resolving `#include` directives.
pub fn preprocess_file(path: &Path) -> Result<String, Vec<Diagnostic>> {
    let canonical = path.canonicalize().map_err(|error| {
        vec![file_error(format!(
            "cannot open {}: {error}",
            path.display()
        ))]
    })?;
    let source = std::fs::read_to_string(&canonical).map_err(|error| {
        vec![file_error(format!(
            "cannot read {}: {error}",
            canonical.display()
        ))]
    })?;
    Preprocessor::new(&source, canonical.parent().map(Path::to_path_buf)).run()
}

struct Preprocessor {
    source: String,
    macros: HashMap<String, Macro>,
    diagnostics: Vec<Diagnostic>,
    base: Option<PathBuf>,
    include_stack: Vec<PathBuf>,
}

impl Preprocessor {
    fn new(source: &str, base: Option<PathBuf>) -> Self {
        let mut macros = HashMap::new();
        for (name, replacement) in [
            ("__STDC__", "1"),
            ("__STDC_VERSION__", "201112L"),
            ("__STDC_HOSTED__", "1"),
        ] {
            macros.insert(
                name.to_string(),
                Macro {
                    params: None,
                    variadic: false,
                    replacement: replacement.to_string(),
                },
            );
        }
        Self {
            source: source.to_string(),
            macros,
            diagnostics: vec![],
            base,
            include_stack: vec![],
        }
    }

    fn run(mut self) -> Result<String, Vec<Diagnostic>> {
        let mut output = String::new();
        let source = std::mem::take(&mut self.source);
        let base = self.base.clone();
        self.process_source(&source, base.as_deref(), &mut output);
        if self.diagnostics.is_empty() {
            Ok(output)
        } else {
            Err(self.diagnostics)
        }
    }

    fn process_source(&mut self, source: &str, base: Option<&Path>, output: &mut String) {
        let source = remove_comments(&splice_lines(&replace_ucns(&replace_trigraphs(source))));
        let mut conditionals: Vec<Conditional> = vec![];
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let active = conditionals.last().is_none_or(|x| x.active);
            if let Some(directive) = trimmed.strip_prefix('#') {
                let directive = directive.trim_start();
                let (name, rest) = split_word(directive);
                if name == "include" && active {
                    self.include(rest, base, index + 1, output);
                } else if let Err(mut errors) =
                    self.directive(directive, index + 1, &mut conditionals)
                {
                    self.diagnostics.append(&mut errors);
                }
            } else if active {
                output.push_str(&self.expand(line, &mut HashSet::new()));
                output.push('\n');
            } else {
                output.push('\n');
            }
        }
        if !conditionals.is_empty() {
            self.error(source.lines().count(), "unterminated conditional directive");
        }
    }

    fn include(&mut self, argument: &str, base: Option<&Path>, line: usize, output: &mut String) {
        let expanded = self.expand(argument.trim(), &mut HashSet::new());
        let argument = expanded.trim();
        let (name, local) = if argument.starts_with('"') && argument.ends_with('"') {
            (&argument[1..argument.len() - 1], base)
        } else if argument.starts_with('<') && argument.ends_with('>') {
            (&argument[1..argument.len() - 1], None)
        } else {
            self.error(line, "invalid #include filename");
            return;
        };
        let Some(path) = find_include(name, local) else {
            self.error(line, format!("include file not found: {name}"));
            return;
        };
        let canonical = match path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                self.error(line, format!("cannot open {}: {error}", path.display()));
                return;
            }
        };
        if self.include_stack.contains(&canonical) {
            self.error(
                line,
                format!("recursive include of {}", canonical.display()),
            );
            return;
        }
        let source = match std::fs::read_to_string(&canonical) {
            Ok(source) => source,
            Err(error) => {
                self.error(
                    line,
                    format!("cannot read {}: {error}", canonical.display()),
                );
                return;
            }
        };
        self.include_stack.push(canonical.clone());
        self.process_source(&source, canonical.parent(), output);
        self.include_stack.pop();
    }

    fn directive(
        &mut self,
        text: &str,
        line: usize,
        stack: &mut Vec<Conditional>,
    ) -> Result<(), Vec<Diagnostic>> {
        let (name, rest) = split_word(text);
        let active = stack.last().is_none_or(|x| x.active);
        match name {
            "define" if active => self.define(rest, line),
            "undef" if active => {
                self.macros.remove(rest.trim());
            }
            "include" if active => self.error(
                line,
                "#include requires the file-aware compilation entry point",
            ),
            "error" if active => self.error(line, format!("#error {}", rest.trim())),
            "pragma" | "line" => {}
            "ifdef" | "ifndef" | "if" => {
                let parent = stack.last().is_none_or(|x| x.active);
                let value = if name == "ifdef" {
                    self.macros.contains_key(rest.trim())
                } else if name == "ifndef" {
                    !self.macros.contains_key(rest.trim())
                } else {
                    self.eval_condition(rest, line)
                };
                stack.push(Conditional {
                    parent_active: parent,
                    active: parent && value,
                    branch_taken: value,
                    saw_else: false,
                });
            }
            "elif" => {
                let Some(top) = stack.last_mut() else {
                    self.error(line, "#elif without #if");
                    return Ok(());
                };
                if top.saw_else {
                    self.error(line, "#elif after #else");
                    return Ok(());
                }
                let parent = top.parent_active;
                let taken = top.branch_taken;
                let value = if parent && !taken {
                    self.eval_condition(rest, line)
                } else {
                    false
                };
                let top = stack.last_mut().unwrap();
                top.active = parent && !taken && value;
                top.branch_taken |= value;
            }
            "else" => {
                let Some(top) = stack.last_mut() else {
                    self.error(line, "#else without #if");
                    return Ok(());
                };
                if top.saw_else {
                    self.error(line, "duplicate #else");
                    return Ok(());
                }
                top.saw_else = true;
                top.active = top.parent_active && !top.branch_taken;
                top.branch_taken = true;
            }
            "endif" => {
                if stack.pop().is_none() {
                    self.error(line, "#endif without #if")
                }
            }
            "" => {}
            _ if !active => {}
            _ => self.error(line, format!("unsupported preprocessing directive #{name}")),
        }
        Ok(())
    }

    fn define(&mut self, rest: &str, line: usize) {
        let rest = rest.trim_start();
        let name_end = rest
            .find(|c: char| !(c == '_' || c.is_ascii_alphanumeric()))
            .unwrap_or(rest.len());
        if name_end == 0 {
            self.error(line, "#define requires a macro name");
            return;
        }
        let name = &rest[..name_end];
        let tail = &rest[name_end..];
        let (params, variadic, replacement) = if tail.starts_with('(') {
            match parse_macro_args(tail) {
                Some((args, end)) => {
                    let mut args: Vec<_> = args.into_iter().map(|x| x.trim().to_string()).collect();
                    let variadic = args.last().is_some_and(|x| x == "...");
                    if variadic {
                        *args.last_mut().unwrap() = "__VA_ARGS__".to_string();
                    }
                    (Some(args), variadic, tail[end..].trim_start().to_string())
                }
                None => {
                    self.error(line, "unterminated macro parameter list");
                    return;
                }
            }
        } else {
            (None, false, tail.trim_start().to_string())
        };
        self.macros.insert(
            name.to_string(),
            Macro {
                params,
                variadic,
                replacement,
            },
        );
    }

    fn expand(&self, text: &str, disabled: &mut HashSet<String>) -> String {
        let mut out = String::new();
        let mut i = 0;
        let bytes = text.as_bytes();
        let mut quote = None;
        while i < bytes.len() {
            let c = text[i..].chars().next().unwrap();
            if let Some(q) = quote {
                out.push(c);
                i += c.len_utf8();
                if c == '\\' && i < bytes.len() {
                    let n = text[i..].chars().next().unwrap();
                    out.push(n);
                    i += n.len_utf8()
                } else if c == q {
                    quote = None
                }
                continue;
            }
            if c == '\'' || c == '"' {
                quote = Some(c);
                out.push(c);
                i += 1;
                continue;
            }
            if c == '_' || c.is_ascii_alphabetic() {
                let start = i;
                i += c.len_utf8();
                while i < bytes.len() {
                    let x = text[i..].chars().next().unwrap();
                    if x == '_' || x.is_ascii_alphanumeric() {
                        i += x.len_utf8()
                    } else {
                        break;
                    }
                }
                let name = &text[start..i];
                let Some(mac) = self.macros.get(name) else {
                    out.push_str(name);
                    continue;
                };
                if disabled.contains(name) {
                    out.push_str(name);
                    continue;
                }
                if let Some(params) = &mac.params {
                    let ws = i + text[i..].len() - text[i..].trim_start().len();
                    if !text[ws..].starts_with('(') {
                        out.push_str(name);
                        continue;
                    }
                    let Some((args, end)) = parse_macro_args(&text[ws..]) else {
                        out.push_str(name);
                        continue;
                    };
                    disabled.insert(name.to_string());
                    if (!mac.variadic && args.len() != params.len())
                        || (mac.variadic && args.len() + 1 < params.len())
                    {
                        out.push_str(name);
                        disabled.remove(name);
                        i = ws;
                        continue;
                    }
                    let mut replacement = mac.replacement.clone();
                    for (index, p) in params.iter().enumerate() {
                        let raw = if mac.variadic && index + 1 == params.len() {
                            args[index..].join(",")
                        } else {
                            args.get(index).cloned().unwrap_or_default()
                        };
                        replacement = stringify_parameter(&replacement, p, &raw);
                        replacement =
                            replace_identifier(&replacement, p, &self.expand(raw.trim(), disabled));
                    }
                    replacement = paste_tokens(&replacement);
                    out.push_str(&self.expand(&replacement, disabled));
                    disabled.remove(name);
                    i = ws + end
                } else {
                    disabled.insert(name.to_string());
                    out.push_str(&self.expand(&mac.replacement, disabled));
                    disabled.remove(name);
                }
                continue;
            }
            out.push(c);
            i += c.len_utf8()
        }
        out
    }

    fn eval_condition(&mut self, text: &str, line: usize) -> bool {
        let mut value = text.to_string();
        while let Some(pos) = value.find("defined") {
            let after = &value[pos + 7..];
            let ws = after.len() - after.trim_start().len();
            let after = &after[ws..];
            let (paren, name_len) = if let Some(x) = after.strip_prefix('(') {
                (true, x.find(')').unwrap_or(x.len()))
            } else {
                (
                    false,
                    after
                        .find(|c: char| !(c == '_' || c.is_ascii_alphanumeric()))
                        .unwrap_or(after.len()),
                )
            };
            let name = if paren {
                &after[1..1 + name_len]
            } else {
                &after[..name_len]
            };
            let consumed = 7 + ws + name_len + if paren { 2 } else { 0 };
            value.replace_range(
                pos..pos + consumed,
                if self.macros.contains_key(name) {
                    "1"
                } else {
                    "0"
                },
            );
        }
        value = self.expand(&value, &mut HashSet::new());
        if let Some(value) = eval_pp_expression(&value) {
            value != 0
        } else {
            self.error(line, "unsupported #if constant expression");
            false
        }
    }
    fn error(&mut self, line: usize, msg: impl Into<String>) {
        let p = Position {
            offset: 0,
            line,
            column: 1,
        };
        self.diagnostics.push(Diagnostic::new(
            ErrorKind::Lexical,
            msg,
            Span { start: p, end: p },
        ))
    }
}

fn eval_pp_expression(source: &str) -> Option<i128> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < source.len() {
        let c = source[i..].chars().next()?;
        if c.is_whitespace() {
            i += c.len_utf8();
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < source.len() && source.as_bytes()[i].is_ascii_alphanumeric() {
                i += 1
            }
            tokens.push(source[start..i].to_string());
            continue;
        }
        if c == '_' || c.is_ascii_alphabetic() {
            i += c.len_utf8();
            while i < source.len() {
                let x = source[i..].chars().next()?;
                if x == '_' || x.is_ascii_alphanumeric() {
                    i += x.len_utf8()
                } else {
                    break;
                }
            }
            tokens.push("0".to_string());
            continue;
        }
        let two = source.get(i..i + 2).unwrap_or("");
        if matches!(two, "||" | "&&" | "==" | "!=" | "<=" | ">=" | "<<" | ">>") {
            tokens.push(two.to_string());
            i += 2
        } else {
            tokens.push(c.to_string());
            i += c.len_utf8()
        }
    }
    let mut parser = PpExpr { tokens, pos: 0 };
    let value = parser.conditional()?;
    (parser.pos == parser.tokens.len()).then_some(value)
}

struct PpExpr {
    tokens: Vec<String>,
    pos: usize,
}
impl PpExpr {
    fn peek(&self) -> &str {
        self.tokens.get(self.pos).map(String::as_str).unwrap_or("")
    }
    fn eat(&mut self, s: &str) -> bool {
        if self.peek() == s {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn conditional(&mut self) -> Option<i128> {
        let c = self.binary(1)?;
        if !self.eat("?") {
            return Some(c);
        }
        let a = self.conditional()?;
        if !self.eat(":") {
            return None;
        }
        let b = self.conditional()?;
        Some(if c != 0 { a } else { b })
    }
    fn info(s: &str) -> Option<(u8, &'static str)> {
        Some(match s {
            "||" => (1, "||"),
            "&&" => (2, "&&"),
            "|" => (3, "|"),
            "^" => (4, "^"),
            "&" => (5, "&"),
            "==" => (6, "=="),
            "!=" => (6, "!="),
            "<" => (7, "<"),
            "<=" => (7, "<="),
            ">" => (7, ">"),
            ">=" => (7, ">="),
            "<<" => (8, "<<"),
            ">>" => (8, ">>"),
            "+" => (9, "+"),
            "-" => (9, "-"),
            "*" => (10, "*"),
            "/" => (10, "/"),
            "%" => (10, "%"),
            _ => return None,
        })
    }
    fn binary(&mut self, min: u8) -> Option<i128> {
        let mut lhs = self.unary()?;
        while let Some((p, op)) = Self::info(self.peek()) {
            if p < min {
                break;
            }
            self.pos += 1;
            let rhs = self.binary(p + 1)?;
            lhs = match op {
                "||" => (lhs != 0 || rhs != 0) as i128,
                "&&" => (lhs != 0 && rhs != 0) as i128,
                "|" => lhs | rhs,
                "^" => lhs ^ rhs,
                "&" => lhs & rhs,
                "==" => (lhs == rhs) as i128,
                "!=" => (lhs != rhs) as i128,
                "<" => (lhs < rhs) as i128,
                "<=" => (lhs <= rhs) as i128,
                ">" => (lhs > rhs) as i128,
                ">=" => (lhs >= rhs) as i128,
                "<<" => lhs.wrapping_shl(rhs as u32),
                ">>" => lhs.wrapping_shr(rhs as u32),
                "+" => lhs.wrapping_add(rhs),
                "-" => lhs.wrapping_sub(rhs),
                "*" => lhs.wrapping_mul(rhs),
                "/" => lhs.checked_div(rhs)?,
                "%" => lhs.checked_rem(rhs)?,
                _ => return None,
            }
        }
        Some(lhs)
    }
    fn unary(&mut self) -> Option<i128> {
        if self.eat("+") {
            self.unary()
        } else if self.eat("-") {
            Some(self.unary()?.wrapping_neg())
        } else if self.eat("!") {
            Some((self.unary()? == 0) as i128)
        } else if self.eat("~") {
            Some(!self.unary()?)
        } else if self.eat("(") {
            let x = self.conditional()?;
            self.eat(")").then_some(x)
        } else {
            let raw = self.tokens.get(self.pos)?;
            self.pos += 1;
            let raw = raw.trim_end_matches(['u', 'U', 'l', 'L']);
            if raw.starts_with("0x") || raw.starts_with("0X") {
                i128::from_str_radix(&raw[2..], 16).ok()
            } else if raw.len() > 1 && raw.starts_with('0') {
                i128::from_str_radix(&raw[1..], 8).ok()
            } else {
                raw.parse().ok()
            }
        }
    }
}

fn split_word(s: &str) -> (&str, &str) {
    let n = s.find(char::is_whitespace).unwrap_or(s.len());
    (&s[..n], s[n..].trim_start())
}
fn parse_macro_args(s: &str) -> Option<(Vec<String>, usize)> {
    if !s.starts_with('(') {
        return None;
    }
    let mut depth = 0;
    let mut start = 1;
    let mut out = vec![];
    let mut quote = None;
    for (i, c) in s.char_indices() {
        if i == 0 {
            depth = 1;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c)
        } else if c == '(' {
            depth += 1
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                if i > start || !out.is_empty() {
                    out.push(s[start..i].to_string())
                }
                return Some((out, i + 1));
            }
        } else if c == ',' && depth == 1 {
            out.push(s[start..i].to_string());
            start = i + 1
        }
    }
    None
}
fn stringify_parameter(text: &str, name: &str, raw: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(hash) = rest.find('#') {
        out.push_str(&rest[..hash]);
        if rest[hash..].starts_with("##") {
            out.push_str("##");
            rest = &rest[hash + 2..];
            continue;
        }
        let after = rest[hash + 1..].trim_start();
        if after.strip_prefix(name).is_some_and(|tail| {
            tail.chars()
                .next()
                .is_none_or(|c| !(c == '_' || c.is_ascii_alphanumeric()))
        }) {
            let escaped = raw
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            out.push('"');
            out.push_str(&escaped);
            out.push('"');
            let whitespace = rest[hash + 1..].len() - after.len();
            rest = &rest[hash + 1 + whitespace + name.len()..];
        } else {
            out.push('#');
            rest = &rest[hash + 1..];
        }
    }
    out.push_str(rest);
    out
}

fn paste_tokens(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(pos) = result.find("##") {
        let left = result[..pos].trim_end().len();
        let right_ws = result[pos + 2..].len() - result[pos + 2..].trim_start().len();
        result.replace_range(left..pos + 2 + right_ws, "");
    }
    result
}

fn replace_identifier(text: &str, name: &str, value: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    let mut quote = None;
    while i < text.len() {
        let c = text[i..].chars().next().unwrap();
        if let Some(q) = quote {
            out.push(c);
            i += c.len_utf8();
            if c == '\\' && i < text.len() {
                let next = text[i..].chars().next().unwrap();
                out.push(next);
                i += next.len_utf8();
            } else if c == q {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            out.push(c);
            i += c.len_utf8();
            continue;
        }
        if c == '_' || c.is_ascii_alphabetic() {
            let s = i;
            i += c.len_utf8();
            while i < text.len() {
                let x = text[i..].chars().next().unwrap();
                if x == '_' || x.is_ascii_alphanumeric() {
                    i += x.len_utf8()
                } else {
                    break;
                }
            }
            if &text[s..i] == name {
                out.push_str(value)
            } else {
                out.push_str(&text[s..i])
            }
        } else {
            out.push(c);
            i += c.len_utf8()
        }
    }
    out
}
fn replace_trigraphs(s: &str) -> String {
    s.replace("??=", "#")
        .replace("??/", "\\")
        .replace("??'", "^")
        .replace("??(", "[")
        .replace("??)", "]")
        .replace("??!", "|")
        .replace("??<", "{")
        .replace("??>", "}")
        .replace("??-", "~")
}
fn replace_ucns(source: &str) -> String {
    let mut output = String::new();
    let mut i = 0;
    while i < source.len() {
        let rest = &source[i..];
        let width = if rest.starts_with("\\u") {
            Some(4)
        } else if rest.starts_with("\\U") {
            Some(8)
        } else {
            None
        };
        if let Some(width) = width {
            let end = i + 2 + width;
            if end <= source.len()
                && let Ok(value) = u32::from_str_radix(&source[i + 2..end], 16)
                && let Some(character) = char::from_u32(value)
            {
                output.push(character);
                i = end;
                continue;
            }
        }
        let character = rest.chars().next().unwrap();
        output.push(character);
        i += character.len_utf8();
    }
    output
}
fn splice_lines(s: &str) -> String {
    s.replace("\\\r\n", "").replace("\\\n", "")
}
fn remove_comments(s: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    let mut quote = None;
    while i < s.len() {
        if quote.is_none() && s[i..].starts_with("//") {
            while i < s.len() && s.as_bytes()[i] != b'\n' {
                i += 1
            }
            continue;
        }
        if quote.is_none() && s[i..].starts_with("/*") {
            out.push(' ');
            i += 2;
            while i < s.len() && !s[i..].starts_with("*/") {
                if s.as_bytes()[i] == b'\n' {
                    out.push('\n')
                }
                i += 1
            }
            i = (i + 2).min(s.len());
            continue;
        }
        let c = s[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
        if let Some(q) = quote {
            if c == q {
                quote = None
            } else if c == '\\' && i < s.len() {
                let n = s[i..].chars().next().unwrap();
                out.push(n);
                i += n.len_utf8()
            }
        } else if c == '\'' || c == '"' {
            quote = Some(c)
        }
    }
    out
}

fn find_include(name: &str, local: Option<&Path>) -> Option<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(local) = local {
        dirs.push(local.to_path_buf());
    }
    if let Some(cpath) = std::env::var_os("CPATH") {
        dirs.extend(std::env::split_paths(&cpath));
    }
    dirs.extend([
        PathBuf::from("/usr/local/include"),
        PathBuf::from("/usr/include"),
        PathBuf::from("/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include"),
    ]);
    dirs.into_iter()
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}

fn file_error(message: impl Into<String>) -> Diagnostic {
    let position = Position {
        offset: 0,
        line: 1,
        column: 1,
    };
    Diagnostic::new(
        ErrorKind::Lexical,
        message,
        Span {
            start: position,
            end: position,
        },
    )
}
