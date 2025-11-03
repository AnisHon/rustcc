use super::lex_yy::{exec_action, find_next, INIT_STATE};
use crate::err::global_err::GlobalError;
use crate::err::lex_error::{LexError, LexResult};
use crate::types::lex::token::Token;
use crate::types::lex::token_kind::TokenKind;
use num_traits::FromPrimitive;
use std::collections::vec_deque::Drain;
use std::collections::VecDeque;
use std::io::{BufReader, Read};
use std::sync::mpsc;
use std::thread;

/// 缓冲区默认块大小
const BUFF_BLOCK: usize = 4096;

pub(super) enum LexMode {
    Normal,         // 正常模式
    #[allow(dead_code)]
    String,         // 保留未使用
    LineComment,     // 行注释
    BlockComment,    // 块注释
    WhileSpace,     // 空白字符
}


///
/// 维护了一个环形缓冲区
/// 维持三个位置，所有位置都是物理位置，而非环形缓冲区的相对位置
/// - `pos`: 当前指针位置
/// - `last_pos`: 最近一次成功的位置
/// - `cursor_pos`: 文件指针位置
///
///
/// # Members
/// - `mode`: 工作模式
/// - `buff`: 环形缓冲区
/// - `reader`: 流
/// - `curr_pos`: lex工作当前位置，（文件指针，绝对位置）
/// - 'curr_state'
/// - `last_pos`: lex工作上一个位置 （文件指针，绝对位置）
/// - `last_state`: 上一个状态
/// - `last_tok`: 上一个tok
/// - `cursor_pos`: 文件指针真实位置
/// - `errors`: 错误缓冲区，用于错误恢复
pub struct Lex<R: Read> {
    mode: LexMode,
    buff: VecDeque<char>,
    reader: BufReader<R>,
    curr_pos: usize,
    curr_state: usize,
    last_pos: Option<usize>,
    last_state: Option<usize>,
    last_tok: Option<usize>,
    cursor_pos: usize,
    errors: Vec<LexError>,
}

impl<R: Read> Lex<R> {
    pub fn new(reader: R) -> Self {
        Self {
            mode: LexMode::Normal,
            buff: VecDeque::with_capacity(BUFF_BLOCK),
            reader: BufReader::new(reader),
            curr_pos: 0,
            curr_state: INIT_STATE,
            last_pos: None,
            last_state: None,
            last_tok: None,
            cursor_pos: 0,
            errors: Vec::new()
        }
    }

    pub(super) fn set_mode(&mut self, mode: LexMode) {
        self.mode = mode;
    }

    pub fn next_token(&mut self) -> Option<Token> {
        while let Some(chr) = self.peek() {
            match self.mode {
                LexMode::Normal => match self.handle_normal(chr) {
                    Ok(Some(token)) => return Some(token),
                    Ok(None) => { /* 继续匹配 */ }
                    Err(err) => self.errors.push(err), // 错误自动恢复，添加到errors，继续匹配
                }
                LexMode::LineComment => self.handle_comment(),
                LexMode::BlockComment => self.handle_block_comment(),
                LexMode::String => self.reset_state(), // 未使用
                LexMode::WhileSpace => self.handle_white_space()
            }
        }
        // 数据流结束，查看是否有剩余部分，以及当前流是否匹配成功
        // 流空了，没问题
        if self.buff.is_empty() {
            return None;
        }

        // 流没空且没有匹配到东西，出错
        if !self.has_last() {
            // 直接弹出来，不回去重新匹配，不恢复状态
            let content: String = self.pop_buff(self.curr_pos).collect();

            let pos = self.curr_pos;

            // 判断类型，分支不会很多，不用设计查表
            let err = match content.as_str() {
                s if s.starts_with("/*") => LexError::UnterminatedComment { pos },
                s if s.starts_with("\"") => LexError::MissingTerminating { pos, content: "\"" },
                s if s.starts_with("'")  => LexError::MissingTerminating { pos, content: "'" },
                s => LexError::InvalidToken { pos, symbol: s.to_string() },
            };

            self.errors.push(err);
            return None;
        }

        // 匹配到东西，构建token
        let token = self.make_token();
        Some(token)
    }

