use crate::common::grammar::{SymbolBound, Rule, Symbol, SymbolVec};





#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LR0Item<T: SymbolBound> {
    pub rule: SymbolVec<T>,
    pub pos: usize,
}

impl<T: SymbolBound> LR0Item<T> {

    /// 通过rule构建默认的LR0项目
    pub fn from_rule(rule: &Rule<T>) -> Self {
        match rule {
            Rule::Epsilon => Self {
                rule: Vec::new(),
                pos: 0,
            },
            Rule::Expression(expr) => Self {
                rule: expr.clone(),
                pos: 0,
            }
        }
    }

    /// 是否是归并项目
    pub fn is_reduced(&self) -> bool {
        self.pos == self.rule.len()
    }

    /// 向后移动
    pub fn move_next(&self) -> Self {
        assert!(!self.is_reduced()); // 规约项目不能在移动
        Self {
            rule: self.rule.clone(),
            pos: self.pos + 1,
        }
    }

    pub fn next_symbol(&self) -> Option<Symbol<T>> {
        if self.is_reduced() {
            None
        } else {
            Some(self.rule[self.pos].clone())
        }
    }


}

pub struct LR1Item<T: SymbolBound> {
    pub rule: SymbolVec<T>,
    pub lookahead: Symbol<T>,
    pub pos: usize,
}

