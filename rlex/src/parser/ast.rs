//
// AST结构，用于指导生成NFA，和构建CharClass
// crated: 2025-07-28
// author: anishan
//

#[derive(Debug)]
pub enum ASTNode {
    Literal(char),           // 普通字段
    CharClass(ASTClassNode), // [] [^]

    Star(Box<ASTNode>),     // X*
    Question(Box<ASTNode>), // X?
    Plus(Box<ASTNode>),
    Range(Box<ASTNode>, ASTRangeNode), // {} 子节点, 当前Token

    // 二元节点
    Concatenation(Vec<ASTNode>), // abc
    Alternation(Vec<ASTNode>),   // A | B
}

#[derive(Debug)]
pub struct ASTClassNode {
    pub chars: Vec<char>,
    pub ranges: Vec<(char, char)>,
    pub reversed: bool,
    pub dot: bool,
}

impl ASTClassNode {
    pub fn dot() -> Self {
        Self {
            chars: vec![],
            ranges: vec![],
            reversed: false,
            dot: true,
        }
    }

    pub fn range(a: char, b: char, reversed: bool) -> ASTClassNode {
        ASTClassNode {
            chars: vec![a, b],
            ranges: vec![(a, b)],
            reversed,
            dot: false,
        }
    }

    pub fn charclass(chars: Vec<char>, ranges: Vec<(char, char)>, reversed: bool) -> ASTClassNode {
        ASTClassNode {
            chars,
            ranges,
            reversed,
            dot: false,
        }
    }

    pub fn words(chars: Vec<char>, reversed: bool) -> ASTClassNode {
        ASTClassNode {
            chars,
            ranges: vec![],
            reversed,
            dot: false,
        }
    }
}

#[derive(Debug)]
pub enum ASTRangeNode {
    Exact(usize),
    Between(usize, usize),
    Min(usize),
}
