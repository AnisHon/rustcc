use crate::lex::{PPToken, PPTokenKind, Punctuator, RawLexer};
use crate::source::{FileId, SourceError, SourceManager, SourceRange};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Option<Vec<String>>,
    pub variadic: bool,
    pub replacement: Vec<PPToken>,
    pub definition_range: SourceRange,
}

#[derive(Debug, Clone)]
struct QueuedToken {
    token: PPToken,
    disabled_macros: Arc<HashSet<String>>,
}

#[derive(Debug, Clone, Copy)]
struct Conditional {
    parent_active: bool,
    active: bool,
    branch_taken: bool,
    saw_else: bool,
}

#[derive(Debug)]
pub struct PreprocessorError {
    pub message: String,
    pub range: SourceRange,
}

impl From<SourceError> for PreprocessorError {
    fn from(error: SourceError) -> Self {
        Self {
            message: error.to_string(),
            range: SourceRange::default(),
        }
    }
}

/// Token-source preprocessor. It consumes raw preprocessing tokens and emits a
/// single include/macro-expanded token stream without constructing language AST.
pub struct Preprocessor<'a> {
    sources: &'a mut SourceManager,
    queue: VecDeque<QueuedToken>,
    macros: HashMap<String, MacroDefinition>,
    conditionals: Vec<Conditional>,
    include_paths: Vec<PathBuf>,
}

impl<'a> Preprocessor<'a> {
    pub fn new(
        sources: &'a mut SourceManager,
        main_file: FileId,
    ) -> Result<Self, PreprocessorError> {
        let queue = lex_file(sources, main_file)?;
        Ok(Self {
            sources,
            queue,
            macros: HashMap::new(),
            conditionals: vec![],
            include_paths: vec![],
        })
    }

    pub fn add_include_path(&mut self, path: impl Into<PathBuf>) {
        self.include_paths.push(path.into());
    }
    pub fn macro_definition(&self, name: &str) -> Option<&MacroDefinition> {
        self.macros.get(name)
    }

    pub fn next_token(&mut self) -> Result<PPToken, PreprocessorError> {
        loop {
            let queued = self.queue.pop_front().ok_or_else(|| PreprocessorError {
                message: "preprocessor token queue exhausted".into(),
                range: SourceRange::default(),
            })?;
            if queued.token.start_of_line
                && queued.token.kind == PPTokenKind::Punctuator(Punctuator::Hash)
            {
                self.handle_directive(queued.token.range)?;
                continue;
            }
            if !self.active() {
                if queued.token.kind == PPTokenKind::EndOfFile {
                    if !self.conditionals.is_empty() {
                        return self
                            .error("unterminated conditional directive", queued.token.range);
                    }
                    return Ok(queued.token);
                }
                continue;
            }
            if queued.token.kind == PPTokenKind::Identifier
                && !queued.disabled_macros.contains(&queued.token.spelling)
                && self.macros.contains_key(&queued.token.spelling)
                && self.expand_macro(&queued)?
            {
                continue;
            }
            if queued.token.kind == PPTokenKind::NewLine {
                continue;
            }
            if queued.token.kind == PPTokenKind::EndOfFile && !self.conditionals.is_empty() {
                return self.error("unterminated conditional directive", queued.token.range);
            }
            if queued.token.kind == PPTokenKind::Invalid {
                return self.error(
                    format!("invalid preprocessing token '{}'", queued.token.spelling),
                    queued.token.range,
                );
            }
            return Ok(queued.token);
        }
    }

