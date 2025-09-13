use std::cell::RefCell;
use crate::err::lex_error::{LexError, LexResult};
use crate::lex::lex_yy::{find_next, find_token, TokenType, INIT_STATE};
use crate::types::token::Token;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::rc::Rc;
use crate::types::symbol_table::SymbolTable;
use crate::util::try_type_name::try_type_name;

pub struct Lex<R: Read> {
    pos: usize,  // 当前指针位置，可以回退，但始终在buff范围内
    line: usize,
    state: usize,
    reader_pos: usize,  // 流指针位置，始终指向最新的位置，只增不减
    buff: VecDeque<char>,
    reader: BufReader<R>,
    last_pos: Option<usize>,
    last_state: Option<usize>,
    symbol_table: Rc<RefCell<SymbolTable<()>>>
}

impl <R: Read> Lex<R> {
    pub fn new(reader: R, symbol_table: Rc<RefCell<SymbolTable<()>>>) -> Self {
        Self {
            pos: 0,
            line: 0,
            state: INIT_STATE,
            reader_pos: 0,
            buff: VecDeque::new(),
            reader: BufReader::new(reader),
            last_pos: None,
            last_state: None,
            symbol_table
        }
    }

    pub fn next_token(&mut self) -> Option<LexResult<Token>> {
        // 状态机转移
        loop {
            let chr =  match self.next_char() {
                Some(c) => c, // 正常
                None if self.state == INIT_STATE => return None, // 未读取流结束
                _ => break  // 已经读取流结束，交给下面处理
            };

            self.state = match find_next(self.state, chr) {
                Some(x) => x, // 正常转移

                None if self.has_load_state() => break, // 回溯

                _ => { // 错误转移
                    return Some(Err(self.build_err("use of undeclared identifier")))
                }
            };

            if let Some(x) = find_token(self.state) {
                if x == TokenType::BlockComment {
                    let closed = self.read_block_comment(); // 手动处理注释
                    if !closed {
                        return Some(Err(self.build_err("unterminated comment /*")));
                    }
                }
                self.save_state();
            }

        }


        // 处理状态
        match find_token(self.state) {
            None => Some(Err(self.build_err("unterminated"))),
            Some(typ) => {
                self.load_state();
                let (value, size) = self.pop_buff();
                let pos = self.pos - size;
                self.reset_state(); // 重要！必须重置状态
                Some(Ok(Token::new(pos, typ, value)))
            }
        }
    }

    // 手动处理注释，返回bool表示注释是否闭合
    fn read_block_comment(&mut self) -> bool {
        self.pos -= 1; // 回退一个，保证存在
        let mut prev: char;
        let mut curr: char = self.next_char().unwrap();
        let mut closed = false;
        while let Some(chr) =self.next_char() {
            prev = curr;
            curr = chr;
            if prev == '*' && curr == '/' {
                closed = true;
                break;
            }
        }
        closed
    }

    fn build_err(&self, msg: &str) -> LexError {
        LexError::new(
            self.get_buff_pos(),
            self.line,
            msg,
            self.buff.iter().collect()
        )
    }

    fn get_buff_pos(&self) -> usize {
        self.pos - (self.reader_pos - self.buff.len())
    }

    /// 读取一行，如果到达文件末尾则返回false，失败则直接终止程序
    fn read_line(&mut self) -> bool {
        self.line += 1; // 行数增加
        let mut buff_str = String::new();
        let size =self.reader.read_line(&mut buff_str).expect("read line error occurred");
        let chars = buff_str.chars();

        // 推入buff，更新reader_pos
        for chr in chars {
            self.buff.push_back(chr);
            self.reader_pos += 1;
        }
        size != 0 // 是否读取成功
    }

    /// 拿到下一个char，自动选择流或者缓冲区，更新pos
    fn next_char(&mut self) -> Option<char> {
        // 计算偏移值, pos - buff_begin_pos
        let buff_pos = self.get_buff_pos();
        if buff_pos >= self.buff.len() && !self.read_line() {
            return None;
        }

        let char = self.buff[buff_pos];
        self.pos += 1;

        Some(char)
    }

    /// 根据pos从头弹出缓冲区，转换成字符串
    fn pop_buff(&mut self) -> (String, usize) {
        let pos = self.get_buff_pos();

        let value = self.buff.drain(0..pos).collect(); // 不包含pos
        (value, pos)

    }

    /// 保存当前状态
    fn save_state(&mut self) {
        self.last_state = Some(self.state);
        self.last_pos = Some(self.pos);
    }

    /// 是否存在保存的state
    fn has_load_state(&self) -> bool {
        self.last_state.is_some() && self.last_pos.is_some()
    }

    /// 加载上一个状态，清空state，如果没有触发panic
    fn load_state(&mut self) {
        if !self.has_load_state() {
            panic!("last_state or last_pos is None");
        }
        self.state = self.last_state.unwrap();
        self.pos = self.last_pos.unwrap();
        self.last_state = None;
        self.last_pos = None;
    }

    fn reset_state(&mut self) {
        self.state = INIT_STATE;
    }

}

impl <R: Read> Iterator for Lex<R> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = self.next_token()?;

            let mut token = match result { // 错误处理未实现
                Ok(x) => x,
                Err(err) => panic!("{:?}", err)
            };


            if token.ignore() {  // 过滤无用Token
                continue;
            }

            try_type_name(&mut token, &self.symbol_table); // 尝试使用符号表转换token

            return Some(token);
        }
    }
}
