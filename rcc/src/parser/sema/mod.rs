use crate::TargetInfo;
use crate::err::{Diagnostic, ErrorKind};
use crate::parser::ast::{
    CType, DeclContextId, DeclId, Declaration, Linkage, Parameter, StorageClass, StorageDuration,
    TagId, TypeKind, ValueCategory,
};
use crate::source::SourceRange;
use std::collections::HashMap;

mod constant_eval;
mod layout;
mod type_system;

/// Semantic state shared by declaration, expression and statement parsing.
pub(crate) struct Sema {
    // Ordinary identifiers, typedef names, enum constants, and tags have C-specific namespace
    // interactions. Parallel scope stacks keep lookup rules explicit instead of hiding them in a
    // generic symbol table.
    scopes: Vec<HashMap<String, Binding>>,
    typedefs: Vec<HashMap<String, Binding>>,
    constants: Vec<HashMap<String, i128>>,
    tags: Vec<HashMap<String, CType>>,
    current_return: Option<CType>,
    loop_depth: usize,
    switches: Vec<SwitchState>,
    target: TargetInfo,
    next_tag_id: u32,
    next_decl_id: u32,
    contexts: Vec<DeclContextId>,
    next_context_id: u32,
    labels: HashMap<String, SourceRange>,
    unresolved_gotos: Vec<(String, SourceRange)>,
}

#[derive(Default)]
/// Per-switch state used to enforce unique converted case values and one default.
struct SwitchState {
    cases: HashMap<i128, SourceRange>,
    default: Option<SourceRange>,
}

#[derive(Clone)]
/// Ordinary-namespace lookup result stored by Sema scopes.
///
/// It connects a spelling to its declaration identity, declared type, value
/// category, and definition state without putting symbol tables in Parser.
pub(crate) struct Binding {
    pub(crate) ty: CType,
    pub(crate) declaration: DeclId,
    pub(crate) category: ValueCategory,
    has_definition: bool,
}