    fn handle_directive(&mut self, hash_range: SourceRange) -> Result<(), PreprocessorError> {
        let line = self.take_directive_line();
        let Some(directive) = line.first() else {
            return Ok(());
        };
        if directive.kind != PPTokenKind::Identifier {
            return self.error("expected preprocessing directive", directive.range);
        }
        let rest = &line[1..];
        match directive.spelling.as_str() {
            "define" if self.active() => self.define_macro(rest),
            "undef" if self.active() => {
                if let Some(name) = rest.first() {
                    self.macros.remove(&name.spelling);
                }
                Ok(())
            }
            "include" if self.active() => self.include(rest, hash_range),
            "ifdef" | "ifndef" | "if" => self.begin_conditional(&directive.spelling, rest),
            "elif" => self.elif(rest),
            "else" => self.else_directive(directive.range),
            "endif" => self.endif(directive.range),
            "error" if self.active() => self.error(
                rest.iter()
                    .map(|token| token.spelling.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                directive.range,
            ),
            "line" if self.active() => self.line_directive(rest),
            _ if !self.active() => Ok(()),
            _ => self.error(
                format!("unsupported directive #{}", directive.spelling),
                directive.range,
            ),
        }
    }

    fn take_directive_line(&mut self) -> Vec<PPToken> {
        let mut line = Vec::new();
        while let Some(token) = self.queue.pop_front() {
            if matches!(
                token.token.kind,
                PPTokenKind::NewLine | PPTokenKind::EndOfFile
            ) {
                if token.token.kind == PPTokenKind::EndOfFile {
                    self.queue.push_front(token);
                }
                break;
            }
            line.push(token.token);
        }
        line
    }

    fn define_macro(&mut self, line: &[PPToken]) -> Result<(), PreprocessorError> {
        let Some(name_token) = line.first() else {
            return self.error("#define requires a name", SourceRange::default());
        };
        if name_token.kind != PPTokenKind::Identifier {
            return self.error("invalid macro name", name_token.range);
        }
        let mut index = 1;
        let mut parameters = None;
        let mut variadic = false;
        if line.get(index).is_some_and(|token| {
            token.kind == PPTokenKind::Punctuator(Punctuator::LParen) && !token.leading_space
        }) {
            index += 1;
            let mut params = Vec::new();
            while !line
                .get(index)
                .is_some_and(|token| token.kind == PPTokenKind::Punctuator(Punctuator::RParen))
            {
                let Some(token) = line.get(index) else {
                    return self.error("unterminated macro parameter list", name_token.range);
                };
                if token.kind == PPTokenKind::Punctuator(Punctuator::Ellipsis) {
                    variadic = true;
                    params.push("__VA_ARGS__".into());
                    index += 1;
                    break;
                }
                if token.kind != PPTokenKind::Identifier {
                    return self.error("expected macro parameter", token.range);
                }
                params.push(token.spelling.clone());
                index += 1;
                if line
                    .get(index)
                    .is_some_and(|token| token.kind == PPTokenKind::Punctuator(Punctuator::Comma))
                {
                    index += 1;
                } else {
                    break;
                }
            }
            if !line
                .get(index)
                .is_some_and(|token| token.kind == PPTokenKind::Punctuator(Punctuator::RParen))
            {
                return self.error("unterminated macro parameter list", name_token.range);
            }
            index += 1;
            parameters = Some(params);
        }
        let replacement = line[index..].to_vec();
        let end = replacement
            .last()
            .map_or(name_token.range.end, |token| token.range.end);
        self.macros.insert(
            name_token.spelling.clone(),
            MacroDefinition {
                name: name_token.spelling.clone(),
                parameters,
                variadic,
                replacement,
                definition_range: SourceRange::new(name_token.range.begin, end),
            },
        );
        Ok(())
    }

    fn expand_macro(&mut self, invocation: &QueuedToken) -> Result<bool, PreprocessorError> {
        let definition = self.macros[&invocation.token.spelling].clone();
        let arguments = if let Some(parameters) = &definition.parameters {
            let Some(next) = self.queue.front() else {
                return Ok(false);
            };
            if next.token.kind != PPTokenKind::Punctuator(Punctuator::LParen) {
                return Ok(false);
            }
            self.queue.pop_front();
            Some(self.take_macro_arguments(
                parameters,
                definition.variadic,
                invocation.token.range,
            )?)
        } else {
            None
        };
        let mut disabled = (*invocation.disabled_macros).clone();
        disabled.insert(definition.name.clone());
        let disabled = Arc::new(disabled);
        let substituted =
            self.substitute_replacement(&definition, arguments.as_ref(), invocation.token.range)?;
        let mut expanded = Vec::with_capacity(substituted.len());
        for token in substituted {
            expanded.push(self.with_expansion(
                token,
                invocation.token.range,
                &definition.name,
                Arc::clone(&disabled),
            )?);
        }
        for token in expanded.into_iter().rev() {
            self.queue.push_front(token);
        }
        Ok(true)
    }

    fn substitute_replacement(
        &mut self,
        definition: &MacroDefinition,
        arguments: Option<&HashMap<String, Vec<PPToken>>>,
        invocation: SourceRange,
    ) -> Result<Vec<PPToken>, PreprocessorError> {
        let Some(arguments) = arguments else {
            return Ok(definition.replacement.clone());
        };
        let replacement = &definition.replacement;
        let mut output: Vec<Option<PPToken>> = Vec::new();
        let mut index = 0;
        while index < replacement.len() {
            let token = &replacement[index];
            if token.kind == PPTokenKind::Punctuator(Punctuator::Hash) {
                let Some(parameter) = replacement.get(index + 1) else {
                    return self.error("# must be followed by a macro parameter", token.range);
                };
                let Some(argument) = arguments.get(&parameter.spelling) else {
                    return self.error("# must be followed by a macro parameter", token.range);
                };
                output.push(Some(self.synthetic_token(stringify(argument), invocation)?));
                index += 2;
                continue;
            }
            if token.kind == PPTokenKind::Punctuator(Punctuator::HashHash) {
                let Some(right_token) = replacement.get(index + 1) else {
                    return self.error(
                        "## cannot appear at the end of a replacement list",
                        token.range,
                    );
                };
                let mut right = arguments
                    .get(&right_token.spelling)
                    .cloned()
                    .unwrap_or_else(|| vec![right_token.clone()]);
                let left = output.pop();
                match (left, right.is_empty()) {
                    (None, _) => {
                        return self.error(
                            "## cannot appear at the beginning of a replacement list",
                            token.range,
                        );
                    }
                    (Some(left), true) => output.push(left),
                    (Some(None), false) => {
                        output.extend(right.into_iter().map(Some));
                    }
                    (Some(Some(left)), false) => {
                        let first = right.remove(0);
                        output.push(Some(self.paste_tokens(
                            &left,
                            &first,
                            invocation,
                            token.range,
                        )?));
                        output.extend(right.into_iter().map(Some));
                    }
                }
                index += 2;
                continue;
            }
            if let Some(argument) = arguments.get(&token.spelling) {
                if argument.is_empty()
                    && replacement.get(index + 1).is_some_and(|next| {
                        next.kind == PPTokenKind::Punctuator(Punctuator::HashHash)
                    })
                {
                    output.push(None);
                } else {
                    output.extend(argument.iter().cloned().map(Some));
                }
            } else {
                output.push(Some(token.clone()));
            }
            index += 1;
        }
        Ok(output.into_iter().flatten().collect())
    }

    fn paste_tokens(
        &mut self,
        left: &PPToken,
        right: &PPToken,
        invocation: SourceRange,
        operator_range: SourceRange,
    ) -> Result<PPToken, PreprocessorError> {
        let spelling = format!("{}{}", left.spelling, right.spelling);
        self.synthetic_token(spelling, invocation)
            .map_err(|_| PreprocessorError {
                message: "token pasting did not produce a valid preprocessing token".into(),
                range: operator_range,
            })
    }

    fn synthetic_token(
        &mut self,
        spelling: String,
        invocation: SourceRange,
    ) -> Result<PPToken, PreprocessorError> {
        let file = self.sources.add_included_buffer(
            "<scratch space>",
            spelling.clone(),
            invocation.begin,
        )?;
        let mut lexer = RawLexer::new(self.sources, file)?;
        let mut token = lexer.next_token()?;
        let end = lexer.next_token()?;
        if token.kind == PPTokenKind::Invalid
            || token.kind == PPTokenKind::EndOfFile
            || token.spelling != spelling
            || end.kind != PPTokenKind::EndOfFile
        {
            return self.error("invalid synthetic preprocessing token", invocation);
        }
        token.leading_space = false;
        token.start_of_line = false;
        Ok(token)
    }

    fn take_macro_arguments(
        &mut self,
        parameters: &[String],
        variadic: bool,
        range: SourceRange,
    ) -> Result<HashMap<String, Vec<PPToken>>, PreprocessorError> {
        let mut args = vec![Vec::new()];
        let mut depth = 0;
        loop {
            let Some(token) = self.queue.pop_front() else {
                return self.error("unterminated macro invocation", range);
            };
            match token.token.kind {
                PPTokenKind::Punctuator(Punctuator::LParen) => {
                    depth += 1;
                    args.last_mut().unwrap().push(token.token);
                }
                PPTokenKind::Punctuator(Punctuator::RParen) if depth == 0 => break,
                PPTokenKind::Punctuator(Punctuator::RParen) => {
                    depth -= 1;
                    args.last_mut().unwrap().push(token.token);
                }
                PPTokenKind::Punctuator(Punctuator::Comma) if depth == 0 => args.push(Vec::new()),
                PPTokenKind::EndOfFile => {
                    return self.error("unterminated macro invocation", range);
                }
                _ => args.last_mut().unwrap().push(token.token),
            }
        }
        if args.len() == 1 && args[0].is_empty() && parameters.is_empty() {
            args.clear();
        }
        if (!variadic && args.len() != parameters.len())
            || (variadic && args.len() + 1 < parameters.len())
        {
            return self.error("macro argument count mismatch", range);
        }
        let mut result = HashMap::new();
        for (index, parameter) in parameters.iter().enumerate() {
            if variadic && index + 1 == parameters.len() {
                let mut tokens = Vec::new();
                for (arg_index, argument) in args[index..].iter().enumerate() {
                    if arg_index != 0 {
                        tokens.push(comma_like(
                            argument.first().map_or(range, |token| token.range),
                        ));
                    }
                    tokens.extend(argument.clone());
                }
                result.insert(parameter.clone(), tokens);
            } else {
                result.insert(
                    parameter.clone(),
                    args.get(index).cloned().unwrap_or_default(),
                );
            }
        }
        Ok(result)
    }

    fn with_expansion(
        &mut self,
        mut token: PPToken,
        invocation: SourceRange,
        macro_name: &str,
        disabled_macros: Arc<HashSet<String>>,
    ) -> Result<QueuedToken, PreprocessorError> {
        let begin = self
            .sources
            .expansion_location(token.range.begin, invocation, macro_name)?;
        let end = self
            .sources
            .expansion_location(token.range.end, invocation, macro_name)?;
        token.range = SourceRange::new(begin, end);
        Ok(QueuedToken {
            token,
            disabled_macros,
        })
    }

    fn include(
        &mut self,
        line: &[PPToken],
        include_range: SourceRange,
    ) -> Result<(), PreprocessorError> {
        let Some(name) = line.first() else {
            return self.error("#include requires a filename", include_range);
        };
        let (filename, quoted) = match name.kind {
            PPTokenKind::String => (name.spelling.trim_matches('"').to_string(), true),
            PPTokenKind::Punctuator(Punctuator::Lt) => {
                let mut text = String::new();
                for token in &line[1..] {
                    if token.kind == PPTokenKind::Punctuator(Punctuator::Gt) {
                        break;
                    }
                    text.push_str(&token.spelling);
                }
                (text, false)
            }
            _ => return self.error("invalid #include filename", name.range),
        };
        let including_file = self.sources.file_position(include_range.begin)?.file_id;
        let local = Path::new(self.sources.filename(including_file)?).parent();
        let path = self
            .find_include(&filename, quoted.then_some(local).flatten())
            .ok_or_else(|| PreprocessorError {
                message: format!("include file not found: {filename}"),
                range: name.range,
            })?;
        let file = self.sources.add_file(path, include_range.begin)?;
        let header = lex_file(self.sources, file)?;
        for token in header.into_iter().rev() {
            if token.token.kind != PPTokenKind::EndOfFile {
                self.queue.push_front(token);
            }
        }
        Ok(())
    }

    fn find_include(&self, name: &str, local: Option<&Path>) -> Option<PathBuf> {
        local
            .into_iter()
            .map(|path| path.join(name))
            .chain(self.include_paths.iter().map(|path| path.join(name)))
            .find(|path| path.is_file())
    }

    fn begin_conditional(
        &mut self,
        directive: &str,
        tokens: &[PPToken],
    ) -> Result<(), PreprocessorError> {
        let parent_active = self.active();
        let value = match directive {
            "ifdef" => tokens
                .first()
                .is_some_and(|token| self.macros.contains_key(&token.spelling)),
            "ifndef" => !tokens
                .first()
                .is_some_and(|token| self.macros.contains_key(&token.spelling)),
            _ => eval_pp_condition(tokens, &self.macros),
        };
        self.conditionals.push(Conditional {
            parent_active,
            active: parent_active && value,
            branch_taken: value,
            saw_else: false,
        });
        Ok(())
    }

    fn elif(&mut self, tokens: &[PPToken]) -> Result<(), PreprocessorError> {
        let Some(current) = self.conditionals.last() else {
            return self.error("#elif without #if", SourceRange::default());
        };
        if current.saw_else {
            return self.error("#elif after #else", SourceRange::default());
        }
        let value = current.parent_active
            && !current.branch_taken
            && eval_pp_condition(tokens, &self.macros);
        let current = self.conditionals.last_mut().unwrap();
        current.active = value;
        current.branch_taken |= value;
        Ok(())
    }

    fn else_directive(&mut self, range: SourceRange) -> Result<(), PreprocessorError> {
        let Some(current) = self.conditionals.last_mut() else {
            return self.error("#else without #if", range);
        };
        if current.saw_else {
            return self.error("duplicate #else", range);
        }
        current.saw_else = true;
        current.active = current.parent_active && !current.branch_taken;
        current.branch_taken = true;
        Ok(())
    }

    fn endif(&mut self, range: SourceRange) -> Result<(), PreprocessorError> {
        self.conditionals
            .pop()
            .map(|_| ())
            .ok_or_else(|| PreprocessorError {
                message: "#endif without #if".into(),
                range,
            })
    }

    fn line_directive(&mut self, tokens: &[PPToken]) -> Result<(), PreprocessorError> {
        let Some(number) = tokens.first() else {
            return Ok(());
        };
        let presumed_line = number.spelling.parse().map_err(|_| PreprocessorError {
            message: "invalid #line number".into(),
            range: number.range,
        })?;
        let file = self.sources.file_position(number.range.begin)?.file_id;
        let directive_end = self.sources.file_position(number.range.end)?.byte_offset as usize;
        let buffer = self.sources.buffer(file)?;
        let offset = buffer[directive_end..]
            .find('\n')
            .map_or(buffer.len(), |relative| directive_end + relative + 1)
            as u32;
        let filename = tokens
            .get(1)
            .filter(|token| token.kind == PPTokenKind::String)
            .map(|token| token.spelling.trim_matches('"').to_string());
        self.sources
            .add_line_directive(file, offset, presumed_line, filename)?;
        Ok(())
    }

    fn active(&self) -> bool {
        self.conditionals
            .last()
            .is_none_or(|condition| condition.active)
    }
    fn error<T>(
        &self,
        message: impl Into<String>,
        range: SourceRange,
    ) -> Result<T, PreprocessorError> {
        Err(PreprocessorError {
            message: message.into(),
            range,
        })
    }
}

fn lex_file(
    sources: &mut SourceManager,
    file: FileId,
) -> Result<VecDeque<QueuedToken>, PreprocessorError> {
    let mut lexer = RawLexer::new(sources, file)?;
    let mut tokens = VecDeque::new();
    loop {
        let token = lexer.next_token()?;
        let eof = token.kind == PPTokenKind::EndOfFile;
        tokens.push_back(QueuedToken {
            token,
            disabled_macros: Arc::new(HashSet::new()),
        });
        if eof {
            break;
        }
    }
    Ok(tokens)
}

fn comma_like(range: SourceRange) -> PPToken {
    PPToken {
        kind: PPTokenKind::Punctuator(Punctuator::Comma),
        spelling: ",".into(),
        range,
        leading_space: false,
        start_of_line: false,
    }
}

fn stringify(tokens: &[PPToken]) -> String {
    let mut contents = String::new();
    for (index, token) in tokens.iter().enumerate() {
        if index != 0 && token.leading_space {
            contents.push(' ');
        }
        for character in token.spelling.chars() {
            if matches!(character, '\\' | '"') {
                contents.push('\\');
            }
            contents.push(character);
        }
    }
    format!("\"{contents}\"")
}

fn eval_pp_condition(tokens: &[PPToken], macros: &HashMap<String, MacroDefinition>) -> bool {
    let mut parser = PpExpression::new(tokens, macros);
    parser
        .conditional()
        .filter(|_| parser.position == tokens.len())
        .is_some_and(|value| value != 0)
}

fn parse_pp_integer(text: &str) -> Option<i128> {
    let text = text.trim_end_matches(['u', 'U', 'l', 'L']);
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        i128::from_str_radix(hex, 16).ok()
    } else if text.len() > 1 && text.starts_with('0') {
        i128::from_str_radix(&text[1..], 8).ok()
    } else {
        text.parse().ok()
    }
}