    pub fn get_errors(&self) -> &[LexError] {
        &self.errors
    }

    ///
    /// 处理普通模式，每次做一次转移
    /// - 所有的正常转义都会调用`next`让指针始终指向最新位置
    /// - save保存的state是旧位置
    /// - pop传递的也是
    ///
    /// # Returns
    /// 可能会出错，交给主循环处理
    /// 当返回None的时候要继续循环
    ///
    fn handle_normal(&mut self, chr: char) -> LexResult<Option<Token>> {
        let mut token = None;
        // 正常转移
        if let Some(x) = find_next(self.curr_state, chr) {
            let tok = exec_action(x, self);
            self.next();
            // 得到tok，保存状态
            if let Some(tok) = tok {
                self.save_state(tok);
            }
            self.curr_state = x;
        } else { // 匹配失败
            // 没有成功结果被保存，出错
            if !self.has_last() {
                let err_pos = self.curr_pos;
                let symbol = self.recover();
                return Err(LexError::InvalidToken{ pos: err_pos, symbol })
            }
            // 有结果被保存
            // 构建Token
            token = Some(self.make_token());
            // 恢复状态
            self.reset_state();
        }
        Ok(token)
    }

    /// 处理单行注释
    fn handle_comment(&mut self) {

        let mut prev;
        let mut curr = '\x00';
        while let Some(chr) = self.peek() {

            prev = curr;
            curr = chr;

            match (prev, curr) {
                ('\r', '\n') => {
                    self.next(); // 指向最新位置
                    break;
                }
                ('\r', _) | ('\n', _)=> { // 当前位置就是最新位置
                    break;
                }
                (_, _) => {
                    self.next(); // 继续匹配
                }
            }
        }

        // 弹出注释串，清空保存，重制状态
        self.pop_buff(self.curr_pos);
        self.clear_save();
        self.reset_state();
    }

    /// 处理多行注释
    fn handle_block_comment(&mut self) {
        let mut prev;
        let mut curr = '\x00';

        while let Some(chr) = self.peek() {
            prev = curr;
            curr = chr;
            self.next();
            if (prev, curr) == ('*', '/') {
                break;
            }
        }

        // 弹出注释串，清空保存，重制状态
        self.pop_buff(self.curr_pos);
        self.clear_save();
        self.reset_state();
    }

    fn handle_white_space(&mut self) {
        while let Some(chr) = self.peek() {
            // 跳过空白符号
            if !chr.is_whitespace() {
                break;
            } else {
                self.next();
            }
        }

        // 弹出空白，清空保存，重制状态
        self.pop_buff(self.curr_pos);
        self.clear_save();
        self.reset_state();
    }

    /// 出错后，对当前状态进行恢复，返回出错单词
    fn recover(&mut self) -> String {
        // 跳过当前词
        while let Some(chr) = self.peek() {
            if chr.is_ascii_whitespace() {
                break
            }
            self.next();
        }

        // 恢复状态
        self.reset_state();
        // 弹出出错符号
        self.pop_buff(self.curr_pos).collect()
    }

    fn buff_pos(&self, cursor: usize) -> usize {
        assert!(cursor <= self.cursor_pos);
        let ring_len = self.buff.len();
        ring_len - (self.cursor_pos - self.curr_pos)
    }

    /// 检查是否需要读取，语义是要读取/消耗pos位置
    /// 保证`pos`始终小于`cursor_pos`
    fn check_read(&mut self, pos: usize) {
        // 不需要加载
        if pos < self.cursor_pos {
            return;
        }

        // 按块加载
        let mut chunk = String::with_capacity(BUFF_BLOCK);
        let n = self.reader
            .by_ref()
            .take(BUFF_BLOCK as u64)
            .read_to_string(&mut chunk)
            .unwrap_or_else(|e| panic!("{}", e));
        for chr in chunk[0..n].chars() {
            self.buff.push_back(chr);
            self.cursor_pos += 1;
        }
    }

