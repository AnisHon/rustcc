use super::parser_core::{DNode, DeclSpec, PResult, Parser};
use crate::lex::token::{Keyword, Literal, TokenKind};
use crate::parser::ast::*;
use crate::source::SourceRange;

impl Parser {
    pub(crate) fn finish_declaration(
        &mut self,
        start: SourceRange,
        spec: DeclSpec,
        name: Option<String>,
        mut ty: CType,
    ) -> PResult<Declaration> {
        self.sema.validate_type(&ty, start)?;
        let mut initializer = if self.eat(&TokenKind::Assign).is_some() {
            Some(self.initializer()?)
        } else {
            None
        };
        if let Some(initializer) = &mut initializer {
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
            range: start.join(self.previous().range),
        })
    }

    pub(crate) fn check_initializer(
        &self,
        target: &CType,
        initializer: &mut Initializer,
    ) -> PResult<()> {
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
                if self.sema.compatible(element, source) {
                    Ok(())
                } else {
                    Err(self.sema_err(
                        "string literal encoding does not match array element type",
                        expression.range,
                    ))
                }
            }
            (Initializer::Expression(expression), _) => {
                *expression = self
                    .sema
                    .assignment_conversion(target, expression.clone())?;
                Ok(())
            }
            (Initializer::List(items), TypeKind::Array { element, size }) => {
                if matches!(size, ArraySize::Constant(size) if items.len() > *size) {
                    return Err(self.sema_err("too many array initializers", self.peek().range));
                }
                for item in items {
                    self.check_initializer(element, &mut item.value)?;
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
                                    self.peek().range,
                                )
                            })?;
                    }
                    let field = fields.get(next).ok_or_else(|| {
                        self.sema_err("too many struct initializers", self.peek().range)
                    })?;
                    self.check_initializer(&field.ty, &mut item.value)?;
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
                    return Err(self.sema_err("too many union initializers", self.peek().range));
                }
                if let Some(item) = items.first_mut() {
                    let field = if let Some(Designator::Field(name)) = item.designators.first() {
                        fields
                            .iter()
                            .find(|field| field.name.as_deref() == Some(name))
                    } else {
                        fields.first()
                    }
                    .ok_or_else(|| {
                        self.sema_err("union has no matching field", self.peek().range)
                    })?;
                    self.check_initializer(&field.ty, &mut item.value)?;
                }
                Ok(())
            }
            (Initializer::List(items), _) if items.len() == 1 => {
                self.check_initializer(target, &mut items[0].value)
            }
            (Initializer::List(_), _) => {
                Err(self.sema_err("excess elements in scalar initializer", self.peek().range))
            }
        }
    }

    pub(crate) fn declaration_specifiers(&mut self) -> PResult<DeclSpec> {
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
                        alignment = Some(self.sema.alignof(&t))
                    } else {
                        let e = self.expression()?;
                        alignment = Some(self.sema.const_int(&e).ok_or_else(|| {
                            self.sema_err("alignment must be an integer constant", e.range)
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
                TokenKind::Identifier(ref n) if self.sema.lookup_typedef(n).is_some() => {
                    consumed = true;
                    let n = n.clone();
                    self.bump();
                    base = self.sema.lookup_typedef(&n);
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
    pub(crate) fn is_type_start(&self) -> bool {
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
            TokenKind::Identifier(n) => self.sema.lookup_typedef(n).is_some(),
            _ => false,
        }
    }

    pub(crate) fn record_spec(&mut self, union: bool) -> PResult<CType> {
        self.bump();
        let name = if let TokenKind::Identifier(n) = self.peek().kind.clone() {
            self.bump();
            Some(n)
        } else {
            None
        };
        let key = name.clone().map(|n| (n, if union { 1 } else { 0 }));
        let existing = key.as_ref().and_then(|key| self.sema.tag(key));
        if self.eat(&TokenKind::LBrace).is_none() {
            if let Some(existing) = existing {
                return Ok(existing);
            }
            let id = self.sema.fresh_tag_id();
            let ty = CType::new(if union {
                TypeKind::Union {
                    id,
                    name,
                    fields: None,
                }
            } else {
                TypeKind::Struct {
                    id,
                    name,
                    fields: None,
                }
            });
            if let Some(key) = key {
                self.sema.define_tag(key, ty.clone());
            }
            return Ok(ty);
        }
        let id = existing.as_ref().map_or_else(
            || self.sema.fresh_tag_id(),
            |ty| match ty.kind {
                TypeKind::Struct { id, .. } | TypeKind::Union { id, .. } => id,
                _ => unreachable!(),
            },
        );
        let mut fields = vec![];
        while !self.at(&TokenKind::RBrace) {
            let s = self.peek().range;
            let spec = self.declaration_specifiers()?;
            if self.eat(&TokenKind::Semi).is_some() {
                continue;
            }
            loop {
                let node = self.declarator(true)?;
                let (n, ty, _) = self.apply_declarator(node, spec.ty.clone())?;
                self.sema.validate_type(&ty, s)?;
                let bit_width =
                    if self.eat(&TokenKind::Colon).is_some() {
                        let e = self.assignment_expression()?;
                        Some(self.sema.const_int(&e).ok_or_else(|| {
                            self.sema_err("bit-field width must be constant", e.range)
                        })? as u32)
                    } else {
                        None
                    };
                fields.push(Field {
                    name: n,
                    ty,
                    bit_width,
                    range: s.join(self.previous().range),
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
                id,
                name,
                fields: Some(fields),
            }
        } else {
            TypeKind::Struct {
                id,
                name,
                fields: Some(fields),
            }
        });
        if let Some(k) = key {
            self.sema.define_tag(k, ty.clone());
        }
        Ok(ty)
    }
    pub(crate) fn enum_spec(&mut self) -> PResult<CType> {
        self.bump();
        let name = if let TokenKind::Identifier(n) = self.peek().kind.clone() {
            self.bump();
            Some(n)
        } else {
            None
        };
        let key = name.clone().map(|n| (n, 2));
        let existing = key.as_ref().and_then(|key| self.sema.tag(key));
        if self.eat(&TokenKind::LBrace).is_none() {
            if let Some(existing) = existing {
                return Ok(existing);
            }
            let ty = CType::new(TypeKind::Enum {
                id: self.sema.fresh_tag_id(),
                name,
                variants: None,
            });
            if let Some(key) = key {
                self.sema.define_tag(key, ty.clone());
            }
            return Ok(ty);
        }
        let id = existing.as_ref().map_or_else(
            || self.sema.fresh_tag_id(),
            |ty| match ty.kind {
                TypeKind::Enum { id, .. } => id,
                _ => unreachable!(),
            },
        );
        let mut variants = vec![];
        let mut next = 0i64;
        while !self.at(&TokenKind::RBrace) {
            let t = self.bump();
            let TokenKind::Identifier(n) = t.kind else {
                return Err(self.err("expected enumerator name"));
            };
            if self.eat(&TokenKind::Assign).is_some() {
                let e = self.assignment_expression()?;
                next = self.sema.const_int(&e).ok_or_else(|| {
                    self.sema_err("enumerator must be an integer constant", e.range)
                })? as i64
            }
            variants.push(EnumVariant {
                name: n.clone(),
                value: next,
                range: t.range,
            });
            self.sema.declare_enumerator(n, next as i128);
            next += 1;
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;
        let ty = CType::new(TypeKind::Enum {
            id,
            name,
            variants: Some(variants),
        });
        if let Some(k) = key {
            self.sema.define_tag(k, ty.clone());
        }
        Ok(ty)
    }

    pub(crate) fn declarator(&mut self, abstract_ok: bool) -> PResult<DNode> {
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
                        return Err(self.sema_err("array bound must have integer type", e.range));
                    }
                    match self.sema.const_int(&e) {
                        Some(value) if value > 0 => ArraySize::Constant(value as usize),
                        Some(_) => {
                            return Err(
                                self.sema_err("array bound must be greater than zero", e.range)
                            );
                        }
                        None if !self.sema.is_file_scope() => ArraySize::Variable(Box::new(e)),
                        None => {
                            return Err(self.sema_err(
                                "variably modified type is not allowed at file scope",
                                e.range,
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
    pub(crate) fn parameter_list(&mut self) -> PResult<(Vec<Parameter>, bool, bool)> {
        let mut p = vec![];
        let mut var = false;
        if self.at(&TokenKind::RParen) {
            return Ok((p, var, false));
        }
        if matches!(self.peek().kind, TokenKind::Identifier(ref name) if self.sema.lookup_typedef(name).is_none())
        {
            loop {
                let token = self.bump();
                let TokenKind::Identifier(name) = token.kind else {
                    unreachable!()
                };
                p.push(Parameter {
                    name: Some(name),
                    ty: CType::int(),
                    range: token.range,
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
            let s = self.peek().range;
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
                range: s.join(self.previous().range),
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

    pub(crate) fn old_parameter_declarations(&mut self, params: &mut [Parameter]) -> PResult<()> {
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
    pub(crate) fn apply_declarator(
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
    pub(crate) fn type_name(&mut self) -> PResult<CType> {
        let s = self.declaration_specifiers()?;
        let node = self.declarator(true)?;
        let (_, t, _) = self.apply_declarator(node, s.ty)?;
        Ok(t)
    }

    pub(crate) fn initializer(&mut self) -> PResult<Initializer> {
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
    pub(crate) fn static_assert(&mut self) -> PResult<StaticAssertion> {
        let start = self.bump().range;
        self.expect(&TokenKind::LParen)?;
        let e = self.assignment_expression()?;
        self.expect(&TokenKind::Comma)?;
        let message = match self.bump().kind {
            TokenKind::Literal(Literal::String { value: message, .. }) => message,
            _ => return Err(self.err("static assertion requires a string literal")),
        };
        self.expect(&TokenKind::RParen)?;
        let semi = self.expect(&TokenKind::Semi)?;
        if self.sema.const_int(&e) == Some(0) {
            return Err(self.sema_err("static assertion failed", e.range));
        }
        Ok(StaticAssertion {
            condition: e,
            message,
            range: start.join(semi.range),
        })
    }

    pub(crate) fn local_declaration(&mut self) -> PResult<Vec<Declaration>> {
        let start = self.peek().range;
        let spec = self.declaration_specifiers()?;
        if self.eat(&TokenKind::Semi).is_some() {
            return Ok(vec![Declaration {
                name: None,
                ty: spec.ty,
                storage: spec.storage,
                function_specifiers: spec.function_specifiers,
                initializer: None,
                alignment: spec.alignment,
                range: start.join(self.previous().range),
            }]);
        }
        let mut out = vec![];
        loop {
            let n = self.declarator(false)?;
            let (name, ty, _) = self.apply_declarator(n, spec.ty.clone())?;
            let d = self.finish_declaration(start, spec.clone(), name, ty)?;
            self.sema.declare(&d)?;
            out.push(d);
            if self.eat(&TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(&TokenKind::Semi)?;
        Ok(out)
    }
}
