use crate::parser::ast::*;
use crate::type_system::{
    ArrayBound, BuiltinType, FunctionType, QualType, Qualifiers as CanonicalQualifiers, RecordKind,
    TypeContext,
};
use std::collections::HashMap;

pub(super) struct TypeImporter<'a> {
    types: &'a mut TypeContext,
    records: HashMap<TagId, QualType>,
    enums: HashMap<TagId, QualType>,
}

impl<'a> TypeImporter<'a> {
    pub(super) fn new(types: &'a mut TypeContext) -> Self {
        Self {
            types,
            records: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub(super) fn import_translation_unit(&mut self, unit: &mut TranslationUnit) {
        for declaration in &mut unit.declarations {
            match declaration {
                ExternalDeclaration::Declaration(declaration) => self.declaration(declaration),
                ExternalDeclaration::Function(function) => {
                    self.ty(&mut function.ty);
                    for parameter in &mut function.parameters {
                        self.ty(&mut parameter.ty);
                    }
                    self.statement(&mut function.body);
                }
                ExternalDeclaration::StaticAssert(assertion) => {
                    self.expression(&mut assertion.condition)
                }
            }
        }
    }

    fn declaration(&mut self, declaration: &mut Declaration) {
        self.ty(&mut declaration.ty);
        if let Some(initializer) = &mut declaration.initializer {
            self.initializer(initializer);
        }
    }

    fn initializer(&mut self, initializer: &mut Initializer) {
        match initializer {
            Initializer::Expression(expression) => self.expression(expression),
            Initializer::List(items) => {
                for item in items {
                    for designator in &mut item.designators {
                        if let Designator::Index(expression) = designator {
                            self.expression(expression);
                        }
                    }
                    self.initializer(&mut item.value);
                }
            }
        }
    }

    fn statement(&mut self, statement: &mut Statement) {
        match &mut statement.kind {
            StatementKind::Empty
            | StatementKind::Goto(_)
            | StatementKind::Continue
            | StatementKind::Break => {}
            StatementKind::Expression(expression) => self.expression(expression),
            StatementKind::Compound(items) => {
                for item in items {
                    match item {
                        BlockItem::Declaration(declaration) => self.declaration(declaration),
                        BlockItem::Statement(statement) => self.statement(statement),
                        BlockItem::StaticAssert(assertion) => {
                            self.expression(&mut assertion.condition)
                        }
                    }
                }
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition);
                self.statement(then_branch);
                if let Some(branch) = else_branch {
                    self.statement(branch);
                }
            }
            StatementKind::Switch { expression, body } => {
                self.expression(expression);
                self.statement(body);
            }
            StatementKind::While { condition, body } => {
                self.expression(condition);
                self.statement(body);
            }
            StatementKind::DoWhile { body, condition } => {
                self.statement(body);
                self.expression(condition);
            }
            StatementKind::For {
                init,
                condition,
                step,
                body,
            } => {
                match init {
                    ForInit::Expression(expression) => {
                        if let Some(expression) = expression {
                            self.expression(expression);
                        }
                    }
                    ForInit::Declaration(declarations) => {
                        for declaration in declarations {
                            self.declaration(declaration);
                        }
                    }
                }
                if let Some(condition) = condition {
                    self.expression(condition);
                }
                if let Some(step) = step {
                    self.expression(step);
                }
                self.statement(body);
            }
            StatementKind::Label { statement, .. } | StatementKind::Default { statement } => {
                self.statement(statement)
            }
            StatementKind::Case { value, statement } => {
                self.expression(value);
                self.statement(statement);
            }
            StatementKind::Return(expression) => {
                if let Some(expression) = expression {
                    self.expression(expression);
                }
            }
        }
    }

    fn expression(&mut self, expression: &mut Expression) {
        self.ty(&mut expression.ty);
        match &mut expression.kind {
            ExpressionKind::Integer(_)
            | ExpressionKind::Floating(_)
            | ExpressionKind::Character { .. }
            | ExpressionKind::String { .. }
            | ExpressionKind::Identifier(_)
            | ExpressionKind::SizeofType(_)
            | ExpressionKind::Alignof(_) => {}
            ExpressionKind::Unary { operand, .. }
            | ExpressionKind::PostIncrement { operand, .. } => self.expression(operand),
            ExpressionKind::Binary { left, right, .. }
            | ExpressionKind::Assignment { left, right, .. } => {
                self.expression(left);
                self.expression(right);
            }
            ExpressionKind::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.expression(condition);
                self.expression(then_expr);
                self.expression(else_expr);
            }
            ExpressionKind::Call { callee, arguments } => {
                self.expression(callee);
                for argument in arguments {
                    self.expression(argument);
                }
            }
            ExpressionKind::Subscript { base, index } => {
                self.expression(base);
                self.expression(index);
            }
            ExpressionKind::Member { base, .. } => self.expression(base),
            ExpressionKind::Cast { target, expression } => {
                self.ty(target);
                self.expression(expression);
            }
            ExpressionKind::ImplicitCast { expression, .. }
            | ExpressionKind::SizeofExpression(expression) => self.expression(expression),
            ExpressionKind::CompoundLiteral { ty, initializer } => {
                self.ty(ty);
                self.initializer(initializer);
            }
            ExpressionKind::GenericSelection {
                controlling,
                selected,
            } => {
                self.expression(controlling);
                self.expression(selected);
            }
            ExpressionKind::Comma(expressions) => {
                for expression in expressions {
                    self.expression(expression);
                }
            }
        }
        match &mut expression.kind {
            ExpressionKind::SizeofType(ty) | ExpressionKind::Alignof(ty) => {
                self.ty(ty);
            }
            _ => {}
        }
    }