    /// 拿到下一个char，自动选择流或者缓冲区，更新pos
    /// curr_pos可取cursor + 1，表示结束
    fn next(&mut self) -> Option<char> {
        let chr = self.peek();
        // 最多指向cursor + 1
        if self.curr_pos <= self.cursor_pos {
            self.curr_pos += 1;
        }
        chr
    }

    fn peek(&mut self) -> Option<char> {
        self.check_read(self.curr_pos);
        let buff_idx = self.buff_pos(self.curr_pos);

        self.buff.get(buff_idx).copied()
    }

    /// 根据pos从头弹出缓冲区，转换成字符串，不包含pos
    fn pop_buff(&mut self, pos: usize) -> Drain<char> {
        let pos = self.buff_pos(pos);
        self.buff.drain(0..pos)
    }

    /// 通过load_state 构建token
    fn make_token(&mut self) -> Token {
        assert!(self.has_last());
        // 加载状态，提取文本，构建token
        let kind = self.load_state();
        let content = self.pop_buff(self.curr_pos);
        let len = content.len();
        let content: String = content.collect();
        let beg = self.curr_pos - len;
        let end = self.curr_pos - 1;

        let kind = TokenKind::from_usize(kind).unwrap();
        Token::new(beg, end, kind, content)
    }

    /// 保存当前状态
    fn save_state(&mut self, tok: usize) {
        self.last_state = Some(self.curr_state);
        self.last_pos = Some(self.curr_pos);
        self.last_tok = Some(tok);
    }

    /// 加载上一个状态，加载后会清空，返回保存的tok结果
    fn load_state(&mut self) -> usize {
        assert!(self.has_last());
        self.curr_state = self.last_state.unwrap();
        self.curr_pos = self.last_pos.unwrap();
        // 加载后清空
        self.last_state = None;
        self.last_pos = None;

        let tok = self.last_tok;
        self.last_tok = None;
        tok.unwrap()
    }

    /// 清空保存的状态
    fn clear_save(&mut self) {
        self.last_state = None;
        self.last_pos = None;
        self.last_tok = None;
    }

    /// 是否存在 last_state和last_pos
    fn has_last(&self) -> bool {
        self.last_state.is_some() && self.last_pos.is_some()
    }

    /// 重制状态
    fn reset_state(&mut self) {
        self.mode = LexMode::Normal;
        self.curr_state = INIT_STATE;
    }

}


/// 异步lex
///
/// # Members
/// - `semantic`: lexer
/// - `token_tx`: token channel，总体速度匹配，但防止积压，使用有界队列
/// - `error_tx`: 错误channel
///
pub struct AsyncLex<R: Read> {
    pub lex: Lex<R>,
    pub token_tx: crossbeam_channel::Sender<Token>,
    pub error_rx: mpsc::Sender<GlobalError>,
}


impl <R: Read + Send + 'static> AsyncLex<R> {

    pub fn new(lex: Lex<R>, token_tx: crossbeam_channel::Sender<Token>, error_rx: mpsc::Sender<GlobalError>) -> AsyncLex<R> {
        Self { lex, token_tx, error_rx }
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            while let Some(x) = self.lex.next_token() {
                // 如果出错了，直接报错
                if self.token_tx.send(x).is_err() {
                    break;
                };
            }

            // 构建 EOF token 发过去
            let pos = self.lex.curr_pos;
            let token = Token::new(pos, pos, TokenKind::EOF, "".to_string());
            let _ = self.token_tx.send(token); // 成功与否不关心
            drop(self.token_tx); // 关闭通道
            
            for x in self.lex.errors {
                // 全局错误通道永远不会关闭
                self.error_rx.send(GlobalError::LexError(x)).unwrap_or_else(|_| panic!("Global Error Handler Crashed"));
            }
        });
    }
}