struct PpExpression<'a> {
    tokens: &'a [PPToken],
    macros: &'a HashMap<String, MacroDefinition>,
    position: usize,
    expanding: HashSet<String>,
}

impl<'a> PpExpression<'a> {
    fn new(tokens: &'a [PPToken], macros: &'a HashMap<String, MacroDefinition>) -> Self {
        Self {
            tokens,
            macros,
            position: 0,
            expanding: HashSet::new(),
        }
    }

    fn conditional(&mut self) -> Option<i128> {
        let condition = self.binary(1)?;
        if self.eat(Punctuator::Question) {
            let yes = self.conditional()?;
            self.expect(Punctuator::Colon)?;
            let no = self.conditional()?;
            Some(if condition != 0 { yes } else { no })
        } else {
            Some(condition)
        }
    }

    fn binary(&mut self, minimum: u8) -> Option<i128> {
        let mut left = self.unary()?;
        while let Some((precedence, operator)) = self.binary_operator() {
            if precedence < minimum {
                break;
            }
            self.position += 1;
            let right = self.binary(precedence + 1)?;
            left = apply_pp_binary(operator, left, right)?;
        }
        Some(left)
    }

    fn unary(&mut self) -> Option<i128> {
        if self.identifier("defined") {
            self.position += 1;
            let parenthesized = self.eat(Punctuator::LParen);
            let name = self.tokens.get(self.position)?;
            if name.kind != PPTokenKind::Identifier {
                return None;
            }
            self.position += 1;
            if parenthesized {
                self.expect(Punctuator::RParen)?;
            }
            return Some(i128::from(self.macros.contains_key(&name.spelling)));
        }
        for (punctuator, operation) in [
            (Punctuator::Plus, 0_u8),
            (Punctuator::Minus, 1),
            (Punctuator::Bang, 2),
            (Punctuator::Tilde, 3),
        ] {
            if self.eat(punctuator) {
                let value = self.unary()?;
                return Some(match operation {
                    0 => value,
                    1 => value.wrapping_neg(),
                    2 => i128::from(value == 0),
                    _ => !value,
                });
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> Option<i128> {
        if self.eat(Punctuator::LParen) {
            let value = self.conditional()?;
            self.expect(Punctuator::RParen)?;
            return Some(value);
        }
        let token = self.tokens.get(self.position)?;
        self.position += 1;
        match token.kind {
            PPTokenKind::Number => parse_pp_integer(&token.spelling),
            PPTokenKind::Character => parse_pp_character(&token.spelling),
            PPTokenKind::Identifier => self.object_macro_value(&token.spelling).or(Some(0)),
            _ => None,
        }
    }

    fn object_macro_value(&mut self, name: &str) -> Option<i128> {
        if !self.expanding.insert(name.to_string()) {
            return Some(0);
        }
        let definition = self.macros.get(name)?;
        if definition.parameters.is_some() {
            self.expanding.remove(name);
            return Some(0);
        }
        let mut nested = Self {
            tokens: &definition.replacement,
            macros: self.macros,
            position: 0,
            expanding: self.expanding.clone(),
        };
        let value = nested.conditional();
        self.expanding.remove(name);
        value.filter(|_| nested.position == definition.replacement.len())
    }

    fn binary_operator(&self) -> Option<(u8, Punctuator)> {
        let PPTokenKind::Punctuator(operator) = self.tokens.get(self.position)?.kind else {
            return None;
        };
        let precedence = match operator {
            Punctuator::Or => 1,
            Punctuator::And => 2,
            Punctuator::Pipe => 3,
            Punctuator::Caret => 4,
            Punctuator::Amp => 5,
            Punctuator::Eq | Punctuator::Ne => 6,
            Punctuator::Lt | Punctuator::Le | Punctuator::Gt | Punctuator::Ge => 7,
            Punctuator::Shl | Punctuator::Shr => 8,
            Punctuator::Plus | Punctuator::Minus => 9,
            Punctuator::Star | Punctuator::Slash | Punctuator::Percent => 10,
            _ => return None,
        };
        Some((precedence, operator))
    }

    fn identifier(&self, spelling: &str) -> bool {
        self.tokens.get(self.position).is_some_and(|token| {
            token.kind == PPTokenKind::Identifier && token.spelling == spelling
        })
    }

    fn eat(&mut self, punctuator: Punctuator) -> bool {
        if self
            .tokens
            .get(self.position)
            .is_some_and(|token| token.kind == PPTokenKind::Punctuator(punctuator))
        {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, punctuator: Punctuator) -> Option<()> {
        self.eat(punctuator).then_some(())
    }
}

fn apply_pp_binary(operator: Punctuator, left: i128, right: i128) -> Option<i128> {
    Some(match operator {
        Punctuator::Or => i128::from(left != 0 || right != 0),
        Punctuator::And => i128::from(left != 0 && right != 0),
        Punctuator::Pipe => left | right,
        Punctuator::Caret => left ^ right,
        Punctuator::Amp => left & right,
        Punctuator::Eq => i128::from(left == right),
        Punctuator::Ne => i128::from(left != right),
        Punctuator::Lt => i128::from(left < right),
        Punctuator::Le => i128::from(left <= right),
        Punctuator::Gt => i128::from(left > right),
        Punctuator::Ge => i128::from(left >= right),
        Punctuator::Shl => left.checked_shl(u32::try_from(right).ok()?)?,
        Punctuator::Shr => left.checked_shr(u32::try_from(right).ok()?)?,
        Punctuator::Plus => left.wrapping_add(right),
        Punctuator::Minus => left.wrapping_sub(right),
        Punctuator::Star => left.wrapping_mul(right),
        Punctuator::Slash => left.checked_div(right)?,
        Punctuator::Percent => left.checked_rem(right)?,
        _ => return None,
    })
}

fn parse_pp_character(text: &str) -> Option<i128> {
    let body = text
        .trim_start_matches(['L', 'u', 'U'])
        .strip_prefix('\'')?
        .strip_suffix('\'')?;
    if let Some(escaped) = body.strip_prefix('\\') {
        Some(match escaped {
            "n" => '\n' as i128,
            "r" => '\r' as i128,
            "t" => '\t' as i128,
            "0" => 0,
            "\\" => '\\' as i128,
            "'" => '\'' as i128,
            _ => return None,
        })
    } else {
        body.chars().next().map(|character| character as i128)
    }
}
