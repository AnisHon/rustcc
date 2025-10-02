use crate::err::parser_error::{ParserError, ParserResult};
use crate::parser::parser_yy::{get_action, get_goto, LRAction, ACTION_CODES, EXPR_LENS, EXPR_NAMES, INIT_STATE};
use crate::parser::token_stream::TokenStream;
use crate::types::ast::ast_nodes::TranslationUnit;
use crate::types::ast::sematic_value::SemanticValue;
use crate::types::lex::token::Token;
use crate::types::parser_context::ParserContext;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

// todo typedef规约后可能需要修正当前peek

/// 前端Parser，负责将Token流翻译为CST
/// Parser会维护一个最低限度的符号表用于转换前端的TYPE_NAME
/// 
pub struct Parser {
    stream: TokenStream,
    context: Rc<RefCell<ParserContext>>,
    state_stack: Vec<usize>,        // 状态栈
    value_stack: Vec<SemanticValue>,    // 语义栈
    last_reduced: usize, // 上一个规约
}

impl Parser {
    pub fn new(token_stream: TokenStream, context: Rc<RefCell<ParserContext>>) -> Self {
        let state_stack = vec![INIT_STATE];
        let value_stack = vec![SemanticValue::default()];
        let last_reduced = INIT_STATE;
        Self {
            stream: token_stream,
            context,
            state_stack,  // 必须压入初始状态
            value_stack, // 保持栈平衡，可去
            last_reduced,
        }
    }

    
    /// 
    /// # Returns
    /// - `TranslationUnit`: CST根节点
    ///
    pub fn parse(mut self) -> ParserResult<TranslationUnit> {
        while let Some(token) = self.stream.peek() {
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
        value.try_into().map_err(|_| {
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

        let _ = self.state_stack.split_off(pop_idx); // 弹出 状态栈
        let state = *self.state_stack.last().unwrap(); // 当前状态
        self.state_stack.push(get_goto(state, expr).unwrap()); // 压入状态栈


        let values: Vec<_> = self.value_stack.split_off(pop_idx); // 推出 语义栈
        let value = self.exec_action(expr, values); // 执行
        self.value_stack.push(value); // 压入语义栈

        // 检查更新typename
        let mut context = self.context.borrow_mut();
        if context.sync_token {
            self.stream.sync();
            context.sync_token = false;
        }

        self.last_reduced = expr; // 追踪上一个规约
        Ok(())
    }

    fn exec_action(&mut self, expr: usize, argument: Vec<SemanticValue>) -> SemanticValue {
        match ACTION_CODES[expr] {
            None => SemanticValue::default(),
            Some(handler) => handler(argument, self.context.borrow_mut().deref_mut())
        }
    }

    /// shift错误只会在shift阶段出现
    fn shift(&mut self, curr_state: usize) -> ParserResult<()> {
        let token = self.stream.next().unwrap();
        let token_id = token.kind as usize;
        let action = get_action(curr_state, token_id);

        let next_state = match action {
            LRAction::Shift(x) => x,
            LRAction::Error => return Err(self.error(token)),
            _ => unreachable!() // 经过peek后分别后不可能出现Reduce和Accept
        };


        let value = SemanticValue::TokenNode(token);
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
        if let Some(x) = self.stream.peek() {
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

