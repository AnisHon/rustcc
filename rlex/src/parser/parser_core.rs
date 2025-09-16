use std::iter::Peekable;
use crate::common::re_err::{ReError, ReResult};
use crate::lex::{ReToken, ReTokenType};
use crate::parser::ast::{ASTClassNode, ASTNode, ASTRangeNode};
use crate::parser::cst::CSTNode;
use std::vec::IntoIter;

///
/// Re手写递归下降解析器
/// 
/// # Members
/// - `tokens`: Peekable迭代器，符合递归下降语义
/// - `cst`: 结果CST 
/// - `ast`: 结果AST 
/// - `last_pos`: 最后的位置，用于友好的错误提示 
/// 
pub struct ReParser {
    tokens: Peekable<IntoIter<ReToken>>,
    last_pos: usize,
}  

///
/// 正则手写递归下降，负责将Token转换为CST和AST
/// 通过peek consume之类cursor功能
///
/// # Grammars
/// ```text
/// Expression  -> Alternation
/// Alternation -> Sequence (| Sequence)*
/// Sequence    -> Quantified Quantified*
/// Quantified  -> Atomic (Star | Question | Plus | Range)
/// Atomic      -> (Expression) | Literal | Dot | MetaChar | CharClass
/// ```
/// - + * ()是运算符类似正则
/// - 没有左递归问题
/// - (Expression)就是捕获组Group 不太想继续建立新推导式
///
///
impl ReParser {
    pub fn new(tokens: Vec<ReToken>) -> ReParser {
        assert!(!tokens.is_empty());
        let last_pos = tokens.last().unwrap().pos; 
        let peekable = tokens.into_iter().peekable();
        
        ReParser {
            tokens: peekable,
            last_pos,
        }
    }
    
    pub fn parse(mut self) -> ReResult<CSTNode> {
        self.parse_expression()
    }
    
    
    /// Expr -> Alternation
    fn parse_expression(&mut self) -> ReResult<CSTNode> {
        Ok(CSTNode::Expr(Box::new(self.parse_alternation()?)))
    }

    /// Alternation -> Sequence (| Sequence)*
    fn parse_alternation(&mut self) -> ReResult<CSTNode> {
        let first = self.parse_sequence()?; // 一定存在的第一个Sequence
        let mut nodes = vec![first];
        let mut illegal_alter = false; // 用于标识 '|' 是否闭合

        while let Some(next_token) = self.tokens.peek() {

            match next_token.typ {
                ReTokenType::Pipe => {
                    illegal_alter = true;
                    self.tokens.next(); // 消耗 |
                    continue;
                }
                ReTokenType::RParen if !illegal_alter => return Ok(CSTNode::Alternation(nodes)), // follow遇到 )

                ReTokenType::Star
                | ReTokenType::Question
                | ReTokenType::Plus
                | ReTokenType::Range
                | ReTokenType::Literal
                | ReTokenType::LParen
                | ReTokenType::CharClass
                | ReTokenType::Dot => {
                    // first遇到这些，继续解析
                    illegal_alter = false;
                    nodes.push(self.parse_sequence()?);
                }
                _ => {
                    // 错误处理
                    return Err(ReError::new(
                        &format!("Illegal '{}' In Alter", next_token.value),
                        next_token.pos,
                    ));
                }
            }
        }

        if illegal_alter {
            // | 未闭合
            return Err(ReError::new("Expect Alternation", self.last_pos));
        }

        Ok(CSTNode::Alternation(nodes))
    }

    /// Sequence -> Quantified Quantified*
    fn parse_sequence(&mut self) -> ReResult<CSTNode> {
        let first = self.parse_quantified()?; // 一定存在的第一个Quantified
        let mut nodes = vec![first];
        while let Some(next_token) = self.tokens.peek() {

            match next_token.typ {
                ReTokenType::Pipe | ReTokenType::RParen => {
                    // follow遇到 ) 或 |
                    return Ok(CSTNode::Sequence(nodes));
                }
                ReTokenType::Star
                | ReTokenType::Question
                | ReTokenType::Plus
                | ReTokenType::Range
                | ReTokenType::Literal
                | ReTokenType::LParen
                | ReTokenType::CharClass
                | ReTokenType::DigitClass
                | ReTokenType::NonDigitClass
                | ReTokenType::WordClass
                | ReTokenType::NonWordClass
                | ReTokenType::NonSpaceClass
                | ReTokenType::Dot => {
                    // first遇到这些，继续解析
                    nodes.push(self.parse_quantified()?);
                }
                _ => {
                    return Err(ReError::new(
                        // 错误处理
                        &format!("Illegal '{}' In Concat", next_token.value),
                        next_token.pos,
                    ));
                }
            }
        }

        Ok(CSTNode::Sequence(nodes))
    }

