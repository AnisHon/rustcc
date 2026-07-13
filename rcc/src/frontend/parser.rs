use super::*;
use std::collections::HashMap;

type PResult<T> = Result<T, Diagnostic>;

#[derive(Clone)]
struct DeclSpec {
    ty: CType,
    storage: StorageClass,
    function_specifiers: FunctionSpecifiers,
    alignment: Option<usize>,
}

#[derive(Clone)]
enum DNode {
    Name(Option<String>),
    Pointer(Box<DNode>, Qualifiers),
    Array(Box<DNode>, ArraySize),
    Function(Box<DNode>, Vec<Parameter>, bool, bool),
}

pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, CType>>,
    typedefs: Vec<HashMap<String, CType>>,
    constants: Vec<HashMap<String, i128>>,
    tags: HashMap<(String, u8), CType>,
    current_return: Option<CType>,
    loop_depth: usize,
    switch_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: vec![],
            scopes: vec![HashMap::new()],
            typedefs: vec![HashMap::new()],
            constants: vec![HashMap::new()],
            tags: HashMap::new(),
            current_return: None,
            loop_depth: 0,
            switch_depth: 0,
        }
    }
    pub fn parse(mut self) -> Result<TranslationUnit, Vec<Diagnostic>> {
        let mut declarations = vec![];
        while !self.at(&TokenKind::Eof) {
            match self.external_declaration() {
                Ok(mut x) => declarations.append(&mut x),
                Err(e) => {
                    self.diagnostics.push(e);
                    self.synchronize();
                }
            }
        }
        if self.diagnostics.is_empty() {
            Ok(TranslationUnit { declarations })
        } else {
            Err(self.diagnostics)
        }
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }
    fn at(&self, k: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(k)
    }
    fn kw(&self, k: Keyword) -> bool {
        self.peek().kind == TokenKind::Keyword(k)
    }
    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if !matches!(t.kind, TokenKind::Eof) {
            self.pos += 1
        }
        t
    }
    fn eat(&mut self, k: &TokenKind) -> Option<Token> {
        if self.at(k) { Some(self.bump()) } else { None }
    }
    fn eat_kw(&mut self, k: Keyword) -> Option<Token> {
        if self.kw(k) { Some(self.bump()) } else { None }
    }
    fn expect(&mut self, k: &TokenKind) -> PResult<Token> {
        if self.at(k) {
            Ok(self.bump())
        } else {
            Err(self.err(format!("expected {:?}, found {:?}", k, self.peek().kind)))
        }
    }
    fn err(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(ErrorKind::Syntax, msg, self.peek().span)
    }
    fn sema_err(&self, msg: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic::new(ErrorKind::Semantic, msg, span)
    }
    fn synchronize(&mut self) {
        while !self.at(&TokenKind::Eof) {
            if self.eat(&TokenKind::Semi).is_some() {
                break;
            }
            if self.at(&TokenKind::RBrace) {
                self.bump();
                break;
            }
            self.bump();
        }
    }
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.typedefs.push(HashMap::new());
        self.constants.push(HashMap::new())
    }
    fn leave_scope(&mut self) {
        self.scopes.pop();
        self.typedefs.pop();
        self.constants.pop();
    }
    fn lookup(&self, n: &str) -> Option<CType> {
        self.scopes.iter().rev().find_map(|s| s.get(n).cloned())
    }
    fn lookup_typedef(&self, n: &str) -> Option<CType> {
        for index in (0..self.typedefs.len()).rev() {
            if self.scopes[index].contains_key(n) {
                return None;
            }
            if let Some(ty) = self.typedefs[index].get(n) {
                return Some(ty.clone());
            }
        }
        None
    }
    fn declare(&mut self, d: &Declaration) -> PResult<()> {
        if let Some(n) = &d.name {
            let table = if d.storage == StorageClass::Typedef {
                if self.scopes.last().unwrap().contains_key(n) {
                    return Err(self.sema_err(
                        format!("redefinition of '{n}' in the ordinary identifier namespace"),
                        d.span,
                    ));
                }
                self.typedefs.last_mut().unwrap()
            } else {
                if self.typedefs.last().unwrap().contains_key(n) {
                    return Err(
                        self.sema_err(format!("redefinition of typedef name '{n}'"), d.span)
                    );
                }
                self.scopes.last_mut().unwrap()
            };
            if let Some(old) = table.get(n) {
                if old != &d.ty {
                    return Err(
                        self.sema_err(format!("incompatible redeclaration of '{n}'"), d.span)
                    );
                }
            } else {
                table.insert(n.clone(), d.ty.clone());
            }
        }
        Ok(())
    }

    fn external_declaration(&mut self) -> PResult<Vec<ExternalDeclaration>> {
        if self.kw(Keyword::StaticAssert) {
            return Ok(vec![ExternalDeclaration::StaticAssert(
                self.static_assert()?,
            )]);
        }
        let start = self.peek().span;
        let spec = self.declaration_specifiers()?;
        if self.eat(&TokenKind::Semi).is_some() {
            return Ok(vec![ExternalDeclaration::Declaration(Declaration {
                name: None,
                ty: spec.ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                span: start.join(self.previous().span),
            })]);
        }
        let node = self.declarator(false)?;
        let (name, mut ty, mut params) = self.apply_declarator(node, spec.ty.clone())?;
        let old_style = matches!(
            &ty.kind,
            TypeKind::Function {
                has_prototype: false,
                ..
            }
        ) && !self.at(&TokenKind::LBrace)
            && self.is_declaration_start();
        if old_style {
            self.old_parameter_declarations(&mut params)?;
            if let TypeKind::Function {
                params: function_params,
                ..
            } = &mut ty.kind
            {
                *function_params = params.clone();
            }
        }
        if self.at(&TokenKind::LBrace) {
            let name = name.ok_or_else(|| self.err("function definition requires a name"))?;
            let TypeKind::Function { .. } = ty.kind else {
                return Err(self.err("only a function declarator may have a body"));
            };
            let d = Declaration {
                name: Some(name.clone()),
                ty: ty.clone(),
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                span: start,
            };
            self.declare(&d)?;
            self.enter_scope();
            for p in &params {
                if let Some(n) = &p.name {
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(n.clone(), p.ty.clone());
                }
            }
            let saved = self.current_return.replace(match &ty.kind {
                TypeKind::Function { return_type, .. } => (**return_type).clone(),
                _ => unreachable!(),
            });
            let body = self.compound_statement(false)?;
            self.current_return = saved;
            self.leave_scope();
            let span = start.join(body.span);
            return Ok(vec![ExternalDeclaration::Function(FunctionDefinition {
                name,
                ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                parameters: params,
                body,
                span,
            })]);
        }
        let mut decls = vec![];
        let first = self.finish_declaration(start, spec.clone(), name, ty)?;
        self.declare(&first)?;
        decls.push(ExternalDeclaration::Declaration(first));
        while self.eat(&TokenKind::Comma).is_some() {
            let node = self.declarator(false)?;
            let (n, t, _) = self.apply_declarator(node, spec.ty.clone())?;
            let d = self.finish_declaration(start, spec.clone(), n, t)?;
            self.declare(&d)?;
            decls.push(ExternalDeclaration::Declaration(d));
        }
        self.expect(&TokenKind::Semi)?;
        Ok(decls)
    }
    fn finish_declaration(
        &mut self,
        start: Span,
        spec: DeclSpec,
        name: Option<String>,
        mut ty: CType,
    ) -> PResult<Declaration> {
        let initializer = if self.eat(&TokenKind::Assign).is_some() {
            Some(self.initializer()?)
        } else {
            None
        };
        if let Some(initializer) = &initializer {
            if let TypeKind::Array { size, .. } = &mut ty.kind
                && matches!(size, ArraySize::Unspecified)
            {
                *size = match initializer {
                    Initializer::List(items) => ArraySize::Constant(items.len()),
                    Initializer::Expression(Expression {
                        kind: ExpressionKind::String { value, .. },
                        ..
                    }) => ArraySize::Constant(value.chars().count() + 1),
                    _ => ArraySize::Unspecified,
                };
            }
            self.check_initializer(&ty, initializer)?;
        }
        Ok(Declaration {
            name,
            ty,
            storage: spec.storage,
            function_specifiers: spec.function_specifiers,
            initializer,
            alignment: spec.alignment,
            span: start.join(self.previous().span),
        })
    }

    fn check_initializer(&self, target: &CType, initializer: &Initializer) -> PResult<()> {
        match (initializer, &target.kind) {
            (Initializer::Expression(expression), TypeKind::Array { element, .. })
                if matches!(expression.kind, ExpressionKind::String { .. }) =>
            {
                let TypeKind::Array {
                    element: source, ..
                } = &expression.ty.kind
                else {
                    unreachable!()
                };
                if self.compatible(element, source) {
                    Ok(())
                } else {
                    Err(self.sema_err(
                        "string literal encoding does not match array element type",
                        expression.span,
                    ))
                }
            }
            (Initializer::Expression(expression), _) => self.require_assignable(target, expression),
            (Initializer::List(items), TypeKind::Array { element, size }) => {
                if matches!(size, ArraySize::Constant(size) if items.len() > *size) {
                    return Err(self.sema_err("too many array initializers", self.peek().span));
                }
                for item in items {
                    self.check_initializer(element, &item.value)?;
                }
                Ok(())
            }
            (
                Initializer::List(items),
                TypeKind::Struct {
                    fields: Some(fields),
                    ..
                },
            ) => {
                let mut next = 0;
                for item in items {
                    if let Some(Designator::Field(name)) = item.designators.first() {
                        next = fields
                            .iter()
                            .position(|field| field.name.as_deref() == Some(name))
                            .ok_or_else(|| {
                                self.sema_err(
                                    format!("no field named '{name}' in initializer"),
                                    self.peek().span,
                                )
                            })?;
                    }
                    let field = fields.get(next).ok_or_else(|| {
                        self.sema_err("too many struct initializers", self.peek().span)
                    })?;
                    self.check_initializer(&field.ty, &item.value)?;
                    next += 1;
                }
                Ok(())
            }
            (
                Initializer::List(items),
                TypeKind::Union {
                    fields: Some(fields),
                    ..
                },
            ) => {
                if items.len() > 1 {
                    return Err(self.sema_err("too many union initializers", self.peek().span));
                }
                if let Some(item) = items.first() {
                    let field = if let Some(Designator::Field(name)) = item.designators.first() {
                        fields
                            .iter()
                            .find(|field| field.name.as_deref() == Some(name))
                    } else {
                        fields.first()
                    }
                    .ok_or_else(|| {
                        self.sema_err("union has no matching field", self.peek().span)
                    })?;
                    self.check_initializer(&field.ty, &item.value)?;
                }
                Ok(())
            }
            (Initializer::List(items), _) if items.len() == 1 => {
                self.check_initializer(target, &items[0].value)
            }
            (Initializer::List(_), _) => {
                Err(self.sema_err("excess elements in scalar initializer", self.peek().span))
            }
        }
    }

    fn declaration_specifiers(&mut self) -> PResult<DeclSpec> {
        use Keyword::*;
        let mut storage = StorageClass::None;
        let mut q = Qualifiers::default();
        let mut signed = None;
        let mut long_count = 0;
        let mut short = false;
        let mut base: Option<CType> = None;
        let mut complex = None;
        let mut consumed = false;
        let mut alignment = None;
        let mut function_specifiers = FunctionSpecifiers::default();
        loop {
            match self.peek().kind.clone() {
                TokenKind::Keyword(Typedef | Extern | Static | Auto | Register | ThreadLocal) => {
                    consumed = true;
                    let k = match self.bump().kind {
                        TokenKind::Keyword(k) => k,
                        _ => unreachable!(),
                    };
                    let s = match k {
                        Typedef => StorageClass::Typedef,
                        Extern => StorageClass::Extern,
                        Static => StorageClass::Static,
                        ThreadLocal => StorageClass::ThreadLocal,
                        Auto => StorageClass::Auto,
                        Register => StorageClass::Register,
                        _ => unreachable!(),
                    };
                    storage = match (storage, s) {
                        (StorageClass::None, x) | (x, StorageClass::None) => x,
                        (StorageClass::Static, StorageClass::ThreadLocal)
                        | (StorageClass::ThreadLocal, StorageClass::Static) => {
                            StorageClass::StaticThreadLocal
                        }
                        (StorageClass::Extern, StorageClass::ThreadLocal)
                        | (StorageClass::ThreadLocal, StorageClass::Extern) => {
                            StorageClass::ExternThreadLocal
                        }
                        _ => return Err(self.err("invalid storage-class combination")),
                    }
                }
                TokenKind::Keyword(Const) => {
                    consumed = true;
                    q.is_const = true;
                    self.bump();
                }
                TokenKind::Keyword(Volatile) => {
                    consumed = true;
                    q.is_volatile = true;
                    self.bump();
                }
                TokenKind::Keyword(Restrict) => {
                    consumed = true;
                    q.is_restrict = true;
                    self.bump();
                }
                TokenKind::Keyword(Signed) => {
                    consumed = true;
                    signed = Some(true);
                    self.bump();
                }
                TokenKind::Keyword(Unsigned) => {
                    consumed = true;
                    signed = Some(false);
                    self.bump();
                }
                TokenKind::Keyword(Short) => {
                    consumed = true;
                    short = true;
                    self.bump();
                }
                TokenKind::Keyword(Long) => {
                    consumed = true;
                    long_count += 1;
                    self.bump();
                }
                TokenKind::Keyword(Void) => {
                    consumed = true;
                    base = Some(CType::void());
                    self.bump();
                }
                TokenKind::Keyword(Bool) => {
                    consumed = true;
                    base = Some(CType::new(TypeKind::Bool));
                    self.bump();
                }
                TokenKind::Keyword(Char) => {
                    consumed = true;
                    base = Some(CType::new(TypeKind::Char { signed }));
                    self.bump();
                }
                TokenKind::Keyword(Int) => {
                    consumed = true;
                    base = Some(CType::int());
                    self.bump();
                }
                TokenKind::Keyword(Float) => {
                    consumed = true;
                    base = Some(CType::new(TypeKind::Float));
                    self.bump();
                }
                TokenKind::Keyword(Double) => {
                    consumed = true;
                    base = Some(CType::new(TypeKind::Double));
                    self.bump();
                }
                TokenKind::Keyword(Struct) => {
                    consumed = true;
                    base = Some(self.record_spec(false)?);
                }
                TokenKind::Keyword(Union) => {
                    consumed = true;
                    base = Some(self.record_spec(true)?);
                }
                TokenKind::Keyword(Enum) => {
                    consumed = true;
                    base = Some(self.enum_spec()?);
                }
                TokenKind::Keyword(Atomic) => {
                    consumed = true;
                    self.bump();
                    if self.eat(&TokenKind::LParen).is_some() {
                        let mut t = self.type_name()?;
                        self.expect(&TokenKind::RParen)?;
                        t.qualifiers.is_atomic = true;
                        base = Some(t)
                    } else {
                        q.is_atomic = true
                    }
                }
                TokenKind::Keyword(Alignas) => {
                    consumed = true;
                    self.bump();
                    self.expect(&TokenKind::LParen)?;
                    if self.is_type_start() {
                        let t = self.type_name()?;
                        alignment = Some(self.alignof(&t))
                    } else {
                        let e = self.expression()?;
                        alignment = Some(self.const_int(&e).ok_or_else(|| {
                            self.sema_err("alignment must be an integer constant", e.span)
                        })? as usize)
                    }
                    self.expect(&TokenKind::RParen)?;
                }
                TokenKind::Keyword(Complex) => {
                    consumed = true;
                    complex = Some(false);
                    self.bump();
                }
                TokenKind::Keyword(Imaginary) => {
                    consumed = true;
                    complex = Some(true);
                    self.bump();
                }
                TokenKind::Keyword(Inline | Noreturn) => {
                    consumed = true;
                    let keyword = self.bump().kind;
                    function_specifiers.is_inline |= keyword == TokenKind::Keyword(Inline);
                    function_specifiers.is_noreturn |= keyword == TokenKind::Keyword(Noreturn);
                }
                TokenKind::Identifier(ref n) if self.lookup_typedef(n).is_some() => {
                    consumed = true;
                    let n = n.clone();
                    self.bump();
                    base = self.lookup_typedef(&n);
                }
                _ => break,
            }
        }
        if !consumed {
            return Err(self.err("expected declaration specifier"));
        }
        if complex.is_some() && base.is_none() && !short && long_count == 0 && signed.is_none() {
            base = Some(CType::new(TypeKind::Double));
        }
        let mut ty = if let Some(mut b) = base.take() {
            match b.kind {
                TypeKind::Int { .. } => {
                    b.kind = if short {
                        TypeKind::Short {
                            signed: signed.unwrap_or(true),
                        }
                    } else if long_count == 1 {
                        TypeKind::Long {
                            signed: signed.unwrap_or(true),
                        }
                    } else if long_count >= 2 {
                        TypeKind::LongLong {
                            signed: signed.unwrap_or(true),
                        }
                    } else {
                        TypeKind::Int {
                            signed: signed.unwrap_or(true),
                        }
                    }
                }
                TypeKind::Double if long_count > 0 => b.kind = TypeKind::LongDouble,
                _ if short || long_count > 0 || signed.is_some() => {
                    return Err(self.err("invalid type specifier combination"));
                }
                _ => {}
            }
            b
        } else if short {
            CType::new(TypeKind::Short {
                signed: signed.unwrap_or(true),
            })
        } else if long_count == 1 {
            CType::new(TypeKind::Long {
                signed: signed.unwrap_or(true),
            })
        } else if long_count >= 2 {
            CType::new(TypeKind::LongLong {
                signed: signed.unwrap_or(true),
            })
        } else {
            CType::new(TypeKind::Int {
                signed: signed.unwrap_or(true),
            })
        };
        if let Some(imaginary) = complex {
            if !matches!(
                ty.kind,
                TypeKind::Float | TypeKind::Double | TypeKind::LongDouble
            ) {
                return Err(self.err("_Complex and _Imaginary require a floating type"));
            }
            ty = CType::new(if imaginary {
                TypeKind::Imaginary(Box::new(ty))
            } else {
                TypeKind::Complex(Box::new(ty))
            });
        }
        ty.qualifiers = q;
        Ok(DeclSpec {
            ty,
            storage,
            function_specifiers,
            alignment,
        })
    }
    fn is_type_start(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Keyword(k) => matches!(
                k,
                Keyword::Void
                    | Keyword::Bool
                    | Keyword::Char
                    | Keyword::Short
                    | Keyword::Int
                    | Keyword::Long
                    | Keyword::Float
                    | Keyword::Double
                    | Keyword::Signed
                    | Keyword::Unsigned
                    | Keyword::Struct
                    | Keyword::Union
                    | Keyword::Enum
                    | Keyword::Const
                    | Keyword::Volatile
                    | Keyword::Restrict
                    | Keyword::Atomic
                    | Keyword::Complex
                    | Keyword::Imaginary
            ),
            TokenKind::Identifier(n) => self.lookup_typedef(n).is_some(),
            _ => false,
        }
    }

    fn record_spec(&mut self, union: bool) -> PResult<CType> {
        self.bump();
        let name = if let TokenKind::Identifier(n) = self.peek().kind.clone() {
            self.bump();
            Some(n)
        } else {
            None
        };
        let key = name.clone().map(|n| (n, if union { 1 } else { 0 }));
        if self.eat(&TokenKind::LBrace).is_none() {
            if let Some(k) = key.as_ref()
                && let Some(t) = self.tags.get(k)
            {
                return Ok(t.clone());
            }
            return Ok(CType::new(if union {
                TypeKind::Union { name, fields: None }
            } else {
                TypeKind::Struct { name, fields: None }
            }));
        }
        let mut fields = vec![];
        while !self.at(&TokenKind::RBrace) {
            let s = self.peek().span;
            let spec = self.declaration_specifiers()?;
            if self.eat(&TokenKind::Semi).is_some() {
                continue;
            }
            loop {
                let node = self.declarator(true)?;
                let (n, ty, _) = self.apply_declarator(node, spec.ty.clone())?;
                let bit_width =
                    if self.eat(&TokenKind::Colon).is_some() {
                        let e = self.assignment_expression()?;
                        Some(self.const_int(&e).ok_or_else(|| {
                            self.sema_err("bit-field width must be constant", e.span)
                        })? as u32)
                    } else {
                        None
                    };
                fields.push(Field {
                    name: n,
                    ty,
                    bit_width,
                    span: s.join(self.previous().span),
                });
                if self.eat(&TokenKind::Comma).is_none() {
                    break;
                }
            }
            self.expect(&TokenKind::Semi)?;
        }
        self.expect(&TokenKind::RBrace)?;
        let ty = CType::new(if union {
            TypeKind::Union {
                name,
                fields: Some(fields),
            }
        } else {
            TypeKind::Struct {
                name,
                fields: Some(fields),
            }
        });
        if let Some(k) = key {
            self.tags.insert(k, ty.clone());
        }
        Ok(ty)
    }
    fn enum_spec(&mut self) -> PResult<CType> {
        self.bump();
        let name = if let TokenKind::Identifier(n) = self.peek().kind.clone() {
            self.bump();
            Some(n)
        } else {
            None
        };
        let key = name.clone().map(|n| (n, 2));
        if self.eat(&TokenKind::LBrace).is_none() {
            if let Some(k) = key.as_ref()
                && let Some(t) = self.tags.get(k)
            {
                return Ok(t.clone());
            }
            return Ok(CType::new(TypeKind::Enum {
                name,
                variants: None,
            }));
        }
        let mut variants = vec![];
        let mut next = 0i64;
        while !self.at(&TokenKind::RBrace) {
            let t = self.bump();
            let TokenKind::Identifier(n) = t.kind else {
                return Err(self.err("expected enumerator name"));
            };
            if self.eat(&TokenKind::Assign).is_some() {
                let e = self.assignment_expression()?;
                next = self.const_int(&e).ok_or_else(|| {
                    self.sema_err("enumerator must be an integer constant", e.span)
                })? as i64
            }
            variants.push(EnumVariant {
                name: n.clone(),
                value: next,
                span: t.span,
            });
            self.scopes
                .last_mut()
                .unwrap()
                .insert(n.clone(), CType::int());
            self.constants.last_mut().unwrap().insert(n, next as i128);
            next += 1;
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;
        let ty = CType::new(TypeKind::Enum {
            name,
            variants: Some(variants),
        });
        if let Some(k) = key {
            self.tags.insert(k, ty.clone());
        }
        Ok(ty)
    }

    fn declarator(&mut self, abstract_ok: bool) -> PResult<DNode> {
        let mut ptrs = vec![];
        while self.eat(&TokenKind::Star).is_some() {
            let mut q = Qualifiers::default();
            loop {
                if self.eat_kw(Keyword::Const).is_some() {
                    q.is_const = true
                } else if self.eat_kw(Keyword::Volatile).is_some() {
                    q.is_volatile = true
                } else if self.eat_kw(Keyword::Restrict).is_some() {
                    q.is_restrict = true
                } else {
                    break;
                }
            }
            ptrs.push(q)
        }
        let mut node = if let TokenKind::Identifier(n) = self.peek().kind.clone() {
            self.bump();
            DNode::Name(Some(n))
        } else if self.eat(&TokenKind::LParen).is_some() {
            let n = self.declarator(abstract_ok)?;
            self.expect(&TokenKind::RParen)?;
            n
        } else if abstract_ok {
            DNode::Name(None)
        } else {
            return Err(self.err("expected declarator"));
        };
        loop {
            if self.eat(&TokenKind::LBracket).is_some() {
                while self.eat_kw(Keyword::Static).is_some()
                    || self.eat_kw(Keyword::Const).is_some()
                    || self.eat_kw(Keyword::Restrict).is_some()
                {}
                let size = if self.eat(&TokenKind::Star).is_some() {
                    ArraySize::Star
                } else if self.at(&TokenKind::RBracket) {
                    ArraySize::Unspecified
                } else {
                    let e = self.assignment_expression()?;
                    if !e.ty.is_integer() {
                        return Err(self.sema_err("array bound must have integer type", e.span));
                    }
                    match self.const_int(&e) {
                        Some(value) if value > 0 => ArraySize::Constant(value as usize),
                        Some(_) => {
                            return Err(
                                self.sema_err("array bound must be greater than zero", e.span)
                            );
                        }
                        None if self.scopes.len() > 1 => ArraySize::Variable(Box::new(e)),
                        None => {
                            return Err(self.sema_err(
                                "variably modified type is not allowed at file scope",
                                e.span,
                            ));
                        }
                    }
                };
                self.expect(&TokenKind::RBracket)?;
                node = DNode::Array(Box::new(node), size)
            } else if self.eat(&TokenKind::LParen).is_some() {
                let (mut ps, var, mut has_prototype) = self.parameter_list()?;
                self.expect(&TokenKind::RParen)?;
                if ps.len() == 1 && ps[0].name.is_none() && matches!(ps[0].ty.kind, TypeKind::Void)
                {
                    ps.clear()
                } else if ps.is_empty() {
                    has_prototype = false;
                }
                node = DNode::Function(Box::new(node), ps, var, has_prototype)
            } else {
                break;
            }
        }
        for q in ptrs.into_iter().rev() {
            node = DNode::Pointer(Box::new(node), q)
        }
        Ok(node)
    }
    fn parameter_list(&mut self) -> PResult<(Vec<Parameter>, bool, bool)> {
        let mut p = vec![];
        let mut var = false;
        if self.at(&TokenKind::RParen) {
            return Ok((p, var, false));
        }
        if matches!(self.peek().kind, TokenKind::Identifier(ref name) if self.lookup_typedef(name).is_none())
        {
            loop {
                let token = self.bump();
                let TokenKind::Identifier(name) = token.kind else {
                    unreachable!()
                };
                p.push(Parameter {
                    name: Some(name),
                    ty: CType::int(),
                    span: token.span,
                });
                if self.eat(&TokenKind::Comma).is_none() {
                    break;
                }
            }
            return Ok((p, false, false));
        }
        loop {
            if self.eat(&TokenKind::Ellipsis).is_some() {
                if p.is_empty() {
                    return Err(self.err("ellipsis requires at least one named parameter"));
                }
                var = true;
                break;
            }
            let s = self.peek().span;
            let spec = self.declaration_specifiers()?;
            let node = self.declarator(true)?;
            let (n, mut ty, _) = self.apply_declarator(node, spec.ty)?;
            if let TypeKind::Array { element, .. } = ty.kind {
                ty = CType::pointer(*element)
            } else if matches!(ty.kind, TypeKind::Function { .. }) {
                ty = CType::pointer(ty)
            }
            p.push(Parameter {
                name: n,
                ty,
                span: s.join(self.previous().span),
            });
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        if p.iter()
            .any(|parameter| matches!(parameter.ty.kind, TypeKind::Void))
            && !(p.len() == 1 && p[0].name.is_none())
        {
            return Err(self.err("void must be the only parameter and have no name"));
        }
        Ok((p, var, true))
    }

    fn old_parameter_declarations(&mut self, params: &mut [Parameter]) -> PResult<()> {
        while !self.at(&TokenKind::LBrace) {
            let spec = self.declaration_specifiers()?;
            if !matches!(spec.storage, StorageClass::None | StorageClass::Register) {
                return Err(self.err("invalid storage class in old-style parameter declaration"));
            }
            loop {
                let node = self.declarator(false)?;
                let (name, mut ty, _) = self.apply_declarator(node, spec.ty.clone())?;
                if let TypeKind::Array { element, .. } = ty.kind {
                    ty = CType::pointer(*element);
                } else if matches!(ty.kind, TypeKind::Function { .. }) {
                    ty = CType::pointer(ty);
                }
                let name = name.ok_or_else(|| self.err("old-style parameter requires a name"))?;
                let parameter = params
                    .iter_mut()
                    .find(|parameter| parameter.name.as_deref() == Some(&name))
                    .ok_or_else(|| self.err(format!("declaration for non-parameter '{name}'")))?;
                parameter.ty = ty;
                if self.eat(&TokenKind::Comma).is_none() {
                    break;
                }
            }
            self.expect(&TokenKind::Semi)?;
        }
        Ok(())
    }
    fn apply_declarator(
        &self,
        node: DNode,
        base: CType,
    ) -> PResult<(Option<String>, CType, Vec<Parameter>)> {
        fn go(n: DNode, b: CType) -> (Option<String>, CType, Vec<Parameter>) {
            match n {
                DNode::Name(x) => (x, b, vec![]),
                DNode::Pointer(c, q) => {
                    let mut t = CType::pointer(b);
                    t.qualifiers = q;
                    go(*c, t)
                }
                DNode::Array(c, s) => go(
                    *c,
                    CType::new(TypeKind::Array {
                        element: Box::new(b),
                        size: s,
                    }),
                ),
                DNode::Function(c, p, v, has_prototype) => {
                    let t = CType::new(TypeKind::Function {
                        return_type: Box::new(b),
                        params: p.clone(),
                        variadic: v,
                        has_prototype,
                    });
                    let (n, t, _) = go(*c, t);
                    (n, t, p)
                }
            }
        }
        Ok(go(node, base))
    }
    fn type_name(&mut self) -> PResult<CType> {
        let s = self.declaration_specifiers()?;
        let node = self.declarator(true)?;
        let (_, t, _) = self.apply_declarator(node, s.ty)?;
        Ok(t)
    }

    fn initializer(&mut self) -> PResult<Initializer> {
        if self.eat(&TokenKind::LBrace).is_none() {
            return Ok(Initializer::Expression(self.assignment_expression()?));
        }
        let mut items = vec![];
        while !self.at(&TokenKind::RBrace) {
            let mut ds = vec![];
            loop {
                if self.eat(&TokenKind::Dot).is_some() {
                    let t = self.bump();
                    let TokenKind::Identifier(n) = t.kind else {
                        return Err(self.err("expected field designator"));
                    };
                    ds.push(Designator::Field(n))
                } else if self.eat(&TokenKind::LBracket).is_some() {
                    let e = self.assignment_expression()?;
                    self.expect(&TokenKind::RBracket)?;
                    ds.push(Designator::Index(e))
                } else {
                    break;
                }
            }
            if !ds.is_empty() {
                self.expect(&TokenKind::Assign)?;
            }
            let value = self.initializer()?;
            items.push(InitializerItem {
                designators: ds,
                value,
            });
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Initializer::List(items))
    }
    fn static_assert(&mut self) -> PResult<StaticAssertion> {
        let start = self.bump().span;
        self.expect(&TokenKind::LParen)?;
        let e = self.assignment_expression()?;
        self.expect(&TokenKind::Comma)?;
        let message = match self.bump().kind {
            TokenKind::Literal(Literal::String { value: message, .. }) => message,
            _ => return Err(self.err("static assertion requires a string literal")),
        };
        self.expect(&TokenKind::RParen)?;
        let semi = self.expect(&TokenKind::Semi)?;
        if self.const_int(&e) == Some(0) {
            return Err(self.sema_err("static assertion failed", e.span));
        }
        Ok(StaticAssertion {
            condition: e,
            message,
            span: start.join(semi.span),
        })
    }

    fn local_declaration(&mut self) -> PResult<Vec<Declaration>> {
        let start = self.peek().span;
        let spec = self.declaration_specifiers()?;
        if self.eat(&TokenKind::Semi).is_some() {
            return Ok(vec![Declaration {
                name: None,
                ty: spec.ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                span: start.join(self.previous().span),
            }]);
        }
        let mut out = vec![];
        loop {
            let n = self.declarator(false)?;
            let (name, ty, _) = self.apply_declarator(n, spec.ty.clone())?;
            let d = self.finish_declaration(start, spec.clone(), name, ty)?;
            self.declare(&d)?;
            out.push(d);
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(&TokenKind::Semi)?;
        Ok(out)
    }

    fn compound_statement(&mut self, create_scope: bool) -> PResult<Statement> {
        let l = self.expect(&TokenKind::LBrace)?;
        if create_scope {
            self.enter_scope()
        }
        let mut items = vec![];
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            if self.kw(Keyword::StaticAssert) {
                items.push(BlockItem::StaticAssert(self.static_assert()?));
                continue;
            }
            if self.is_declaration_start() {
                for d in self.local_declaration()? {
                    items.push(BlockItem::Declaration(d))
                }
            } else {
                items.push(BlockItem::Statement(self.statement()?))
            }
        }
        let r = self.expect(&TokenKind::RBrace)?;
        if create_scope {
            self.leave_scope()
        }
        Ok(Statement {
            kind: StatementKind::Compound(items),
            span: l.span.join(r.span),
        })
    }
    fn is_declaration_start(&self) -> bool {
        self.is_type_start()
            || matches!(
                self.peek().kind,
                TokenKind::Keyword(
                    Keyword::Typedef
                        | Keyword::Extern
                        | Keyword::Static
                        | Keyword::Auto
                        | Keyword::Register
                        | Keyword::ThreadLocal
                        | Keyword::Alignas
                        | Keyword::Inline
                        | Keyword::Noreturn
                )
            )
    }
    fn statement(&mut self) -> PResult<Statement> {
        let start = self.peek().span;
        if self.at(&TokenKind::LBrace) {
            return self.compound_statement(true);
        }
        if self.eat(&TokenKind::Semi).is_some() {
            return Ok(Statement {
                kind: StatementKind::Empty,
                span: start.join(self.previous().span),
            });
        }
        if self.eat_kw(Keyword::If).is_some() {
            self.expect(&TokenKind::LParen)?;
            let c = self.expression()?;
            self.require_scalar(&c, "if condition")?;
            self.expect(&TokenKind::RParen)?;
            let then_branch = Box::new(self.statement()?);
            let else_branch = if self.eat_kw(Keyword::Else).is_some() {
                Some(Box::new(self.statement()?))
            } else {
                None
            };
            let end = else_branch.as_ref().map_or(then_branch.span, |x| x.span);
            return Ok(Statement {
                kind: StatementKind::If {
                    condition: c,
                    then_branch,
                    else_branch,
                },
                span: start.join(end),
            });
        }
        if self.eat_kw(Keyword::Switch).is_some() {
            self.expect(&TokenKind::LParen)?;
            let e = self.expression()?;
            if !e.ty.is_integer() {
                return Err(self.sema_err("switch expression must have integer type", e.span));
            }
            self.expect(&TokenKind::RParen)?;
            self.switch_depth += 1;
            let body = Box::new(self.statement()?);
            self.switch_depth -= 1;
            let span = start.join(body.span);
            return Ok(Statement {
                kind: StatementKind::Switch {
                    expression: e,
                    body,
                },
                span,
            });
        }
        if self.eat_kw(Keyword::While).is_some() {
            self.expect(&TokenKind::LParen)?;
            let c = self.expression()?;
            self.require_scalar(&c, "while condition")?;
            self.expect(&TokenKind::RParen)?;
            self.loop_depth += 1;
            let body = Box::new(self.statement()?);
            self.loop_depth -= 1;
            let span = start.join(body.span);
            return Ok(Statement {
                kind: StatementKind::While { condition: c, body },
                span,
            });
        }
        if self.eat_kw(Keyword::Do).is_some() {
            self.loop_depth += 1;
            let body = Box::new(self.statement()?);
            self.loop_depth -= 1;
            self.eat_kw(Keyword::While)
                .ok_or_else(|| self.err("expected while after do body"))?;
            self.expect(&TokenKind::LParen)?;
            let c = self.expression()?;
            self.require_scalar(&c, "do-while condition")?;
            self.expect(&TokenKind::RParen)?;
            let semi = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::DoWhile { body, condition: c },
                span: start.join(semi.span),
            });
        }
        if self.eat_kw(Keyword::For).is_some() {
            self.expect(&TokenKind::LParen)?;
            self.enter_scope();
            let init = if self.is_declaration_start() {
                ForInit::Declaration(self.local_declaration()?)
            } else {
                let e = if self.at(&TokenKind::Semi) {
                    None
                } else {
                    Some(self.expression()?)
                };
                self.expect(&TokenKind::Semi)?;
                ForInit::Expression(e)
            };
            let condition = if self.at(&TokenKind::Semi) {
                None
            } else {
                let e = self.expression()?;
                self.require_scalar(&e, "for condition")?;
                Some(e)
            };
            self.expect(&TokenKind::Semi)?;
            let step = if self.at(&TokenKind::RParen) {
                None
            } else {
                Some(self.expression()?)
            };
            self.expect(&TokenKind::RParen)?;
            self.loop_depth += 1;
            let body = Box::new(self.statement()?);
            self.loop_depth -= 1;
            self.leave_scope();
            let span = start.join(body.span);
            return Ok(Statement {
                kind: StatementKind::For {
                    init,
                    condition,
                    step,
                    body,
                },
                span,
            });
        }
        if self.eat_kw(Keyword::Goto).is_some() {
            let t = self.bump();
            let TokenKind::Identifier(n) = t.kind else {
                return Err(self.err("expected label after goto"));
            };
            let s = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::Goto(n),
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Continue).is_some() {
            if self.loop_depth == 0 {
                return Err(self.sema_err("continue is only valid in a loop", start));
            }
            let s = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::Continue,
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Break).is_some() {
            if self.loop_depth == 0 && self.switch_depth == 0 {
                return Err(self.sema_err("break is only valid in a loop or switch", start));
            }
            let s = self.expect(&TokenKind::Semi)?;
            return Ok(Statement {
                kind: StatementKind::Break,
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Return).is_some() {
            let e = if self.at(&TokenKind::Semi) {
                None
            } else {
                Some(self.expression()?)
            };
            let s = self.expect(&TokenKind::Semi)?;
            let ret = self
                .current_return
                .clone()
                .ok_or_else(|| self.sema_err("return outside a function", start))?;
            match (&ret.kind, &e) {
                (TypeKind::Void, None) => {}
                (TypeKind::Void, Some(x)) => {
                    return Err(self.sema_err("void function cannot return a value", x.span));
                }
                (_, None) => {
                    return Err(self.sema_err("non-void function must return a value", start));
                }
                (_, Some(x)) => self.require_assignable(&ret, x)?,
            }
            return Ok(Statement {
                kind: StatementKind::Return(e),
                span: start.join(s.span),
            });
        }
        if self.eat_kw(Keyword::Case).is_some() {
            if self.switch_depth == 0 {
                return Err(self.sema_err("case outside switch", start));
            }
            let e = self.assignment_expression()?;
            if self.const_int(&e).is_none() {
                return Err(self.sema_err("case value must be an integer constant", e.span));
            }
            self.expect(&TokenKind::Colon)?;
            let st = Box::new(self.statement()?);
            let span = start.join(st.span);
            return Ok(Statement {
                kind: StatementKind::Case {
                    value: e,
                    statement: st,
                },
                span,
            });
        }
        if self.eat_kw(Keyword::Default).is_some() {
            if self.switch_depth == 0 {
                return Err(self.sema_err("default outside switch", start));
            }
            self.expect(&TokenKind::Colon)?;
            let st = Box::new(self.statement()?);
            let span = start.join(st.span);
            return Ok(Statement {
                kind: StatementKind::Default { statement: st },
                span,
            });
        }
        if let TokenKind::Identifier(n) = self.peek().kind.clone()
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| matches!(t.kind, TokenKind::Colon))
        {
            self.bump();
            self.bump();
            let st = Box::new(self.statement()?);
            let span = start.join(st.span);
            return Ok(Statement {
                kind: StatementKind::Label {
                    name: n,
                    statement: st,
                },
                span,
            });
        }
        let e = self.expression()?;
        let semi = self.expect(&TokenKind::Semi)?;
        Ok(Statement {
            kind: StatementKind::Expression(e),
            span: start.join(semi.span),
        })
    }

    fn expression(&mut self) -> PResult<Expression> {
        let first = self.assignment_expression()?;
        if self.eat(&TokenKind::Comma).is_none() {
            return Ok(first);
        }
        let start = first.span;
        let mut xs = vec![first];
        loop {
            xs.push(self.assignment_expression()?);
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        let ty = xs.last().unwrap().ty.clone();
        let span = start.join(xs.last().unwrap().span);
        Ok(Expression {
            kind: ExpressionKind::Comma(xs),
            ty,
            category: ValueCategory::RValue,
            span,
        })
    }
    fn assignment_expression(&mut self) -> PResult<Expression> {
        let left = self.conditional_expression()?;
        let op = match self.peek().kind {
            TokenKind::Assign => AssignOp::Assign,
            TokenKind::PlusEq => AssignOp::Add,
            TokenKind::MinusEq => AssignOp::Subtract,
            TokenKind::StarEq => AssignOp::Multiply,
            TokenKind::SlashEq => AssignOp::Divide,
            TokenKind::PercentEq => AssignOp::Remainder,
            TokenKind::ShlEq => AssignOp::ShiftLeft,
            TokenKind::ShrEq => AssignOp::ShiftRight,
            TokenKind::AmpEq => AssignOp::BitAnd,
            TokenKind::CaretEq => AssignOp::BitXor,
            TokenKind::PipeEq => AssignOp::BitOr,
            _ => return Ok(left),
        };
        self.bump();
        if left.category != ValueCategory::LValue || left.ty.qualifiers.is_const {
            return Err(self.sema_err("assignment requires a modifiable lvalue", left.span));
        }
        let right = self.assignment_expression()?;
        self.require_assignable(&left.ty, &right)?;
        let span = left.span.join(right.span);
        let ty = left.ty.clone();
        Ok(Expression {
            kind: ExpressionKind::Assignment {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            category: ValueCategory::RValue,
            span,
        })
    }
    fn conditional_expression(&mut self) -> PResult<Expression> {
        let c = self.binary_expression(1)?;
        if self.eat(&TokenKind::Question).is_none() {
            return Ok(c);
        }
        self.require_scalar(&c, "conditional operand")?;
        let a = self.expression()?;
        self.expect(&TokenKind::Colon)?;
        let b = self.conditional_expression()?;
        let ty = self.common_type(&a.ty, &b.ty).ok_or_else(|| {
            self.sema_err("incompatible conditional operands", a.span.join(b.span))
        })?;
        let span = c.span.join(b.span);
        Ok(Expression {
            kind: ExpressionKind::Conditional {
                condition: Box::new(c),
                then_expr: Box::new(a),
                else_expr: Box::new(b),
            },
            ty,
            category: ValueCategory::RValue,
            span,
        })
    }

    fn binary_info(k: &TokenKind) -> Option<(u8, BinaryOp)> {
        use BinaryOp::*;
        use TokenKind::*;
        Some(match k {
            Or => (1, LogicalOr),
            And => (2, LogicalAnd),
            Pipe => (3, BitOr),
            Caret => (4, BitXor),
            Amp => (5, BitAnd),
            Eq => (6, Equal),
            Ne => (6, NotEqual),
            Lt => (7, Less),
            Le => (7, LessEqual),
            Gt => (7, Greater),
            Ge => (7, GreaterEqual),
            Shl => (8, ShiftLeft),
            Shr => (8, ShiftRight),
            Plus => (9, Add),
            Minus => (9, Subtract),
            Star => (10, Multiply),
            Slash => (10, Divide),
            Percent => (10, Remainder),
            _ => return None,
        })
    }
    fn binary_expression(&mut self, min: u8) -> PResult<Expression> {
        let mut lhs = self.cast_expression()?;
        while let Some((prec, op)) = Self::binary_info(&self.peek().kind) {
            if prec < min {
                break;
            }
            self.bump();
            let rhs = self.binary_expression(prec + 1)?;
            lhs = self.make_binary(op, lhs, rhs)?
        }
        Ok(lhs)
    }
    fn make_binary(
        &self,
        op: BinaryOp,
        left: Expression,
        right: Expression,
    ) -> PResult<Expression> {
        use BinaryOp::*;
        let l = left.ty.decay();
        let r = right.ty.decay();
        let ty = match op {
            LogicalAnd | LogicalOr => {
                if !l.is_scalar() || !r.is_scalar() {
                    return Err(self.sema_err(
                        "logical operands must be scalar",
                        left.span.join(right.span),
                    ));
                }
                CType::int()
            }
            Less | LessEqual | Greater | GreaterEqual | Equal | NotEqual => {
                if !(l.is_arithmetic() && r.is_arithmetic()
                    || matches!(l.kind, TypeKind::Pointer(_))
                        && matches!(r.kind, TypeKind::Pointer(_)))
                {
                    return Err(
                        self.sema_err("invalid comparison operands", left.span.join(right.span))
                    );
                }
                CType::int()
            }
            Add | Subtract if matches!(l.kind, TypeKind::Pointer(_)) && r.is_integer() => l.clone(),
            Subtract
                if matches!(l.kind, TypeKind::Pointer(_))
                    && matches!(r.kind, TypeKind::Pointer(_)) =>
            {
                CType::new(TypeKind::Long { signed: true })
            }
            Add if l.is_integer() && matches!(r.kind, TypeKind::Pointer(_)) => r.clone(),
            ShiftLeft | ShiftRight | Remainder | BitAnd | BitXor | BitOr => {
                if !l.is_integer() || !r.is_integer() {
                    return Err(self.sema_err(
                        "operator requires integer operands",
                        left.span.join(right.span),
                    ));
                }
                self.usual_arithmetic(&l, &r)
            }
            Multiply | Divide | Add | Subtract => {
                if !l.is_arithmetic() || !r.is_arithmetic() {
                    return Err(self.sema_err(
                        "operator requires arithmetic operands",
                        left.span.join(right.span),
                    ));
                }
                self.usual_arithmetic(&l, &r)
            }
        };
        let span = left.span.join(right.span);
        Ok(Expression {
            kind: ExpressionKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            category: ValueCategory::RValue,
            span,
        })
    }

    fn cast_expression(&mut self) -> PResult<Expression> {
        if self.at(&TokenKind::LParen) {
            let save = self.pos;
            let l = self.bump();
            if self.is_type_start()
                && let Ok(ty) = self.type_name()
                && self.eat(&TokenKind::RParen).is_some()
            {
                if self.at(&TokenKind::LBrace) {
                    let init = self.initializer()?;
                    let span = l.span.join(self.previous().span);
                    return Ok(Expression {
                        kind: ExpressionKind::CompoundLiteral {
                            ty: ty.clone(),
                            initializer: Box::new(init),
                        },
                        ty,
                        category: ValueCategory::LValue,
                        span,
                    });
                }
                let e = self.cast_expression()?;
                if matches!(ty.kind, TypeKind::Array { .. } | TypeKind::Function { .. }) {
                    return Err(self.sema_err("cast target must be a scalar type", l.span));
                }
                let span = l.span.join(e.span);
                return Ok(Expression {
                    kind: ExpressionKind::Cast {
                        target: ty.clone(),
                        expression: Box::new(e),
                    },
                    ty,
                    category: ValueCategory::RValue,
                    span,
                });
            }
            self.pos = save;
        }
        self.unary_expression()
    }
    fn unary_expression(&mut self) -> PResult<Expression> {
        let start = self.peek().span;
        if self.eat_kw(Keyword::Sizeof).is_some() {
            if self.eat(&TokenKind::LParen).is_some() {
                let save = self.pos;
                if self.is_type_start() {
                    let ty = self.type_name()?;
                    let r = self.expect(&TokenKind::RParen)?;
                    return Ok(Expression {
                        kind: ExpressionKind::SizeofType(ty),
                        ty: CType::new(TypeKind::Long { signed: false }),
                        category: ValueCategory::RValue,
                        span: start.join(r.span),
                    });
                }
                self.pos = save;
                let e = self.expression()?;
                let r = self.expect(&TokenKind::RParen)?;
                return Ok(Expression {
                    kind: ExpressionKind::SizeofExpression(Box::new(e)),
                    ty: CType::new(TypeKind::Long { signed: false }),
                    category: ValueCategory::RValue,
                    span: start.join(r.span),
                });
            }
            let e = self.unary_expression()?;
            let span = start.join(e.span);
            return Ok(Expression {
                kind: ExpressionKind::SizeofExpression(Box::new(e)),
                ty: CType::new(TypeKind::Long { signed: false }),
                category: ValueCategory::RValue,
                span,
            });
        }
        if self.eat_kw(Keyword::Alignof).is_some() {
            self.expect(&TokenKind::LParen)?;
            let ty = self.type_name()?;
            let r = self.expect(&TokenKind::RParen)?;
            return Ok(Expression {
                kind: ExpressionKind::Alignof(ty),
                ty: CType::new(TypeKind::Long { signed: false }),
                category: ValueCategory::RValue,
                span: start.join(r.span),
            });
        }
        let op = match self.peek().kind {
            TokenKind::Plus => Some(UnaryOp::Plus),
            TokenKind::Minus => Some(UnaryOp::Minus),
            TokenKind::Bang => Some(UnaryOp::LogicalNot),
            TokenKind::Tilde => Some(UnaryOp::BitNot),
            TokenKind::Amp => Some(UnaryOp::AddressOf),
            TokenKind::Star => Some(UnaryOp::Dereference),
            TokenKind::Inc => Some(UnaryOp::PreIncrement),
            TokenKind::Dec => Some(UnaryOp::PreDecrement),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let e = self.cast_expression()?;
            let (ty, cat) = match op {
                UnaryOp::AddressOf => {
                    if e.category == ValueCategory::RValue {
                        return Err(
                            self.sema_err("address-of requires an lvalue or function", e.span)
                        );
                    }
                    (CType::pointer(e.ty.clone()), ValueCategory::RValue)
                }
                UnaryOp::Dereference => match e.ty.decay().kind {
                    TypeKind::Pointer(to) => {
                        let c = if matches!(to.kind, TypeKind::Function { .. }) {
                            ValueCategory::Function
                        } else {
                            ValueCategory::LValue
                        };
                        (*to, c)
                    }
                    _ => return Err(self.sema_err("dereference requires a pointer", e.span)),
                },
                UnaryOp::LogicalNot => {
                    self.require_scalar(&e, "logical not")?;
                    (CType::int(), ValueCategory::RValue)
                }
                UnaryOp::BitNot => {
                    if !e.ty.is_integer() {
                        return Err(self.sema_err("bitwise not requires integer operand", e.span));
                    }
                    (self.integer_promote(&e.ty), ValueCategory::RValue)
                }
                UnaryOp::Plus | UnaryOp::Minus => {
                    if !e.ty.is_arithmetic() {
                        return Err(self.sema_err(
                            "unary arithmetic operator requires arithmetic operand",
                            e.span,
                        ));
                    }
                    (self.integer_promote(&e.ty), ValueCategory::RValue)
                }
                UnaryOp::PreIncrement | UnaryOp::PreDecrement => {
                    if e.category != ValueCategory::LValue
                        || !e.ty.is_scalar()
                        || e.ty.qualifiers.is_const
                    {
                        return Err(
                            self.sema_err("increment requires modifiable scalar lvalue", e.span)
                        );
                    }
                    (e.ty.clone(), ValueCategory::RValue)
                }
            };
            let span = start.join(e.span);
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    op,
                    operand: Box::new(e),
                },
                ty,
                category: cat,
                span,
            });
        }
        self.postfix_expression()
    }
    fn postfix_expression(&mut self) -> PResult<Expression> {
        let mut e = self.primary_expression()?;
        loop {
            if self.eat(&TokenKind::LBracket).is_some() {
                let i = self.expression()?;
                let r = self.expect(&TokenKind::RBracket)?;
                let base = e.ty.decay();
                let ity = i.ty.decay();
                let elem = match (&base.kind, &ity.kind) {
                    (TypeKind::Pointer(x), _) if ity.is_integer() => (**x).clone(),
                    (_, TypeKind::Pointer(x)) if base.is_integer() => (**x).clone(),
                    _ => {
                        return Err(self.sema_err(
                            "subscript requires a pointer and integer",
                            e.span.join(i.span),
                        ));
                    }
                };
                let span = e.span.join(r.span);
                e = Expression {
                    kind: ExpressionKind::Subscript {
                        base: Box::new(e),
                        index: Box::new(i),
                    },
                    ty: elem,
                    category: ValueCategory::LValue,
                    span,
                };
                continue;
            }
            if self.eat(&TokenKind::LParen).is_some() {
                let mut args = vec![];
                if !self.at(&TokenKind::RParen) {
                    loop {
                        args.push(self.assignment_expression()?);
                        if self.eat(&TokenKind::Comma).is_none() {
                            break;
                        }
                    }
                }
                let r = self.expect(&TokenKind::RParen)?;
                let fty = e.ty.decay();
                let TypeKind::Pointer(target) = fty.kind else {
                    return Err(self.sema_err("called object is not a function", e.span));
                };
                let TypeKind::Function {
                    return_type,
                    params,
                    variadic,
                    has_prototype,
                } = target.kind
                else {
                    return Err(self.sema_err("called object is not a function", e.span));
                };
                if has_prototype
                    && (args.len() < params.len() || (!variadic && args.len() != params.len()))
                {
                    return Err(self.sema_err(
                        format!(
                            "function expects {} argument(s), got {}",
                            params.len(),
                            args.len()
                        ),
                        e.span.join(r.span),
                    ));
                }
                if has_prototype {
                    for (p, a) in params.iter().zip(args.iter()) {
                        self.require_assignable(&p.ty, a)?
                    }
                }
                let span = e.span.join(r.span);
                e = Expression {
                    kind: ExpressionKind::Call {
                        callee: Box::new(e),
                        arguments: args,
                    },
                    ty: *return_type,
                    category: ValueCategory::RValue,
                    span,
                };
                continue;
            }
            let indirect = if self.eat(&TokenKind::Dot).is_some() {
                false
            } else if self.eat(&TokenKind::Arrow).is_some() {
                true
            } else {
                if self.eat(&TokenKind::Inc).is_some() || self.eat(&TokenKind::Dec).is_some() {
                    let dec = matches!(self.previous().kind, TokenKind::Dec);
                    if e.category != ValueCategory::LValue
                        || !e.ty.is_scalar()
                        || e.ty.qualifiers.is_const
                    {
                        return Err(self.sema_err(
                            "postfix increment requires modifiable scalar lvalue",
                            e.span,
                        ));
                    }
                    let span = e.span.join(self.previous().span);
                    let ty = e.ty.clone();
                    e = Expression {
                        kind: ExpressionKind::PostIncrement {
                            operand: Box::new(e),
                            decrement: dec,
                        },
                        ty,
                        category: ValueCategory::RValue,
                        span,
                    };
                    continue;
                }
                break;
            };
            let t = self.bump();
            let TokenKind::Identifier(field) = t.kind else {
                return Err(self.err("expected member name"));
            };
            let record_ty = if indirect {
                match e.ty.decay().kind {
                    TypeKind::Pointer(x) => *x,
                    _ => {
                        return Err(
                            self.sema_err("arrow requires pointer to struct or union", e.span)
                        );
                    }
                }
            } else {
                e.ty.clone()
            };
            let fields = match record_ty.kind {
                TypeKind::Struct {
                    fields: Some(f), ..
                }
                | TypeKind::Union {
                    fields: Some(f), ..
                } => f,
                _ => {
                    return Err(
                        self.sema_err("member access requires a complete struct or union", e.span)
                    );
                }
            };
            let f = fields
                .iter()
                .find(|x| x.name.as_deref() == Some(&field))
                .ok_or_else(|| self.sema_err(format!("no member named '{field}'"), t.span))?;
            let span = e.span.join(t.span);
            e = Expression {
                kind: ExpressionKind::Member {
                    base: Box::new(e),
                    field,
                    indirect,
                },
                ty: f.ty.clone(),
                category: ValueCategory::LValue,
                span,
            };
        }
        Ok(e)
    }
    fn primary_expression(&mut self) -> PResult<Expression> {
        let t = self.bump();
        match t.kind {
            TokenKind::Identifier(n) => {
                let ty = self.lookup(&n).ok_or_else(|| {
                    self.sema_err(format!("use of undeclared identifier '{n}'"), t.span)
                })?;
                let category = if matches!(ty.kind, TypeKind::Function { .. }) {
                    ValueCategory::Function
                } else {
                    ValueCategory::LValue
                };
                Ok(Expression {
                    kind: ExpressionKind::Identifier(n),
                    ty,
                    category,
                    span: t.span,
                })
            }
            TokenKind::Literal(Literal::Integer(raw)) => {
                let (v, ty) = self
                    .integer_literal(&raw)
                    .ok_or_else(|| self.sema_err("invalid integer literal", t.span))?;
                Ok(Expression {
                    kind: ExpressionKind::Integer(v),
                    ty,
                    category: ValueCategory::RValue,
                    span: t.span,
                })
            }
            TokenKind::Literal(Literal::Floating(raw)) => {
                let clean = raw.trim_end_matches(['f', 'F', 'l', 'L']);
                let v = if clean.starts_with("0x") || clean.starts_with("0X") {
                    parse_hex_float(clean)
                } else {
                    clean.parse().ok()
                }
                .ok_or_else(|| self.sema_err("invalid floating literal", t.span))?;
                let ty = if raw.ends_with(['f', 'F']) {
                    CType::new(TypeKind::Float)
                } else if raw.ends_with(['l', 'L']) {
                    CType::new(TypeKind::LongDouble)
                } else {
                    CType::new(TypeKind::Double)
                };
                Ok(Expression {
                    kind: ExpressionKind::Floating(v),
                    ty,
                    category: ValueCategory::RValue,
                    span: t.span,
                })
            }
            TokenKind::Literal(Literal::Character {
                value: raw,
                encoding,
            }) => {
                let v = self
                    .decode_char(&raw)
                    .ok_or_else(|| self.sema_err("invalid character literal", t.span))?;
                let ty = match encoding {
                    StringEncoding::Narrow | StringEncoding::Wide => CType::int(),
                    StringEncoding::Utf16 => CType::new(TypeKind::Short { signed: false }),
                    StringEncoding::Utf32 => CType::uint(),
                    StringEncoding::Utf8 => {
                        return Err(
                            self.sema_err("u8 character constants are not part of C11", t.span)
                        );
                    }
                };
                Ok(Expression {
                    kind: ExpressionKind::Character { value: v, encoding },
                    ty,
                    category: ValueCategory::RValue,
                    span: t.span,
                })
            }
            TokenKind::Literal(Literal::String {
                value: mut raw,
                mut encoding,
            }) => {
                let mut span = t.span;
                while let TokenKind::Literal(Literal::String {
                    value: s,
                    encoding: next_encoding,
                }) = self.peek().kind.clone()
                {
                    span = span.join(self.bump().span);
                    if encoding == StringEncoding::Narrow {
                        encoding = next_encoding;
                    } else if next_encoding != StringEncoding::Narrow && next_encoding != encoding {
                        return Err(
                            self.sema_err("incompatible adjacent string literal encodings", span)
                        );
                    }
                    raw.push_str(&s)
                }
                let element = match encoding {
                    StringEncoding::Narrow | StringEncoding::Utf8 => {
                        CType::new(TypeKind::Char { signed: None })
                    }
                    StringEncoding::Utf16 => CType::new(TypeKind::Short { signed: false }),
                    StringEncoding::Utf32 => CType::uint(),
                    StringEncoding::Wide => CType::int(),
                };
                let ty = CType::new(TypeKind::Array {
                    element: Box::new(element),
                    size: ArraySize::Constant(raw.chars().count() + 1),
                });
                Ok(Expression {
                    kind: ExpressionKind::String {
                        value: raw,
                        encoding,
                    },
                    ty,
                    category: ValueCategory::LValue,
                    span,
                })
            }
            TokenKind::LParen => {
                let e = self.expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::Keyword(Keyword::Generic) => self.generic_selection(t.span),
            _ => Err(Diagnostic::new(
                ErrorKind::Syntax,
                "expected expression",
                t.span,
            )),
        }
    }
    fn generic_selection(&mut self, start: Span) -> PResult<Expression> {
        self.expect(&TokenKind::LParen)?;
        let controlling = self.assignment_expression()?;
        self.expect(&TokenKind::Comma)?;
        let mut selected = None;
        let mut default = None;
        loop {
            if self.eat_kw(Keyword::Default).is_some() {
                self.expect(&TokenKind::Colon)?;
                default = Some(self.assignment_expression()?)
            } else {
                let ty = self.type_name()?;
                self.expect(&TokenKind::Colon)?;
                let e = self.assignment_expression()?;
                if self.compatible(&controlling.ty.decay(), &ty) {
                    if selected.is_some() {
                        return Err(self.sema_err("multiple matching generic associations", e.span));
                    }
                    selected = Some(e)
                }
            }
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        let r = self.expect(&TokenKind::RParen)?;
        let choice = selected.or(default).ok_or_else(|| {
            self.sema_err(
                "generic selection has no matching association",
                controlling.span,
            )
        })?;
        let ty = choice.ty.clone();
        let category = choice.category;
        Ok(Expression {
            kind: ExpressionKind::GenericSelection {
                controlling: Box::new(controlling),
                selected: Box::new(choice),
            },
            ty,
            category,
            span: start.join(r.span),
        })
    }

    fn require_scalar(&self, e: &Expression, where_: &str) -> PResult<()> {
        if e.ty.decay().is_scalar() {
            Ok(())
        } else {
            Err(self.sema_err(format!("{where_} requires scalar type"), e.span))
        }
    }
    fn require_assignable(&self, to: &CType, from: &Expression) -> PResult<()> {
        let f = from.ty.decay();
        if self.compatible(to, &f)
            || to.is_arithmetic() && f.is_arithmetic()
            || matches!(to.kind, TypeKind::Pointer(_))
                && (matches!(f.kind, TypeKind::Pointer(_)) || self.const_int(from) == Some(0))
        {
            Ok(())
        } else {
            Err(self.sema_err("incompatible assignment", from.span))
        }
    }
    fn compatible(&self, a: &CType, b: &CType) -> bool {
        match (&a.kind, &b.kind) {
            (TypeKind::Void, TypeKind::Void)
            | (TypeKind::Bool, TypeKind::Bool)
            | (TypeKind::Float, TypeKind::Float)
            | (TypeKind::Double, TypeKind::Double)
            | (TypeKind::LongDouble, TypeKind::LongDouble) => true,
            (TypeKind::Char { signed: x }, TypeKind::Char { signed: y }) => x == y,
            (TypeKind::Complex(x), TypeKind::Complex(y))
            | (TypeKind::Imaginary(x), TypeKind::Imaginary(y)) => self.compatible(x, y),
            (TypeKind::Short { signed: x }, TypeKind::Short { signed: y })
            | (TypeKind::Int { signed: x }, TypeKind::Int { signed: y })
            | (TypeKind::Long { signed: x }, TypeKind::Long { signed: y })
            | (TypeKind::LongLong { signed: x }, TypeKind::LongLong { signed: y }) => x == y,
            (TypeKind::Pointer(x), TypeKind::Pointer(y)) => {
                self.compatible(x, y)
                    || matches!(x.kind, TypeKind::Void)
                    || matches!(y.kind, TypeKind::Void)
            }
            (
                TypeKind::Array {
                    element: x,
                    size: sx,
                },
                TypeKind::Array {
                    element: y,
                    size: sy,
                },
            ) => {
                self.compatible(x, y)
                    && (sx == sy
                        || matches!(sx, ArraySize::Unspecified)
                        || matches!(sy, ArraySize::Unspecified))
            }
            (TypeKind::Struct { name: x, .. }, TypeKind::Struct { name: y, .. })
            | (TypeKind::Union { name: x, .. }, TypeKind::Union { name: y, .. })
            | (TypeKind::Enum { name: x, .. }, TypeKind::Enum { name: y, .. }) => {
                x.is_some() && x == y
            }
            (
                TypeKind::Function {
                    return_type: x,
                    params: px,
                    variadic: vx,
                    has_prototype: hx,
                },
                TypeKind::Function {
                    return_type: y,
                    params: py,
                    variadic: vy,
                    has_prototype: hy,
                },
            ) => {
                self.compatible(x, y)
                    && vx == vy
                    && hx == hy
                    && px.len() == py.len()
                    && px
                        .iter()
                        .zip(py)
                        .all(|(a, b)| self.compatible(&a.ty, &b.ty))
            }
            _ => false,
        }
    }
    fn common_type(&self, a: &CType, b: &CType) -> Option<CType> {
        if a.is_arithmetic() && b.is_arithmetic() {
            Some(self.usual_arithmetic(a, b))
        } else if self.compatible(a, b)
            || matches!(a.kind, TypeKind::Pointer(_)) && matches!(b.kind, TypeKind::Pointer(_))
        {
            Some(a.clone())
        } else {
            None
        }
    }
    fn integer_promote(&self, t: &CType) -> CType {
        if matches!(
            t.kind,
            TypeKind::Bool | TypeKind::Char { .. } | TypeKind::Short { .. } | TypeKind::Enum { .. }
        ) {
            CType::int()
        } else {
            t.clone()
        }
    }
    fn rank(&self, t: &CType) -> (u8, bool) {
        match t.kind {
            TypeKind::Bool => (0, false),
            TypeKind::Char { signed } => (1, signed.unwrap_or(true)),
            TypeKind::Short { signed } => (2, signed),
            TypeKind::Int { signed } => (3, signed),
            TypeKind::Long { signed } => (4, signed),
            TypeKind::LongLong { signed } => (5, signed),
            _ => (3, true),
        }
    }
    fn usual_arithmetic(&self, a: &CType, b: &CType) -> CType {
        let complex =
            matches!(a.kind, TypeKind::Complex(_)) || matches!(b.kind, TypeKind::Complex(_));
        let imaginary =
            matches!(a.kind, TypeKind::Imaginary(_)) || matches!(b.kind, TypeKind::Imaginary(_));
        if complex || imaginary {
            let real_a = match &a.kind {
                TypeKind::Complex(x) | TypeKind::Imaginary(x) => &**x,
                _ => a,
            };
            let real_b = match &b.kind {
                TypeKind::Complex(x) | TypeKind::Imaginary(x) => &**x,
                _ => b,
            };
            let inner = self.usual_arithmetic(real_a, real_b);
            return CType::new(if complex {
                TypeKind::Complex(Box::new(inner))
            } else {
                TypeKind::Imaginary(Box::new(inner))
            });
        }
        if matches!(a.kind, TypeKind::LongDouble) || matches!(b.kind, TypeKind::LongDouble) {
            return CType::new(TypeKind::LongDouble);
        }
        if matches!(a.kind, TypeKind::Double) || matches!(b.kind, TypeKind::Double) {
            return CType::new(TypeKind::Double);
        }
        if matches!(a.kind, TypeKind::Float) || matches!(b.kind, TypeKind::Float) {
            return CType::new(TypeKind::Float);
        }
        let a = self.integer_promote(a);
        let b = self.integer_promote(b);
        let (ra, sa) = self.rank(&a);
        let (rb, sb) = self.rank(&b);
        let rank = ra.max(rb);
        let signed = if sa == sb {
            sa
        } else {
            (rb < ra || sb) && (ra < rb || sa)
        };
        CType::new(match rank {
            0..=3 => TypeKind::Int { signed },
            4 => TypeKind::Long { signed },
            _ => TypeKind::LongLong { signed },
        })
    }
    fn integer_literal(&self, raw: &str) -> Option<(i128, CType)> {
        let s = raw.trim_end_matches(['u', 'U', 'l', 'L']);
        let value = if s.starts_with("0x") || s.starts_with("0X") {
            i128::from_str_radix(&s[2..], 16).ok()
        } else if s.len() > 1 && s.starts_with('0') {
            i128::from_str_radix(&s[1..], 8).ok()
        } else {
            s.parse().ok()
        }?;
        let suffix = raw[s.len()..].to_ascii_lowercase();
        if !matches!(
            suffix.as_str(),
            "" | "u" | "l" | "ul" | "lu" | "ll" | "ull" | "llu"
        ) {
            return None;
        }
        let decimal =
            !(s.starts_with("0x") || s.starts_with("0X") || s.len() > 1 && s.starts_with('0'));
        let candidates: &[(u8, bool)] = match suffix.as_str() {
            "u" => &[(3, false), (4, false), (5, false)],
            "l" if decimal => &[(4, true), (5, true)],
            "l" => &[(4, true), (4, false), (5, true), (5, false)],
            "ul" | "lu" => &[(4, false), (5, false)],
            "ll" if decimal => &[(5, true)],
            "ll" => &[(5, true), (5, false)],
            "ull" | "llu" => &[(5, false)],
            "" if decimal => &[(3, true), (4, true), (5, true)],
            "" => &[
                (3, true),
                (3, false),
                (4, true),
                (4, false),
                (5, true),
                (5, false),
            ],
            _ => return None,
        };
        let (rank, signed) = candidates.iter().copied().find(|(rank, signed)| {
            let bits = if *rank == 3 { 32 } else { 64 };
            if *signed {
                value < (1i128 << (bits - 1))
            } else {
                value < (1i128 << bits)
            }
        })?;
        let ty = CType::new(match rank {
            3 => TypeKind::Int { signed },
            4 => TypeKind::Long { signed },
            _ => TypeKind::LongLong { signed },
        });
        Some((value, ty))
    }
    fn decode_char(&self, s: &str) -> Option<i64> {
        let mut cs = s.chars();
        let c = cs.next()?;
        if c != '\\' {
            return Some(c as i64);
        }
        Some(match cs.next()? {
            'n' => 10,
            'r' => 13,
            't' => 9,
            '0' => 0,
            '\\' => 92,
            '\'' => 39,
            '"' => 34,
            x => x as i64,
        })
    }
    fn const_int(&self, e: &Expression) -> Option<i128> {
        use BinaryOp::*;
        match &e.kind {
            ExpressionKind::Integer(x) => Some(*x),
            ExpressionKind::Identifier(name) => self
                .constants
                .iter()
                .rev()
                .find_map(|scope| scope.get(name).copied()),
            ExpressionKind::Character { value, .. } => Some(*value as i128),
            ExpressionKind::Unary { op, operand } => {
                let x = self.const_int(operand)?;
                Some(match op {
                    UnaryOp::Plus => x,
                    UnaryOp::Minus => -x,
                    UnaryOp::LogicalNot => (x == 0) as i128,
                    UnaryOp::BitNot => !x,
                    _ => return None,
                })
            }
            ExpressionKind::Binary { op, left, right } => {
                let a = self.const_int(left)?;
                if *op == LogicalAnd && a == 0 {
                    return Some(0);
                }
                if *op == LogicalOr && a != 0 {
                    return Some(1);
                }
                let b = self.const_int(right)?;
                Some(match op {
                    Multiply => a * b,
                    Divide => a.checked_div(b)?,
                    Remainder => a.checked_rem(b)?,
                    Add => a + b,
                    Subtract => a - b,
                    ShiftLeft => a << b,
                    ShiftRight => a >> b,
                    Less => (a < b) as i128,
                    LessEqual => (a <= b) as i128,
                    Greater => (a > b) as i128,
                    GreaterEqual => (a >= b) as i128,
                    Equal => (a == b) as i128,
                    NotEqual => (a != b) as i128,
                    BitAnd => a & b,
                    BitXor => a ^ b,
                    BitOr => a | b,
                    LogicalAnd => (a != 0 && b != 0) as i128,
                    LogicalOr => (a != 0 || b != 0) as i128,
                })
            }
            ExpressionKind::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.const_int(condition)? != 0 {
                    self.const_int(then_expr)
                } else {
                    self.const_int(else_expr)
                }
            }
            ExpressionKind::Cast { expression, .. } => self.const_int(expression),
            ExpressionKind::SizeofType(t) => Some(self.sizeof(t) as i128),
            ExpressionKind::SizeofExpression(x) => Some(self.sizeof(&x.ty) as i128),
            ExpressionKind::Alignof(t) => Some(self.alignof(t) as i128),
            _ => None,
        }
    }
    fn sizeof(&self, t: &CType) -> usize {
        self.layout(t).0
    }
    fn alignof(&self, t: &CType) -> usize {
        self.layout(t).1
    }
    fn layout(&self, t: &CType) -> (usize, usize) {
        match &t.kind {
            TypeKind::Bool | TypeKind::Char { .. } => (1, 1),
            TypeKind::Short { .. } => (2, 2),
            TypeKind::Int { .. } | TypeKind::Float | TypeKind::Enum { .. } => (4, 4),
            TypeKind::Long { .. }
            | TypeKind::LongLong { .. }
            | TypeKind::Double
            | TypeKind::Pointer(_) => (8, 8),
            TypeKind::LongDouble => (16, 16),
            TypeKind::Complex(inner) | TypeKind::Imaginary(inner) => {
                let (size, align) = self.layout(inner);
                (size * 2, align)
            }
            TypeKind::Array { element, size } => {
                let (element_size, align) = self.layout(element);
                let count = match size {
                    ArraySize::Constant(size) => *size,
                    _ => 0,
                };
                (element_size * count, align)
            }
            TypeKind::Struct {
                fields: Some(fields),
                ..
            } => {
                let mut offset = 0;
                let mut aggregate_align = 1;
                for field in fields {
                    let (size, align) = self.layout(&field.ty);
                    aggregate_align = aggregate_align.max(align);
                    offset = align_up(offset, align);
                    offset += size;
                }
                (align_up(offset, aggregate_align), aggregate_align)
            }
            TypeKind::Union {
                fields: Some(fields),
                ..
            } => {
                let size = fields
                    .iter()
                    .map(|field| self.layout(&field.ty).0)
                    .max()
                    .unwrap_or(0);
                let align = fields
                    .iter()
                    .map(|field| self.layout(&field.ty).1)
                    .max()
                    .unwrap_or(1);
                (align_up(size, align), align)
            }
            _ => (0, 1),
        }
    }
}

fn align_up(value: usize, alignment: usize) -> usize {
    value.div_ceil(alignment) * alignment
}

fn parse_hex_float(raw: &str) -> Option<f64> {
    let raw = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"))?;
    let (mantissa, exponent) = raw.split_once(['p', 'P'])?;
    let exponent: i32 = exponent.parse().ok()?;
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let whole = if whole.is_empty() {
        0.0
    } else {
        u128::from_str_radix(whole, 16).ok()? as f64
    };
    let mut scale = 1.0 / 16.0;
    let mut value = whole;
    for digit in fraction.chars() {
        value += digit.to_digit(16)? as f64 * scale;
        scale /= 16.0;
    }
    Some(value * 2f64.powi(exponent))
}
