use crate::char_class::char_class_set::CharClassSet;
use crate::parser::ast::{ASTClassNode, ASTNode, ASTRangeNode};
use common::lex::state_id_factory::IncrementalStateIDFactory;
use common::lex::{ClassID, NFASymbol, StateID, StateMeta, NFA};
use common::utils::unique_id_factory::UniqueIDFactory;
use std::collections::BTreeSet;

struct PartialNFA(NFA, usize);

/// NFA构造器
/// # Members 
/// - `char_class_set`: NFA构造是过程，并不需要保持，因此使用借用而不拿到所有权
/// - `id_factory`: ID生成器
pub struct NFABuilder<'a> {
    char_class_set: &'a CharClassSet,
    id_factory: IncrementalStateIDFactory, // ID自增
}

impl<'a> NFABuilder<'a> {
    pub fn new(char_class_set: &'a CharClassSet) -> Self {
        Self {
            char_class_set,
            id_factory: IncrementalStateIDFactory::new(0),
        }
    }

    /// 通过传入的AST构建NFA 注意CharClassSet的对应情况
    pub fn build(&mut self, ast: &ASTNode) -> NFA {
        let PartialNFA(mut nfa, out_state) = self.build_recursive(ast);
        nfa.set_terminate(out_state); // 对外接口都是只有一个NFA
        nfa
    }

    /// 获取class_id，简化这一长串 self.char_class_set.find_char(chr)
    fn class_id(&self, chr: char) -> ClassID {
        self.char_class_set.find_char(chr)
    }

    /// 同上 [begin, end]
    fn interval(&self, beg: char, end: char) -> BTreeSet<usize> {
        self.char_class_set.find_interval(beg, end)
    }

    /// 同上 U - [begin, end]
    fn reverse_interval(&self, beg: char, end: char) -> BTreeSet<usize> {
        self.char_class_set.find_reverse_interval(beg, end)
    }

    /// 获取ID并自增
    fn id(&mut self) -> StateID {
        self.id_factory.next_id()
    }

    fn peek_id(&self) -> StateID {
        self.id_factory.peek()
    }

    fn skip_id(&mut self, count: usize) {
        self.id_factory.skip_id(count)
    }

    /// 默认初始化的NFA，包含一个出点一个入点，没有边
    fn default_nfa(&mut self) -> (NFA, StateID, StateID) {
        let init_state = self.id();
        let out_state = self.id();
        let mut nfa = NFA::new(init_state);
        nfa.add_state(init_state, StateMeta::default())
            .add_state(out_state, StateMeta::default());
        (nfa, init_state, out_state)
    }

    /// 递归扫描AST构建NFA
    fn build_recursive(&mut self, ast: &ASTNode) -> PartialNFA {
        match ast {
            ASTNode::Literal(chr) => self.build_literal_nfa(*chr),
            ASTNode::CharClass(node) => self.build_char_class_nfa(node),
            ASTNode::Star(node) => self.build_star_nfa(node),
            ASTNode::Question(node) => self.build_question_nfa(node),
            ASTNode::Plus(node) => self.build_plus_dfa(node),
            ASTNode::Range(node, range) => self.build_range_nfa(node, range),
            ASTNode::Concatenation(nodes) => self.build_concat_nfa(nodes),
            ASTNode::Alternation(nodes) => self.build_alternation_nfa(nodes),
        }
    }

