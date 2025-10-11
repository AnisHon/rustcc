use crate::content_manager::ContentManager;
use crate::err::global_err::GlobalError;
use crate::err::lex_error::{LexError, LexResult};
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::IntSuffix::{L, U, UL};
use crate::lex::types::token_kind::{FloatSuffix, IntSuffix, LiteralKind, Symbol, TokenKind};
use crate::lex::{keyword, operator};
use std::sync::{mpsc, Arc};
use std::thread;
use unicode_ident::{is_xid_continue, is_xid_start};

///
///
///
/// # Members

pub struct Lex {
    content_manager: Arc<ContentManager>,
    curr_pos: usize,
    last_pos: usize, // 上次位置
}

impl Lex {
    pub fn new(content: Arc<ContentManager>) -> Self {
        Self {
            content_manager: content,
            curr_pos: 0,
            last_pos: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
        let chr = self.content_manager.chars(self.curr_pos).next();
        if let Some(chr) = chr {
            self.curr_pos += chr.len_utf8();
        }
        chr
    }

    fn peek(&mut self) -> Option<char> {
        self.content_manager.chars(self.curr_pos).next()
    }

    fn peek_n(&mut self, n: usize) -> Option<char> {
        self.content_manager.chars(self.curr_pos).nth(n)
    }

    fn skip_bytes(&mut self, n: usize) {
        self.curr_pos += n;
    }

    fn expect_patten(&mut self, patten: &str) -> bool {
        let mut chars = self.content_manager.chars(self.curr_pos);
        for chr in patten.chars() {
               if Some(chr) != chars.next() {
                   return false;
               }
        }
        true
    }

    fn expect(&mut self, chr: char) -> bool {
        self.peek() == Some(chr)
    }

    /// 取出patten
    fn get_patten(&self) -> &str {
        let patten = self.content_manager.str(self.last_pos..self.curr_pos);
        patten
    }

    fn clear_patten(&mut self) {
        self.last_pos = self.curr_pos;
    }

    /// 构建token，清空区间
    fn make_token(&mut self, kind: TokenKind) -> Token {
        let token = Token::new(self.last_pos, self.curr_pos, kind);
        self.clear_patten();
        token
    }

    /// 跳过一个单词（空字符作为边界）
    pub fn skip_word(&mut self) {
        while let Some(chr) = self.peek() {
            if chr.is_whitespace() {
                break;
            }
            self.next();
        }
    }

    pub fn recover(&mut self) {
        self.skip_word();
        self.clear_patten();
    }

    pub fn peek_next_is_digit(&mut self) -> bool {
        self.peek_n(1).map(|x| x.is_ascii_digit()).unwrap_or(false)
    }

    pub fn next_token(&mut self) -> LexResult<Option<Token>> {
        while let Some(chr) = self.peek() {
            let token = if chr.is_whitespace() {
                self.skip_whitespace();
                continue
            } else if chr.is_ascii_digit() || (chr == '.' && self.peek_next_is_digit()) {
                self.maybe_number_constant()?
            } else if is_xid_start(chr) {
                self.maybe_keyword_or_ident()?
            } else if chr == '"' || chr == '\'' {
                self.maybe_string_or_char()?
            } else if self.expect_patten("//") {
                self.skip_line_comment();
                continue
            } else if self.expect_patten("/*") {
                self.skip_block_comment()?;
                continue
            } else {
                self.maybe_operator()?
            };

            return Ok(Some(token))
        }
        Ok(None)
    }

    fn skip_whitespace(&mut self) {
        while let Some(chr) = self.peek() {
            if !chr.is_whitespace() {
                break
            }
            self.next();
        }
        self.clear_patten();
    }

    /// 匹配 E|e . [0-9]
    fn try_float(&mut self) -> LexResult<()> {
        let mut dot = false; // 小数点是否出现过
        let mut exp = false; // E是否出现过

        while let Some(chr) = self.peek() {
            match chr {
                '0'..='9' => {},
                'E' | 'e' => {
                    if exp {
                        break;
                    }
                    exp = true;
                    self.skip_bytes(1); // 消耗 'e'

                    // 消耗可能的 '+' '-'
                    if self.expect('+') || self.expect('-') {
                        self.skip_bytes(1);
                    }

                    if !self.peek().map(|x| x.is_ascii_digit()).unwrap_or(false) {
                        return Err(LexError::Exponent { pos: self.curr_pos });
                    }

                    continue; // 跳过 e 部分继续读取数字
                }
                '.' => {
                    if dot {
                        break;
                    }
                    dot = true;
                }
                _ => break
            }
            self.next();
        }

        Ok(())
    }

    /// 匹配 [a-f0-9]*
    /// # Return
    /// - `true`: 是`int`
    /// - `false`: 是`float`(遇到`.` `e` `E`)
    fn try_int(&mut self, base: u32) -> LexResult<bool> {
        while let Some(chr) = self.peek() {
            // 检测浮点标志
            if chr == '.' || chr == 'e' || chr == 'E' {
                return Ok(false); // 转为浮点
            } else if !chr.is_digit(16) {
                break;
            }

            if !chr.is_digit(base) {
                break;
            }

            self.next();
        }

        Ok(true) // 没有遇到'.'或'e'是整数
    }

    /// 返回是否有后缀
    fn try_int_suffix(&mut self) -> LexResult<Option<IntSuffix>> {
        use IntSuffix::*;
        let mut suffix = None;
        let mut valid = true;
        let beg = self.curr_pos;

        while let Some(chr) = self.peek() {
            // 检查是否是合法的前后缀
            let chr = match chr {
                'U' | 'u' => U,
                'L' | 'l' => L,
                chr if is_xid_continue(chr) => {  // 这个字符肯定有错
                    valid = false;
                    break
                }
                _ => break // 这些字符可以当做结束
            };

            suffix = match (suffix, chr) {
                (None, U) => Some(U),
                (None, L) =>  Some(L),
                (Some(L), L) => Some(LL),
                (Some(U), L) | (Some(L), U) => Some(UL),
                (Some(LL), U) | (Some(U), LL) | (Some(UL), L) => Some(ULL),
                _ => {
                    valid = false;
                    break
                }
            };
            self.skip_bytes(1);
        }
        if valid {
            Ok(suffix)
        } else {
            self.skip_word();
            let end = self.curr_pos;
            let content = self.content_manager.str(beg..end).to_owned();
            Err(LexError::Invalid { beg, end, invalid: "suffix", content, typ: "integer" })
        }
    }

    fn try_float_suffix(&mut self) -> LexResult<Option<FloatSuffix>> {
        use FloatSuffix::*;
        let beg = self.curr_pos;
        let chr = match self.peek() {
            Some(x) if x.is_ascii_digit() => x, // 是后缀字符
            Some(_) | None => return Ok(None), // 不是后缀字符
        };

        let float = match chr {
            'f' | 'F' => {
                self.skip_bytes(1);
                F
            },
            'l' | 'L' => {
                self.skip_bytes(1);
                L
            },
            _ => {
                self.skip_word();
                let end = self.curr_pos;
                let content = self.content_manager.str(beg..end).to_owned();
                return Err(LexError::Invalid { beg, end, invalid: "suffix", content, typ: "floating" })
            }
        };

        // 如果后缀是非后缀字符，继续匹配。
        if self.peek().map(|x| is_xid_continue(x)).unwrap_or(false) {
            self.skip_word();
            let end = self.curr_pos;
            let content = self.content_manager.str(beg..end).to_owned();
            Err(LexError::Invalid { beg, end, invalid: "suffix", content, typ: "floating" })
        } else {
            Ok(Some(float))
        }
    }

    fn maybe_number_constant(&mut self) -> LexResult<Token> {
        let mut base = 10;

        // 一定存在
        if self.peek().unwrap() == '0' {
            self.next();
            if let Some(x) = self.peek() {
                match x {
                    'x' | 'X' => {
                        self.skip_bytes(1); // 跳过 x
                        base = 16
                    },
                    'b' | 'B' => {
                        self.skip_bytes(1); // 跳过 b
                        base = 2
                    },
                    '0'..='9' => {
                        // 不用跳
                        base = 8
                    },
                    _ => {}
                }
            }
        }

        let is_int = self.try_int(base)?;
        let kind = if is_int {
            let patten = self.get_patten(); // 获取当前数字的部分
            let num = make_integer(patten, base);

            let suffix = self.try_int_suffix()?;

            LiteralKind::Integer { value: num, suffix }
        } else {
            self.try_float()?;
            let patten = self.get_patten(); // 获取当前数字的部分
            let value = Symbol::new(patten);
            let suffix = self.try_float_suffix()?;
            
            LiteralKind::Float { value, suffix }
        };

        let kind = TokenKind::Literal(kind);
        let token = self.make_token(kind);
        Ok(token)
    }

    /// 尝试解析为keyword或者ident
    fn maybe_keyword_or_ident(&mut self) -> LexResult<Token> {
        while let Some(chr) = self.peek() {
            if !is_xid_continue(chr) {
                break;
            }
            self.next();
        }
        
        let patten = self.get_patten();


        let kind = match keyword::KEYWORDS.get(patten) {
            None => make_ident(patten),
            Some(&x) => TokenKind::Keyword(x),
        };

        let token = self.make_token(kind);
        Ok(token)
    }


    /// 尝试解析为string 或 char
    fn maybe_string_or_char(&mut self) -> LexResult<Token> {
        let quote = self.peek().unwrap();
        self.skip_bytes(1); // 跳过   ' 或 “

        let mut esc = false; // 转义状态
        let mut closed = false; // 是否闭合
        while let Some(chr) = self.peek() {
            match chr {
                '\\' => esc = true, // 进入转译
                '\n' | '\r' => break, // 闭合
                chr if chr == quote && !esc => {
                    closed = true;
                    self.skip_bytes(1); // 跳过   ' 或 “
                    break;
                }
                _ => esc = false
            }
            self.next();
        }


        // 未闭合出错
        if !closed {
            return Err(LexError::MissingTerminating {pos: self.last_pos, chr: quote })
        }


        let patten = self.get_patten();
        let kind = match quote {
            '"' => make_string(patten),
            '\'' => make_char(patten),
            _ => unreachable!()
        };
        let token = self.make_token(kind);
        Ok(token)
    }
    

    /// 尝试解析operator
    fn maybe_operator(&mut self) -> LexResult<Token> {
        let mut pos = self.curr_pos;
        let mut state = operator::INIT_STATE;
        let mut last_state = operator::INIT_STATE;

        let mut iter = self.content_manager.chars(self.curr_pos).peekable();

        while let Some(&chr) = iter.peek() {
            if !chr.is_ascii() {
                break
            }
            state = match operator::find_next(state, chr) {
                Some(x) => x,
                None => break,
            };

            iter.next();
            pos += chr.len_utf8();

            if operator::STATES[state].is_some() {
                last_state = state;
                self.curr_pos = pos;
            };
        }

        self.curr_pos = pos;

        let kind = match operator::STATES[last_state].clone() {
            None => return Err(LexError::UnknownSymbol {pos: self.curr_pos, symbol: self.peek().unwrap()}),
            Some(x) => x,
        };

        Ok(self.make_token(kind))
    }

    fn skip_line_comment(&mut self) {
        self.skip_bytes(2); // 跳过  ‘//’
        let mut prev;
        let mut curr = '/';
        while let Some(chr) = self.peek() {
            prev = curr;
            curr = chr;

            match (prev, curr) {
                ('\r', '\n') => {
                    self.skip_bytes(1); // 指向最新位置
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

        self.clear_patten();
    }

    fn skip_block_comment(&mut self) -> LexResult<()> {
        self.skip_bytes(2); // 跳过  ‘/*’
        let mut prev;
        let mut curr = '*';
        let mut closed = false;

        while let Some(chr) = self.peek() {
            prev = curr;
            curr = chr;
            self.next();
            if (prev, curr) == ('*', '/') {
                closed = true;
                break;
            }
        }

        if closed {
            self.clear_patten();
            Ok(())
        } else {
            Err(LexError::UnterminatedComment { pos: self.last_pos })
        }
    }

}


pub fn make_ident(patten: &str) -> TokenKind {
    let symbol = Symbol::new(patten);
    TokenKind::Ident(symbol)
}

pub fn make_string(patten: &str) -> TokenKind {
    let value = Symbol::new(patten);
    TokenKind::Literal(LiteralKind::String { value })
}

pub fn make_char(patten: &str) -> TokenKind {
    let value = Symbol::new(patten);
    TokenKind::Literal(LiteralKind::Char { value })
}

pub fn make_integer(patten: &str, base: u32) -> u64 {
    let patten = match base {
        2 => &patten[2..],
        8 => &patten[1..],
        10 => patten,
        16 => &patten[2..],
        _ => unreachable!()
    };
    u64::from_str_radix(patten, base).unwrap()
}



/// 执行lex
///
/// # Arguments
/// - `lex`: lexer
/// - `error_tx`: 错误channel
///
/// # Returns
/// 解析后的Token
/// 
pub fn run_lexer<'a>(mut lex: Lex, error_rx: mpsc::Sender<GlobalError>) -> Vec<Token> {
    let mut tokens = Vec::new();
    loop {
        let tok = match lex.next_token() {
            Ok(x) => x,
            Err(err) => {
                // 出错恢复重试
                lex.recover();
                error_rx.send(GlobalError::LexError(err)).unwrap_or_else(|_| panic!("Global Error Handler Crashed"));
                continue;
            }
        };
        
        if let Some(tok) = tok {
            tokens.push(tok);
        } else {
            // 返回None，推入EOF结束
            let pos = lex.curr_pos;
            let token = Token::new(pos, pos, TokenKind::Eof);
            tokens.push(token);
            break
        }
    }
    
    tokens
}