    /// Quantified -> Atomic (Star | Question | Plus | Range)
    fn parse_quantified(&mut self) -> ReResult<CSTNode> {
        let left = self.parse_atomic()?; // 一定存在的第一个Atomic

        let next_token = match self.tokens.peek() {
            Some(x) => x,
            None => return Ok(CSTNode::Quantified(Box::new(left))),
        };

        let node = match next_token.typ {
            ReTokenType::Star => {
                self.tokens.next(); // 消耗 *
                CSTNode::Star(Box::new(left))
            }, // 如果当前是 运算符直接组合
            ReTokenType::Question => {
                self.tokens.next(); // 消耗 ?
                CSTNode::Question(Box::new(left))
            },
            ReTokenType::Plus => {
                self.tokens.next(); // 消耗 +
                CSTNode::Plus(Box::new(left))
            },
            ReTokenType::Range => {
                // 消耗 {..}
                let next_token = self.tokens.next().unwrap();
                CSTNode::Range(Box::new(left), next_token)
            },
            ReTokenType::LParen
            | ReTokenType::RParen
            | ReTokenType::Literal
            | ReTokenType::Pipe
            | ReTokenType::CharClass
            | ReTokenType::DigitClass
            | ReTokenType::NonDigitClass
            | ReTokenType::WordClass
            | ReTokenType::NonWordClass
            | ReTokenType::NonSpaceClass
            | ReTokenType::Dot => {
                // 如果当前是 | ( Literal [ 则结束
                left
            }
            _ => {
                // 错误处理
                return Err(ReError::new(
                    &format!(
                        "Illegal '{}'({:?}) In Quantifier",
                        next_token.value, next_token.typ
                    ),
                    next_token.pos,
                ));
            }
        };



        Ok(CSTNode::Quantified(Box::new(node)))
    }

    fn parse_atomic(&mut self) -> ReResult<CSTNode> {
        let token = self.tokens.next()
            .unwrap();

        let node = match token.typ {
            ReTokenType::LParen => self.parse_group()?, // 推导parse_group，已经消费掉第一个(，parse_group负责消费末尾的 )

            ReTokenType::Literal => CSTNode::Literal(token),
            ReTokenType::Dot => CSTNode::Dot(token),
            ReTokenType::CharClass => CSTNode::CharClass(token),
            ReTokenType::DigitClass
            | ReTokenType::NonDigitClass
            | ReTokenType::WordClass
            | ReTokenType::NonWordClass
            | ReTokenType::SpaceClass
            | ReTokenType::NonSpaceClass => CSTNode::MetaChar(token),

            _ => {
                return Err(ReError::new(
                    &format!("Illegal Character '{}'", token.value),
                    token.pos,
                ));
            }
        };

        Ok(CSTNode::Atomic(Box::new(node)))
    }

    ///
    /// Group -> \( Alternation \)
    /// 不负责消费第一个括号，负责消费末尾括号
    ///
    fn parse_group(&mut self) -> ReResult<CSTNode> {
        let node = self.parse_alternation()?;
        let right = self.tokens.next()
            .ok_or(ReError::new("Brace Not Close", self.last_pos))?;
        if !matches!(right.typ, ReTokenType::RParen) {
            return Err(ReError::new("Brace Not Close", right.pos));
        }

        Ok(CSTNode::Group(Box::new(node)))
    }
}

fn resolve_meta(token: &ReToken) -> ASTClassNode {
    match token.typ {
        ReTokenType::DigitClass => ASTClassNode::range('0', '9', false),
        ReTokenType::NonDigitClass => ASTClassNode::range('0', '9', true),
        ReTokenType::WordClass => {
            ASTClassNode::charclass(vec!['_'], vec![('0', '9'), ('A', 'Z'), ('a', 'z')], false)
        }
        ReTokenType::NonWordClass => {
            ASTClassNode::charclass(vec!['_'], vec![('0', '9'), ('A', 'Z'), ('a', 'z')], true)
        }
        ReTokenType::SpaceClass => {
            ASTClassNode::words(vec![' ', '\t', '\n', '\r', '\x0B', '\x0C'], false)
        } // \v \f
        ReTokenType::NonSpaceClass => {
            ASTClassNode::words(vec![' ', '\t', '\n', '\r', '\x0B', '\x0C'], true)
        } // \v \f
        _ => panic!("Impossible"),
    }
}