    fn ty(&mut self, ty: &mut CType) -> QualType {
        let unqualified = match &mut ty.kind {
            TypeKind::Void => self.types.builtin(BuiltinType::Void),
            TypeKind::Bool => self.types.builtin(BuiltinType::Bool),
            TypeKind::Char { signed: None } => self.types.builtin(BuiltinType::Char),
            TypeKind::Char { signed: Some(true) } => self.types.builtin(BuiltinType::SignedChar),
            TypeKind::Char {
                signed: Some(false),
            } => self.types.builtin(BuiltinType::UnsignedChar),
            TypeKind::Short { signed: true } => self.types.builtin(BuiltinType::Short),
            TypeKind::Short { signed: false } => self.types.builtin(BuiltinType::UnsignedShort),
            TypeKind::Int { signed: true } => self.types.builtin(BuiltinType::Int),
            TypeKind::Int { signed: false } => self.types.builtin(BuiltinType::UnsignedInt),
            TypeKind::Long { signed: true } => self.types.builtin(BuiltinType::Long),
            TypeKind::Long { signed: false } => self.types.builtin(BuiltinType::UnsignedLong),
            TypeKind::LongLong { signed: true } => self.types.builtin(BuiltinType::LongLong),
            TypeKind::LongLong { signed: false } => {
                self.types.builtin(BuiltinType::UnsignedLongLong)
            }
            TypeKind::Float => self.types.builtin(BuiltinType::Float),
            TypeKind::Double => self.types.builtin(BuiltinType::Double),
            TypeKind::LongDouble => self.types.builtin(BuiltinType::LongDouble),
            TypeKind::Complex(inner) => match inner.kind {
                TypeKind::Float => self.types.builtin(BuiltinType::FloatComplex),
                TypeKind::LongDouble => self.types.builtin(BuiltinType::LongDoubleComplex),
                _ => self.types.builtin(BuiltinType::DoubleComplex),
            },
            TypeKind::Imaginary(inner) => match inner.kind {
                TypeKind::Float => self.types.builtin(BuiltinType::FloatImaginary),
                TypeKind::LongDouble => self.types.builtin(BuiltinType::LongDoubleImaginary),
                _ => self.types.builtin(BuiltinType::DoubleImaginary),
            },
            TypeKind::Pointer(pointee) => {
                let pointee = self.ty(pointee);
                self.types.pointer(pointee)
            }
            TypeKind::Array { element, size } => {
                let element = self.ty(element);
                let bound = match size {
                    ArraySize::Constant(size) => ArrayBound::Constant(*size as u64),
                    ArraySize::Variable(expression) => {
                        self.expression(expression);
                        ArrayBound::Variable
                    }
                    ArraySize::Unspecified => ArrayBound::Incomplete,
                    ArraySize::Star => ArrayBound::Star,
                };
                self.types
                    .array(element, bound)
                    .expect("parser admitted invalid array type")
            }
            TypeKind::Function {
                return_type,
                params,
                variadic,
                has_prototype,
            } => {
                let result = self.ty(return_type);
                let parameters = params
                    .iter_mut()
                    .map(|param| self.ty(&mut param.ty))
                    .collect();
                self.types
                    .function(FunctionType {
                        result,
                        parameters,
                        variadic: *variadic,
                        has_prototype: *has_prototype,
                        calling_convention: Default::default(),
                    })
                    .expect("parser admitted invalid function type")
            }
            TypeKind::Struct { id, fields, .. } => {
                if let Some(fields) = fields {
                    for field in fields {
                        self.ty(&mut field.ty);
                    }
                }
                *self
                    .records
                    .entry(*id)
                    .or_insert_with(|| self.types.fresh_record(RecordKind::Struct))
            }
            TypeKind::Union { id, fields, .. } => {
                if let Some(fields) = fields {
                    for field in fields {
                        self.ty(&mut field.ty);
                    }
                }
                *self
                    .records
                    .entry(*id)
                    .or_insert_with(|| self.types.fresh_record(RecordKind::Union))
            }
            TypeKind::Enum { id, .. } => *self
                .enums
                .entry(*id)
                .or_insert_with(|| self.types.fresh_enum()),
        };
        let mut qualifiers = CanonicalQualifiers::empty();
        if ty.qualifiers.is_const {
            qualifiers = qualifiers.union(CanonicalQualifiers::CONST);
        }
        if ty.qualifiers.is_volatile {
            qualifiers = qualifiers.union(CanonicalQualifiers::VOLATILE);
        }
        if ty.qualifiers.is_restrict {
            qualifiers = qualifiers.union(CanonicalQualifiers::RESTRICT);
        }
        if ty.qualifiers.is_atomic {
            qualifiers = qualifiers.union(CanonicalQualifiers::ATOMIC);
        }
        ty.canonical = unqualified.with_qualifiers(qualifiers);
        ty.canonical
    }
}