impl Sema {
    pub(crate) fn new(target: TargetInfo) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            typedefs: vec![HashMap::new()],
            constants: vec![HashMap::new()],
            tags: vec![HashMap::new()],
            current_return: None,
            loop_depth: 0,
            switches: Vec::new(),
            target,
            next_tag_id: 0,
            next_decl_id: 0,
            contexts: vec![DeclContextId(0)],
            next_context_id: 1,
            labels: HashMap::new(),
            unresolved_gotos: Vec::new(),
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.typedefs.push(HashMap::new());
        self.constants.push(HashMap::new());
        self.tags.push(HashMap::new());
    }

    pub(crate) fn leave_scope(&mut self) {
        self.scopes.pop();
        self.typedefs.pop();
        self.constants.pop();
        self.tags.pop();
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    pub(crate) fn lookup_typedef(&self, name: &str) -> Option<CType> {
        // An ordinary declaration in an inner scope hides an outer typedef of the same spelling.
        for index in (0..self.typedefs.len()).rev() {
            if self.scopes[index].contains_key(name) {
                return None;
            }
            if let Some(ty) = self.typedefs[index].get(name) {
                return Some(ty.ty.clone());
            }
        }
        None
    }

    pub(crate) fn declare(&mut self, declaration: &mut Declaration) -> Result<(), Diagnostic> {
        self.declare_impl(declaration, false)
    }

    pub(crate) fn declare_function_definition(
        &mut self,
        declaration: &mut Declaration,
    ) -> Result<(), Diagnostic> {
        self.declare_impl(declaration, true)
    }

    fn declare_impl(
        &mut self,
        declaration: &mut Declaration,
        function_definition: bool,
    ) -> Result<(), Diagnostic> {
        // Compatible declarations form a chain, but definitions and declarations without linkage
        // have stricter duplication rules. Binding the newest declaration mirrors C visibility.
        let Some(name) = declaration.name.clone() else {
            return Ok(());
        };
        let is_typedef = declaration.storage == StorageClass::Typedef;
        if is_typedef {
            if self.scopes.last().unwrap().contains_key(&name) {
                return Err(Diagnostic::new(
                    ErrorKind::Semantic,
                    format!("redefinition of '{name}' in the ordinary identifier namespace"),
                    declaration.range,
                ));
            }
        } else if self.typedefs.last().unwrap().contains_key(&name) {
            return Err(Diagnostic::new(
                ErrorKind::Semantic,
                format!("redefinition of typedef name '{name}'"),
                declaration.range,
            ));
        }
        let old = if is_typedef {
            self.typedefs.last().unwrap().get(&name).cloned()
        } else {
            self.scopes.last().unwrap().get(&name).cloned()
        };
        if let Some(old) = &old {
            if (!is_typedef && declaration.linkage == Linkage::None)
                || old.has_definition && (function_definition || declaration.initializer.is_some())
            {
                return Err(Diagnostic::new(
                    ErrorKind::Semantic,
                    format!("redefinition of '{name}'"),
                    declaration.range,
                ));
            }
            if !self.compatible(&old.ty, &declaration.ty) {
                return Err(Diagnostic::new(
                    ErrorKind::Semantic,
                    format!("incompatible redeclaration of '{name}'"),
                    declaration.range,
                ));
            }
            declaration.previous_declaration = Some(old.declaration);
        }
        let table = if is_typedef {
            self.typedefs.last_mut().unwrap()
        } else {
            self.scopes.last_mut().unwrap()
        };
        table.insert(
            name,
            Binding {
                ty: declaration.ty.clone(),
                declaration: declaration.id,
                category: if matches!(declaration.ty.kind, TypeKind::Function { .. }) {
                    ValueCategory::Function
                } else {
                    ValueCategory::LValue
                },
                has_definition: old.is_some_and(|binding| binding.has_definition)
                    || function_definition
                    || declaration.initializer.is_some(),
            },
        );
        Ok(())
    }

    pub(crate) fn begin_function(
        &mut self,
        parameters: &mut [Parameter],
        return_type: CType,
    ) -> DeclContextId {
        // Labels have function scope, while parameters and local objects use ordinary lexical
        // scopes. Both states are initialized here so Parser never owns semantic symbol tables.
        let context = self.fresh_context();
        self.labels.clear();
        self.unresolved_gotos.clear();
        self.contexts.push(context);
        self.enter_scope();
        for parameter in parameters {
            parameter.context = context;
            if let Some(name) = &parameter.name {
                self.scopes.last_mut().unwrap().insert(
                    name.clone(),
                    Binding {
                        ty: parameter.ty.clone(),
                        declaration: parameter.id,
                        category: ValueCategory::LValue,
                        has_definition: true,
                    },
                );
            }
        }
        self.current_return = Some(return_type);
        context
    }

    pub(crate) fn end_function(&mut self) -> Result<(), Diagnostic> {
        let unresolved = self
            .unresolved_gotos
            .iter()
            .find(|(name, _)| !self.labels.contains_key(name))
            .cloned();
        self.current_return = None;
        // Truncate rather than pop once: a syntax error may have escaped several nested compound
        // scopes. Restoring the translation-unit baseline is required before Parser recovery.
        self.scopes.truncate(1);
        self.typedefs.truncate(1);
        self.constants.truncate(1);
        self.tags.truncate(1);
        self.contexts.truncate(1);
        self.loop_depth = 0;
        self.switches.clear();
        if let Some((name, range)) = unresolved {
            Err(Diagnostic::new(
                ErrorKind::Semantic,
                format!("use of undeclared label '{name}'"),
                range,
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn current_return(&self) -> Option<CType> {
        self.current_return.clone()
    }

    pub(crate) fn tag(&self, name: &str) -> Option<CType> {
        self.tags
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    pub(crate) fn current_tag(&self, name: &str) -> Option<CType> {
        self.tags.last().unwrap().get(name).cloned()
    }

    pub(crate) fn define_tag(&mut self, name: String, ty: CType) {
        self.tags.last_mut().unwrap().insert(name, ty);
    }

    pub(crate) fn fresh_tag_id(&mut self) -> TagId {
        let id = TagId(self.next_tag_id);
        self.next_tag_id += 1;
        id
    }

    pub(crate) fn fresh_decl_id(&mut self) -> DeclId {
        let id = DeclId(self.next_decl_id);
        self.next_decl_id += 1;
        id
    }

    pub(crate) fn current_context(&self) -> DeclContextId {
        *self.contexts.last().unwrap()
    }

    pub(crate) fn fresh_context(&mut self) -> DeclContextId {
        let context = DeclContextId(self.next_context_id);
        self.next_context_id += 1;
        context
    }

    pub(crate) fn declaration_properties(
        &self,
        storage: StorageClass,
        ty: &CType,
    ) -> (DeclContextId, Linkage, StorageDuration) {
        let function = matches!(ty.kind, TypeKind::Function { .. });
        let linkage = if storage == StorageClass::Typedef {
            Linkage::None
        } else if self.is_file_scope() {
            if storage == StorageClass::Static {
                Linkage::Internal
            } else {
                Linkage::External
            }
        } else if storage == StorageClass::Extern || function {
            Linkage::External
        } else {
            Linkage::None
        };
        let storage_duration = if function || storage == StorageClass::Typedef {
            StorageDuration::None
        } else if matches!(
            storage,
            StorageClass::ThreadLocal
                | StorageClass::StaticThreadLocal
                | StorageClass::ExternThreadLocal
        ) {
            StorageDuration::Thread
        } else if self.is_file_scope()
            || matches!(storage, StorageClass::Static | StorageClass::Extern)
        {
            StorageDuration::Static
        } else {
            StorageDuration::Automatic
        };
        (self.current_context(), linkage, storage_duration)
    }

    pub(crate) fn declare_enumerator(&mut self, name: String, value: i128) -> DeclId {
        let declaration = self.fresh_decl_id();
        self.scopes.last_mut().unwrap().insert(
            name.clone(),
            Binding {
                ty: CType::int(),
                declaration,
                category: ValueCategory::RValue,
                has_definition: true,
            },
        );
        self.constants.last_mut().unwrap().insert(name, value);
        declaration
    }

    pub(crate) fn is_file_scope(&self) -> bool {
        self.scopes.len() == 1
    }

    pub(crate) fn begin_loop(&mut self) {
        self.loop_depth += 1;
    }
    pub(crate) fn end_loop(&mut self) {
        self.loop_depth -= 1;
    }
    pub(crate) fn in_loop(&self) -> bool {
        self.loop_depth != 0
    }
    pub(crate) fn begin_switch(&mut self) {
        self.switches.push(SwitchState::default());
    }
    pub(crate) fn end_switch(&mut self) {
        self.switches.pop();
    }
    pub(crate) fn in_switch(&self) -> bool {
        !self.switches.is_empty()
    }

    pub(crate) fn declare_case(
        &mut self,
        value: i128,
        range: SourceRange,
    ) -> Result<(), Diagnostic> {
        let switch = self
            .switches
            .last_mut()
            .ok_or_else(|| Diagnostic::new(ErrorKind::Semantic, "case outside switch", range))?;
        if switch.cases.insert(value, range).is_some() {
            Err(Diagnostic::new(
                ErrorKind::Semantic,
                format!("duplicate case value {value}"),
                range,
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn declare_default(&mut self, range: SourceRange) -> Result<(), Diagnostic> {
        let switch = self
            .switches
            .last_mut()
            .ok_or_else(|| Diagnostic::new(ErrorKind::Semantic, "default outside switch", range))?;
        if switch.default.replace(range).is_some() {
            Err(Diagnostic::new(
                ErrorKind::Semantic,
                "multiple default labels in one switch",
                range,
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn reference_label(&mut self, name: String, range: SourceRange) {
        self.unresolved_gotos.push((name, range));
    }

    pub(crate) fn declare_label(
        &mut self,
        name: String,
        range: SourceRange,
    ) -> Result<(), Diagnostic> {
        if self.labels.insert(name.clone(), range).is_some() {
            Err(Diagnostic::new(
                ErrorKind::Semantic,
                format!("duplicate label '{name}'"),
                range,
            ))
        } else {
            Ok(())
        }
    }
}
