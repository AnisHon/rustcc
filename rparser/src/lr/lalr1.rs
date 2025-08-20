use common::utils::id_util::IncIDFactory;
use crate::common::grammar::{Grammar, SymbolBound};
use crate::lr::lr0::LR0Builder;
use crate::lr::lr1::LR1Builder;

pub struct LALR1Builder<'a, T: SymbolBound> {
    grammar: &'a Grammar<T>,
    id_factory: IncIDFactory,
}

impl<'a, T: SymbolBound> LALR1Builder<'a, T> {
    pub fn new(grammar: &'a Grammar<T>) -> Self {
        Self {
            grammar,
            id_factory: IncIDFactory::new(0)
        }
    }
    
    /// 使用LR0传播算法构建LALR
    pub fn build_table(self) {
        let (id2item_map, transition, init_state) = LR0Builder::new(self.grammar).build_table();
    }
    
    /// 使用LR1合并算法构建LALR
    pub fn build_from_lr1(self) {
        let (id2item_map, transition, init_state) = LR1Builder::new(self.grammar).build_table();
        
    }
    
    
    
}

