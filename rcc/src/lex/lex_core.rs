use crate::err::global_err::GlobalError;
use std::io::{BufReader, Read};
use std::iter::Peekable;
use std::str::{Chars, FromStr};
use std::sync::mpsc;
use std::thread;
use unicode_ident::{is_xid_continue, is_xid_start};
use crate::content_manager::ContentManager;
use crate::err::lex_error::{LexError, LexResult};
use crate::lex::{keyword, operator};
use crate::lex::types::token::Token;
use crate::lex::types::token_kind::{FloatSuffix, IntSuffix, Keyword, LiteralKind, Symbol, TokenKind};
use crate::util::utf8::unescape_str;

///
///
///
/// # Members

pub struct Lex<'a> {
    content_manager: &'a ContentManager,
    curr_pos: usize,
    last_pos: usize, // 上次位置
}

impl<'a> Lex<'a> {
    pub fn new(content: &'a ContentManager) -> Self {
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
    fn try_suffix(&mut self) -> bool {
        let mut flag = false;
        while let Some(chr) = self.peek() {
            if !chr.is_ascii_alphanumeric() {
                break
            }
            flag = true;
            self.next();
        }
        flag
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
        if !is_int {
            self.try_float();
        }

        let beg = self.last_pos;
        let end = self.curr_pos;
        let patten = self.content_manager.str(beg..end);

        let has_suffix = self.try_suffix();
        let suffix_beg = end;
        let suffix_end = self.curr_pos;
        let suffix = match has_suffix {
            true => Some(self.content_manager.str(suffix_beg..suffix_end)),
            false => None,
        };

        let kind = if is_int {
            let num = match base {
                2 => make_bin(patten),
                8 => make_oct(patten),
                10 => make_dec(patten),
                16 => make_hex(patten),
                _ => unreachable!()
            }?;

            let suffix = match suffix {
                Some(x) => Some(make_int_suffix(x)?),
                None => None,
            };
            LiteralKind::Integer { value: num, suffix }
        } else {
            let patten = self.content_manager.str(beg..end);
            let num = make_float(patten)?;

            let suffix = match suffix {
                Some(x) => Some(make_float_suffix(x)?),
                None => None,
            };
            LiteralKind::Float { value: num, suffix }
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


/// 异步lex
///
/// # Members
/// - `types`: lexer
/// - `token_tx`: token channel，总体速度匹配，但防止积压，使用有界队列
/// - `error_tx`: 错误channel
///
// pub struct AsyncLex<R: Read> {
//     pub lex: Lex<R>,
//     pub token_tx: crossbeam_channel::Sender<Token>,
//     pub error_rx: mpsc::Sender<GlobalError>,
// }
//
//
// impl <R: Read + Send + 'static> AsyncLex<R> {
//
//     pub fn new(lex: Lex<R>, token_tx: crossbeam_channel::Sender<Token>, error_rx: mpsc::Sender<GlobalError>) -> AsyncLex<R> {
//         Self { lex, token_tx, error_rx }
//     }
//
//     pub fn start(mut self) {
//         thread::spawn(move || {
//             while let Some(x) = self.lex.next_token() {
//                 // 如果出错了，直接报错
//                 if self.token_tx.send(x).is_err() {
//                     break;
//                 };
//             }
//
//             // 构建 EOF token 发过去
//             let pos = self.lex.curr_pos;
//             let token = Token::new(pos, pos, TokenKind::Eof);
//             let _ = self.token_tx.send(token); // 成功与否不关心
//             drop(self.token_tx); // 关闭通道
//
//             for x in self.lex.errors {
//                 // 全局错误通道永远不会关闭
//                 self.error_rx.send(GlobalError::LexError(x)).unwrap_or_else(|_| panic!("Global Error Handler Crashed"));
//             }
//         });
//     }
// }


pub fn make_ident(patten: &str) -> TokenKind {
    let symbol = Symbol(patten.to_owned());
    TokenKind::Ident(symbol)
}
pub fn make_bin(patten: &str) -> LexResult<u64> {
    let patten = &patten[2..];
    // todo
    Ok(u64::from_str_radix(patten, 2).unwrap())
}

pub fn make_oct(patten: &str) -> LexResult<u64> {
    let patten = &patten[1..];
    // todo
    Ok(u64::from_str_radix(patten, 8).unwrap())
}

pub fn make_dec(patten: &str) -> LexResult<u64> {
    // todo
    Ok(u64::from_str_radix(patten, 10).unwrap())
}

pub fn make_hex(patten: &str) -> LexResult<u64> {
    let patten = &patten[2..];
    // todo
    Ok(u64::from_str_radix(patten, 16).unwrap())
}


pub fn make_float(patten: &str) -> LexResult<f64> {
    // todo
    Ok(f64::from_str(patten).unwrap())
}

fn make_int_suffix(value: &str) -> LexResult<IntSuffix> {
    todo!()
}

fn make_float_suffix(value: &str) -> LexResult<FloatSuffix> {
    todo!()
}
pub fn make_string(patten: &str) -> TokenKind {
    // todo 
    let value = unescape_str(patten);
    TokenKind::Literal(LiteralKind::String { value })
}

pub fn make_char(patten: &str) -> TokenKind {
    // todo
    let value = unescape_str(patten);
    TokenKind::Literal(LiteralKind::Char { value })
}


#[test]
fn test() {
    let content = include_str!("../../resources/test.c");
    let manager = ContentManager::new(content.to_string());
    let mut lex = Lex::new(&manager);
    while let Some(x) = lex.next_token().unwrap() {
        println!("{:?}.", x)
    }
}