fn resolve_range(token: &ReToken) -> ReResult<ASTRangeNode> {
    let nums: Vec<String> = token
        .value
        .split(',')
        .map(|s| s.to_string()) // 保留所有部分，包括可能的空字符串
        .collect();

    let node = match nums.len() {
        1 => ASTRangeNode::Exact(
            nums[0]
                .parse()
                .map_err(|_| ReError::new("Illegal Number", token.pos))?,
        ),
        2 if nums[1].is_empty() => ASTRangeNode::Min(
            nums[0]
                .parse()
                .map_err(|_| ReError::new("Illegal Number", token.pos))?,
        ),
        2 if !nums[1].is_empty() => ASTRangeNode::Between(
            nums[0]
                .parse()
                .map_err(|_| ReError::new("Illegal Number", token.pos))?,
            nums[1]
                .parse()
                .map_err(|_| ReError::new("Illegal Number", token.pos))?,
        ),

        _ => return Err(ReError::new("Illegal range", token.pos)),
    };

    Ok(node)
}

fn resolve_char_class(token: &ReToken) -> ReResult<ASTClassNode> {
    let chars: Vec<char> = token.value.chars().collect();
    let reversed;
    let mut i = 0;

    if chars[0] == '^' {
        i = 1;
        reversed = true;
    } else {
        reversed = false;
    }

    let mut node_ranges: Vec<(char, char)> = vec![];
    let mut node_chars: Vec<char> = vec![];

    let mut last: Option<char> = None;
    let mut curr;
    let mut next;

    while i < chars.len() {
        curr = chars[i];
        next = chars.get(i + 1);

        if curr == '-' {
            match (last, next) {
                (Some(l), Some(n)) if *n != '^' => {
                    // 范围符号
                    node_ranges.push((l, *n));
                }
                _ => {
                    // 其他情况，直接当作普通字符
                    node_chars.push('-');
                }
            }
            i += 2;
        } else {
            node_chars.push(curr);
            i += 1
        }
        last = Some(curr);
    }

    Ok(ASTClassNode::charclass(node_chars, node_ranges, reversed))
}

///
/// 递归解析CST
///
fn to_ast_recursive(node: &CSTNode) -> ReResult<ASTNode> {
    let node = match node {
        // 这里的都是最底层叶子结点，停止递归
        CSTNode::Literal(token) => ASTNode::Literal(token.get_char()),
        CSTNode::Dot(_) => ASTNode::CharClass(ASTClassNode::dot()),
        CSTNode::MetaChar(token) => ASTNode::CharClass(resolve_meta(token)),
        CSTNode::CharClass(token) => ASTNode::CharClass(resolve_char_class(token)?),

        // 这里都是一元运算符节点，一次递归
        CSTNode::Star(node) => ASTNode::Star(Box::new(to_ast_recursive(node)?)),
        CSTNode::Question(node) => ASTNode::Question(Box::new(to_ast_recursive(node)?)),
        CSTNode::Plus(node) => ASTNode::Plus(Box::new(to_ast_recursive(node)?)),
        CSTNode::Range(node, token) => {
            let range_node = resolve_range(token)?;
            ASTNode::Range(Box::new(to_ast_recursive(node)?), range_node)
        }

        // 这里都是多元运算节点，多次递归
        CSTNode::Sequence(nodes) => {
            let mut ast_nodes = vec![];
            for x in nodes.iter() {
                ast_nodes.push(to_ast_recursive(x)?)
            }
            if ast_nodes.len() == 1 {
                // 如果只有一个元素则不嵌套，这里确实不美观
                ast_nodes.pop().unwrap()
            } else {
                ASTNode::Concatenation(ast_nodes)
            }
        }

        CSTNode::Alternation(nodes) => {
            let mut ast_nodes = vec![];
            for x in nodes.iter() {
                ast_nodes.push(to_ast_recursive(x)?)
            }
            if ast_nodes.len() == 1 {
                // 如果只有一个元素则不嵌套，这里确实不美观
                ast_nodes.pop().unwrap()
            } else {
                ASTNode::Alternation(ast_nodes)
            }
        }

        // 这是CST的抽象节点，一次递归并且丢弃推导式语义层次
        CSTNode::Group(node)
        | CSTNode::Quantified(node)
        | CSTNode::Atomic(node)
        | CSTNode::Expr(node) => to_ast_recursive(node)?,
    };

    Ok(node)
}

///
/// 将 CST 转换为 AST，丢弃CST的无实意语义层次，拆解单层 Alternation 和 Sequence，
/// 去除Token，简化为字符，ASTClassNode，ASTRangeNode
///
pub fn to_ast(ast: &CSTNode) -> ReResult<ASTNode> {
    to_ast_recursive(ast)
}