    /// 构建纯字符的nfa
    /// x - A -> y
    fn build_literal_nfa(&mut self, chr: char) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        nfa.add_edge(
            init_state,
            NFASymbol::ClassID(self.class_id(chr)),
            out_state,
        );
        PartialNFA(nfa, out_state)
    }

    /// 构建 CharClass nfa
    /// x - [a, b] -> y
    fn build_char_class_nfa(&mut self, node: &ASTClassNode) -> PartialNFA {
        if node.dot {
            // 处理 .
            self.build_dot_nfa()
        } else if node.reversed {
            // 处理反转模式
            self.build_reserved_class_nfa(node)
        } else {
            // 处理普通模式
            self.build_class_nfa(node)
        }
    }

    /// build_char_class_nfa 子方法 专门构建 '.'
    fn build_dot_nfa(&mut self) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        // 此处默认.不包含'\n'
        self.reverse_interval('\n', '\n')
            .into_iter()
            .for_each(|class_id| {
                nfa.add_edge(init_state, NFASymbol::ClassID(class_id), out_state);
            });
        PartialNFA(nfa, out_state)
    }

    /// build_char_class_nfa 子方法 用于构建普通等价类
    fn build_class_nfa(&mut self, node: &ASTClassNode) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();

        // 处理区间
        for (l, r) in node.ranges.iter() {
            self.interval(*l, *r).into_iter().for_each(|class_id| {
                nfa.add_edge(init_state, NFASymbol::ClassID(class_id), out_state);
            });
        }

        // 处理字符
        node.chars.iter().for_each(|chr| {
            let class_id = self.class_id(*chr);
            nfa.add_edge(init_state, NFASymbol::ClassID(class_id), out_state);
        });

        PartialNFA(nfa, out_state)
    }

    /// build_char_class_nfa 子方法 用于构建普通翻转等价类 reserved = true
    fn build_reserved_class_nfa(&mut self, node: &ASTClassNode) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();

        // 处理反转区间
        let interval = node
            .ranges
            .iter()
            .map(|(l, r)| self.reverse_interval(*l, *r))
            .chain(
                node.chars
                    .iter()
                    .map(|chr| self.reverse_interval(*chr, *chr)),
            ) // 处理反转字符
            .reduce(|a, b| a.intersection(&b).cloned().collect::<BTreeSet<_>>());

        if let Some(interval) = interval {
            // 设置边
            interval.into_iter().for_each(|class_id| {
                nfa.add_edge(init_state, NFASymbol::ClassID(class_id), out_state);
            });
        }

        PartialNFA(nfa, out_state)
    }

    /// 构建 (XXX)*       <---
    ///                  |   |
    ///              A -> XXX -> C
    ///              |           |
    ///              ------------>
    fn build_star_nfa(&mut self, node: &ASTNode) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();

        let PartialNFA(inner_nfa, inner_out) = self.build_recursive(node);
        let inner_init = nfa.merge(inner_nfa);

        nfa.add_edge(init_state, NFASymbol::Epsilon, inner_init) // A -> XXX
            .add_edge(inner_out, NFASymbol::Epsilon, out_state) // XXX -> C
            .add_edge(init_state, NFASymbol::Epsilon, out_state) // A -> C
            .add_edge(inner_out, NFASymbol::Epsilon, inner_init); // XXX -> XXX 

        PartialNFA(nfa, out_state)
    }

    /// 构建 (XXX)?
    ///              A -> XXX -> C
    ///              |           |
    ///              ------------>
    fn build_question_nfa(&mut self, node: &ASTNode) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();

        let PartialNFA(inner_nfa, inner_out) = self.build_recursive(node);
        let inner_init = nfa.merge(inner_nfa);

        nfa.add_edge(init_state, NFASymbol::Epsilon, inner_init) // A -> XXX
            .add_edge(inner_out, NFASymbol::Epsilon, out_state) // XXX -> C
            .add_edge(init_state, NFASymbol::Epsilon, out_state); // A -> C

        PartialNFA(nfa, out_state)
    }

    /// 构建 (XXX)*       <---
    ///                  |   |
    ///              A -> XXX -> C
    ///
    fn build_plus_dfa(&mut self, node: &ASTNode) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();

        let PartialNFA(inner_nfa, inner_out) = self.build_recursive(node);
        let inner_init = nfa.merge(inner_nfa);

        nfa.add_edge(init_state, NFASymbol::Epsilon, inner_init) // A -> XXX
            .add_edge(inner_out, NFASymbol::Epsilon, out_state) // XXX -> C
            .add_edge(inner_out, NFASymbol::Epsilon, inner_init); // XXX -> XXX 

        PartialNFA(nfa, out_state)
    }

    /// 构建range语法 A{n, m} / {n,} / {n}
    fn build_range_nfa(&mut self, node: &ASTNode, range: &ASTRangeNode) -> PartialNFA {
        match range {
            ASTRangeNode::Exact(x) => self.build_exact_range_nfa(node, *x),
            ASTRangeNode::Between(b, e) => self.build_between_range_nfa(node, *b, *e),
            ASTRangeNode::Min(x) => self.build_min_range_nfa(node, *x),
        }
    }
    fn repeat_nfa_with<F>(
        &mut self,
        inner_nfa: &mut NFA,
        inner_init_: StateID,
        inner_out_: StateID,
        repeat: usize,
        mut on_connect: F,
        nfa: &mut NFA,
        start: StateID,
    ) -> StateID
    where
        F: FnMut(&mut NFA, StateID, StateID, StateID), // prev, inner_init, inner_out
    {
        let mut prev = start;
        for _ in 0..repeat {
            let inner_init = nfa.merge_offset(inner_nfa, self.peek_id());
            let offset = inner_init - inner_init_;
            let inner_out = inner_out_ + offset;
            self.skip_id(offset);

            on_connect(nfa, prev, inner_init, inner_out);
            prev = inner_out;
        }
        prev
    }
    fn build_exact_range_nfa(&mut self, node: &ASTNode, x: usize) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        let PartialNFA(mut inner_nfa, inner_out_) = self.build_recursive(node);
        let inner_init_ = inner_nfa.get_init_state();

        let prev = self.repeat_nfa_with(
            &mut inner_nfa,
            inner_init_,
            inner_out_,
            x,
            |nfa, prev, inner_init, _inner_out| {
                nfa.add_edge(prev, NFASymbol::Epsilon, inner_init);
            },
            &mut nfa,
            init_state,
        );

        nfa.add_edge(prev, NFASymbol::Epsilon, out_state);
        PartialNFA(nfa, out_state)
    }

    fn build_between_range_nfa(&mut self, node: &ASTNode, beg: usize, end: usize) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        let PartialNFA(mut inner_nfa, inner_out_) = self.build_recursive(node);
        let inner_init_ = inner_nfa.get_init_state();

        let mut prev = self.repeat_nfa_with(
            &mut inner_nfa,
            inner_init_,
            inner_out_,
            beg,
            |nfa, prev, inner_init, _inner_out| {
                nfa.add_edge(prev, NFASymbol::Epsilon, inner_init);
            },
            &mut nfa,
            init_state,
        );

        prev = self.repeat_nfa_with(
            &mut inner_nfa,
            inner_init_,
            inner_out_,
            end - beg,
            |nfa, prev, inner_init, _inner_out| {
                nfa.add_edge(prev, NFASymbol::Epsilon, inner_init).add_edge(
                    inner_init,
                    NFASymbol::Epsilon,
                    out_state,
                );
            },
            &mut nfa,
            prev,
        );

        nfa.add_edge(prev, NFASymbol::Epsilon, out_state);
        PartialNFA(nfa, out_state)
    }

    fn build_min_range_nfa(&mut self, node: &ASTNode, x: usize) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        let PartialNFA(mut inner_nfa, inner_out_) = self.build_recursive(node);
        let inner_init_ = inner_nfa.get_init_state();

        let prev = self.repeat_nfa_with(
            &mut inner_nfa,
            inner_init_,
            inner_out_,
            x,
            |nfa, prev, inner_init, _inner_out| {
                nfa.add_edge(prev, NFASymbol::Epsilon, inner_init);
            },
            &mut nfa,
            init_state,
        );

        let inner_init = nfa.merge_offset(&inner_nfa, self.peek_id());
        let offset = inner_init - inner_init_;
        let inner_out = inner_out_ + offset;
        self.skip_id(offset);

        nfa.add_edge(prev, NFASymbol::Epsilon, inner_init)
            .add_edge(inner_out, NFASymbol::Epsilon, inner_init) // backward ε
            .add_edge(inner_init, NFASymbol::Epsilon, inner_out) // forward ε
            .add_edge(inner_out, NFASymbol::Epsilon, out_state);

        PartialNFA(nfa, out_state)
    }

    /// 创建联结NFA  A -> XXX -> B -> ... -> C
    fn build_concat_nfa(&mut self, nodes: &[ASTNode]) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        let mut prev_state = init_state;
        nodes
            .iter()
            .map(|node| self.build_recursive(node))
            .for_each(|PartialNFA(inner_nfa, inner_out)| {
                let inner_init = nfa.merge(inner_nfa);
                nfa.add_edge(prev_state, NFASymbol::Epsilon, inner_init);
                prev_state = inner_out;
            });
        nfa.add_edge(prev_state, NFASymbol::Epsilon, out_state);
        PartialNFA(nfa, out_state)
    }

    /// 创建"候选"NFA A -> XXX | YYY | ZZZ -> C
    fn build_alternation_nfa(&mut self, nodes: &[ASTNode]) -> PartialNFA {
        let (mut nfa, init_state, out_state) = self.default_nfa();
        nodes
            .iter()
            .map(|node| self.build_recursive(node))
            .for_each(|PartialNFA(inner_nfa, inner_out)| {
                let inner_init = nfa.merge(inner_nfa);
                nfa.add_edge(init_state, NFASymbol::Epsilon, inner_init)
                    .add_edge(inner_out, NFASymbol::Epsilon, out_state);
            });

        PartialNFA(nfa, out_state)
    }
}
