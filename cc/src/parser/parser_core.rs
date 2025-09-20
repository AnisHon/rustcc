use crate::err::parser_error::{ParserError, ParserResult};
use crate::parser::cst::DirectDeclarator::Paren;
use crate::parser::cst::{DeclarationSpecifiers, DirectDeclarator, InitDeclarator, SemanticValue, StorageClassSpecifier, TranslationUnit};
use crate::parser::parser_yy::{exec_action, get_action, get_goto, LRAction, EXPR_LENS, EXPR_NAMES, INIT_STATE};
use crate::types::lex::token::Token;
use crate::types::lex::token_kind::TokenKind;
use crate::types::symbol_table::SymbolTable;
use crate::util::try_type_name::try_type_name;
use std::cell::RefCell;
use std::iter::Peekable;
use std::rc::Rc;

/// 前端Parser，负责将Token流翻译为CST
/// Parser会维护一个最低限度的符号表用于转换前端的TYPE_NAME
/// 
pub struct Parser<I, ValueType = SemanticValue>
where
    I: Iterator<Item = Token>,

{
    iter: Peekable<I>,
    state_stack: Vec<usize>,        // 状态栈
    value_stack: Vec<ValueType>,    // 语义栈
    symbol_table: Rc<RefCell<SymbolTable<()>>>,
    last_reduced: usize, // 上一个规约
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(iter: I, symbol_table: Rc<RefCell<SymbolTable<()>>>) -> Self{
        Self {
            iter: iter.peekable(),
            state_stack: vec![INIT_STATE],  // 必须压入初始状态
            value_stack: vec![SemanticValue::default()], // 保持栈平衡，可去
            symbol_table: Rc::clone(&symbol_table),
            last_reduced: INIT_STATE,
        }
    }

    
    /// 
    /// 解析过程分为两类，Reduce类 和 Shift类，使用peek进行区分，peek是来自lexer的裸Token，完全可用于Reduce规约
    /// Shift类将使用next消耗Token，Token会经过Parser解析，得到精确Token（TYPE_NAME）
    /// 
    /// # Returns
    /// TranslationUnit: CST根节点
    pub fn parse(mut self) -> ParserResult<TranslationUnit> {
        while let Some(token) = self.iter.peek() {
            let curr_state = *self.state_stack.last().unwrap();
            let token_id = token.kind as usize;
            let action = get_action(curr_state, token_id);

            // peek 阶段只做粗略语义区分
            match action {
                LRAction::Reduce(expr) => self.reduce(expr)?,
                LRAction::Accept(expr) => {self.reduce(expr)?;break;}, // shift类多一个break
                _ => self.shift(curr_state)?, // 错误只会在shift阶段出现，交由shift处理
            }
        }

        let value = self.value_stack.pop().unwrap();
        value.into_translation_unit().map_err(|_| {
            let x = self.get_latest_token();
            ParserError::new(x.span, "Syntax Error", self.get_latest_expr())
        })
    }

    /// 处理规约
    /// # Error
    /// Reduce本身不会出错，但是符号表可能会出现重定义
    /// 
    fn reduce(&mut self, expr: usize) -> ParserResult<()> {
        println!("reduce: {}", EXPR_NAMES[expr]);
        let expr_len = EXPR_LENS[expr];
        let pop_idx = self.state_stack.len() - expr_len;

        self.state_stack.drain(pop_idx..); // 推出 状态栈
        let state = *self.state_stack.last().unwrap(); // 当前状态
        self.state_stack.push(get_goto(state, expr).unwrap()); // 压入状态栈

        let values: Vec<_> = self.value_stack.drain(pop_idx..).collect(); // 推出 语义栈
        let value = exec_action(expr, values); // 执行语义
        self.register_typedef(&value)   // 检查typedef
            .map_err(|err| err.with_name(EXPR_NAMES[expr]))?; // 补充name信息
        self.value_stack.push(value); // 压入语义栈

        self.last_reduced = expr; // 追踪上一个规约
        Ok(())
    }

    /// 定义是否是typedef
    fn is_type_def(specifiers: Option<&DeclarationSpecifiers>) -> bool {
        let specifiers = match specifiers {
            None => return false,
            Some(x) => x,
        };

        match specifiers {
            DeclarationSpecifiers::StorageClass(StorageClassSpecifier::Typedef(_), _) => true,
            DeclarationSpecifiers::StorageClass(_, rest) => Self::is_type_def(rest.as_deref()),
            DeclarationSpecifiers::TypeSpecifier(_, rest) => Self::is_type_def(rest.as_deref()),
            DeclarationSpecifiers::TypeQualifier(_, rest) => Self::is_type_def(rest.as_deref()),
        }
    }

    /// 记录typedef定义，用于Lexer识别TYPE_NAME
    fn register_typedef(&mut self, value: &SemanticValue) -> ParserResult<()> {
        let declaration = match value.as_declaration() { // declaration包含typedef
            None => return Ok(()),
            Some(x) => x
        };


        if !Self::is_type_def(Some(&declaration.specifiers)) {
            return Ok(());
        }

        let list = match &declaration.init_declarators {
            None =>  return Ok(()),
            Some(x) => &x.0,
        };

        for init_decl in list {
            let declarator = match init_decl {
                InitDeclarator::Plain(x) => x,
                InitDeclarator::Initialized(x, _) => x
            };

            if let Err(err) = self.recursive_register(&declarator.direct) {
                let token = self.iter.peek().unwrap();
                return Err(err.with_token(token)) // 补充pos line信息
            }
        }

        // 对当前符号修正
        let token = self.iter.peek_mut().unwrap();
        try_type_name(token, &self.symbol_table);

        Ok(())
    }

    /// 地柜解析typedef Token
    fn recursive_register(&mut self, declarator: &DirectDeclarator) -> ParserResult<()> {
        match &declarator {
            DirectDeclarator::Id(x) => self.symbol_table.borrow_mut().insert(x.value.as_string().unwrap(), ()),
            Paren(x) => self.recursive_register(&x.direct),
            DirectDeclarator::Array(x, _) => self.recursive_register(x),
            DirectDeclarator::FuncParams(x, _) => self.recursive_register(x),
            DirectDeclarator::FuncIdentifiers(x, _) => self.recursive_register(x),
        }
    }
    
    /// shift错误只会在shift阶段出现
    fn shift(&mut self, curr_state: usize) -> ParserResult<()> {
        let token = self.iter.next().unwrap();
        let token_id = token.kind as usize;
        let action = get_action(curr_state, token_id);

        // println!("{} {:?}", token.typ, TOKEN_CONTENTS[token.typ]);
        //
        // if matches!(action, LRAction::Error) {
        //     panic!("{} {} {}", curr_state, token, self.iter.peek().unwrap())
        // }

        let next_state = match action {
            LRAction::Shift(x) => x,
            LRAction::Error => return Err(self.error(token)),
            _ => unreachable!() // 经过peek后分别后不可能出现Reduce和Accept
        };

        // 针对typedef粗略的符号表作用域控制
        if token.kind == TokenKind::LBrace {
            self.symbol_table.borrow_mut().enter_scope();
        } else if token.kind == TokenKind::RBrace {
            self.symbol_table.borrow_mut().exit_scope();
        }

        let value = SemanticValue::Token(token);
        self.state_stack.push(next_state);
        self.value_stack.push(value);

        Ok(())
    }

    fn error(&mut self, token: Token) -> ParserError {    // 错误恢复没做
        let expr = self.get_latest_expr();
        ParserError::new(token.span, "Syntax Error", expr)
    }

    /// 获取最近的token,优先从Iter中获取
    fn get_latest_token(&mut self) -> Token {
        // 试图从Iter中取新的Token
        if let Some(x) = self.iter.peek() {
            return x.clone(); // 避免不必要的引用问题
        }

        // todo 如果向前无法找到，则从语义栈反向遍历，深度优先搜索每个value查询最近Token

        // 如果都找不到，我不知道会不会发生，也不知道怎么处理
        unreachable!();
    }
    

    /// 获取最近的推导式名
    fn get_latest_expr(&mut self) -> &'static str {
        EXPR_NAMES[self.last_reduced]
    }

}





