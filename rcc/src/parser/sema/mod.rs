use crate::TargetInfo;
use crate::err::{Diagnostic, ErrorKind};
use crate::parser::ast::{CType, Declaration, Parameter, StorageClass, TagId};
use std::collections::HashMap;

mod constant_eval;
mod layout;
mod type_system;

/// Semantic state shared by declaration, expression and statement parsing.
pub(crate) struct Sema {
    scopes: Vec<HashMap<String, CType>>,
    typedefs: Vec<HashMap<String, CType>>,
    constants: Vec<HashMap<String, i128>>,
    tags: HashMap<(String, u8), CType>,
    current_return: Option<CType>,
    loop_depth: usize,
    switch_depth: usize,
    target: TargetInfo,
    next_tag_id: u32,
}

impl Sema {
    pub(crate) fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            typedefs: vec![HashMap::new()],
            constants: vec![HashMap::new()],
            tags: HashMap::new(),
            current_return: None,
            loop_depth: 0,
            switch_depth: 0,
            target: TargetInfo::default(),
            next_tag_id: 0,
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.typedefs.push(HashMap::new());
        self.constants.push(HashMap::new());
    }

    pub(crate) fn leave_scope(&mut self) {
        self.scopes.pop();
        self.typedefs.pop();
        self.constants.pop();
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<CType> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    pub(crate) fn lookup_typedef(&self, name: &str) -> Option<CType> {
        for index in (0..self.typedefs.len()).rev() {
            if self.scopes[index].contains_key(name) {
                return None;
            }
            if let Some(ty) = self.typedefs[index].get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub(crate) fn declare(&mut self, declaration: &Declaration) -> Result<(), Diagnostic> {
        let Some(name) = &declaration.name else {
            return Ok(());
        };
        let table = if declaration.storage == StorageClass::Typedef {
            if self.scopes.last().unwrap().contains_key(name) {
                return Err(Diagnostic::new(
                    ErrorKind::Semantic,
                    format!("redefinition of '{name}' in the ordinary identifier namespace"),
                    declaration.range,
                ));
            }
            self.typedefs.last_mut().unwrap()
        } else {
            if self.typedefs.last().unwrap().contains_key(name) {
                return Err(Diagnostic::new(
                    ErrorKind::Semantic,
                    format!("redefinition of typedef name '{name}'"),
                    declaration.range,
                ));
            }
            self.scopes.last_mut().unwrap()
        };
        if let Some(old) = table.get(name) {
            if old != &declaration.ty {
                return Err(Diagnostic::new(
                    ErrorKind::Semantic,
                    format!("incompatible redeclaration of '{name}'"),
                    declaration.range,
                ));
            }
        } else {
            table.insert(name.clone(), declaration.ty.clone());
        }
        Ok(())
    }

    pub(crate) fn begin_function(&mut self, parameters: &[Parameter], return_type: CType) {
        self.enter_scope();
        for parameter in parameters {
            if let Some(name) = &parameter.name {
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), parameter.ty.clone());
            }
        }
        self.current_return = Some(return_type);
    }

    pub(crate) fn end_function(&mut self) {
        self.current_return = None;
        self.leave_scope();
    }

    pub(crate) fn current_return(&self) -> Option<CType> {
        self.current_return.clone()
    }

    pub(crate) fn tag(&self, key: &(String, u8)) -> Option<CType> {
        self.tags.get(key).cloned()
    }

    pub(crate) fn define_tag(&mut self, key: (String, u8), ty: CType) {
        self.tags.insert(key, ty);
    }

    pub(crate) fn fresh_tag_id(&mut self) -> TagId {
        let id = TagId(self.next_tag_id);
        self.next_tag_id += 1;
        id
    }

    pub(crate) fn declare_enumerator(&mut self, name: String, value: i128) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.clone(), CType::int());
        self.constants.last_mut().unwrap().insert(name, value);
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
        self.switch_depth += 1;
    }
    pub(crate) fn end_switch(&mut self) {
        self.switch_depth -= 1;
    }
    pub(crate) fn in_switch(&self) -> bool {
        self.switch_depth != 0
    }
}
