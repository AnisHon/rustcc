use crate::err::lex_error::{LexError, LexResult};
use super::lex_yy::{find_next, INIT_STATE, TERMINATE_MAP};
use crate::types::lex::token::Token;
use std::collections::VecDeque;
use std::io::{BufReader, Read};

/// 缓冲区默认块大小
const BUFF_BLOCK: usize = 4096;

enum LexMode {
    Normal,         // 正常模式
    String,         // 未实现
    LineCommon,     // 行注释
    BlockCommon,    // 块注释
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
    cursor_pos: usize,

    errors: Vec<LexError>
}

impl <R: Read> Lex<R> {
    pub fn new(reader: R) -> Self {
        Self {
            mode: LexMode::Normal,
            buff: VecDeque::with_capacity(BUFF_BLOCK),
            reader: BufReader::new(reader),

            curr_pos: 0,
            curr_state: INIT_STATE,
            last_pos: None,
            last_state: None,
            cursor_pos: 0,

            errors: Vec::new()
        }
    }

    pub fn next_token(&mut self) -> LexResult<Option<Token>> {
        while let Some(_) = self.peek() {
            match self.mode {
                LexMode::Normal => {
                    let token = self.handle_normal()?;
                    if token.is_some() {
                        return Ok(token);
                    }
                }
                LexMode::LineCommon => self.handle_common(),
                LexMode::BlockCommon => self.handle_common(),
                LexMode::String => self.reset_state(), // 未使用
            }
        }

        Ok(None)
    }

    fn handle_normal(&mut self) -> LexResult<Option<Token>> {
        while let Some(chr) = self.peek() {

            if let Some(x) = find_next(self.curr_state, chr) {
                if TERMINATE_MAP[x] {
                    self.save_state();
                }
                self.curr_pos = x;
            }
        }
        Ok(None)
    }

    ///
    /// 处理换行
    ///
    fn handle_common(&mut self) {

        let mut prev;
        let mut curr = '\x00';
        while let Some(chr) = self.peek() {
            prev = curr;
            curr = chr;

            if curr == '\n' {
                self.next();
                break
            }

            match (prev, curr) {
                ('\r', '\n') => {
                    self.next();
                    break;
                }
                ('\r', _) => {
                    break;
                }
                (_, _) => {
                    self.next();
                }
            }
        }

        self.pop_buff(self.curr_pos);
        self.reset_state();
    }

    fn handle_block_common(&mut self) {
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

        self.pop_buff(self.curr_pos);
        self.reset_state();
    }

    /// 出错后，对当前状态进行恢复
    ///
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
        self.pop_buff(self.curr_pos)
    }

    fn buff_pos(&self, cursor: usize) -> usize {
        assert!(cursor <= self.cursor_pos);
        let ring_len = self.buff.len();
        let buff_pos = ring_len - (self.cursor_pos - self.curr_pos) - 1;
        buff_pos
    }

    /// 检查是否需要读取
    fn check_read(&mut self, pos: usize) {
        // 不需要
        if pos < self.cursor_pos {
            return;
        }

        let mut chunk = String::with_capacity(BUFF_BLOCK);
        let n = self.reader
            .by_ref()
            .take(4096)
            .read_to_string(&mut chunk)
            .unwrap_or_else(|e| panic!("{}", e));

        self.buff.extend(chunk[0..n].chars());
        self.cursor_pos += n;
    }

    /// 拿到下一个char，自动选择流或者缓冲区，更新pos
    fn next(&mut self) -> Option<char> {
        let chr = self.peek();
        if chr.is_some() {
            self.curr_pos += 1;
        }
        chr
    }

    fn peek(&mut self) -> Option<char> {
        let buff_idx = self.buff_pos(self.curr_pos);
        self.check_read(buff_idx);

        if buff_idx >= self.buff.len() {
            return None;
        }

        let char = self.buff[buff_idx];

        Some(char)
    }

    /// 根据pos从头弹出缓冲区，转换成字符串，不包含pos
    fn pop_buff(&mut self, pos: usize) -> String {
        let pos = self.buff_pos(pos);
        let value = self.buff.drain(0..pos).collect();
        value
    }

    /// 保存当前状态
    fn save_state(&mut self) {
        self.last_state = Some(self.curr_state);
        self.last_pos = Some(self.curr_state);
    }

    /// 加载上一个状态，如果没有返回false，加载后会清空
    fn load_state(&mut self) -> bool {
        if !self.has_last() {
            return false;
        }
        self.curr_state = self.last_state.unwrap();
        self.curr_pos = self.last_pos.unwrap();
        // 加载后清空
        self.last_state = None;
        self.last_pos = None;
        true
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
