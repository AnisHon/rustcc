//
// 由手写递归下降生成的树形结构，是token与ast之间的中间表达
// crated: 2025-07-27
// author: anishan
//

use std::rc::Rc;
use crate::lex::ReToken;



#[derive(Debug)]
pub enum CSTNode {
    // 原子具体节点
    Literal(Rc<ReToken>),           // 普通字段
    Dot(Rc<ReToken>),               // .
    MetaChar(Rc<ReToken>),          // \w \s \d
    CharClass(Rc<ReToken>),    // [] [^]
    Group(Box<CSTNode>),        // 捕获组

    // 量词具体节点
    Star(Box<CSTNode>),           // X*
    Question(Box<CSTNode>),       // X?
    Plus(Box<CSTNode>),
    Range(Box<CSTNode>, Rc<ReToken>), // {} 子节点, 当前Token

    // 二元节点
    Sequence(Vec<CSTNode>),              // abc
    Alternation(Vec<CSTNode>),           // A | B

    // 抽象节点
    Atomic(Box<CSTNode>),         // 原子节点 Dot Class Literal MetaChar
    Quantified(Box<CSTNode>),     // 量词节点 * ? + {}
    Expr(Box<CSTNode>),           // 最顶层节点
}
