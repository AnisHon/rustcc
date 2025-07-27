//
// 通过手写实现的正则lexer
// crated: 2025-07-26
// author: anishan
//

use std::cmp::Ordering;
use std::iter::Enumerate;
use std::str::Chars;
use crate::lex::re_err::{ReError, ReResult};
use crate::lex::types::{ReToken, ReTokenType};



///
/// 将正则字符串解析为Token
/// # 过程
/// 解析过程分多次
/// 1. 初始分割为Token，转换转义字符 元字符 字符
/// 2. 处理char class
/// 3. 处理范围语法（未实现，待定）
#[cfg_attr(test, allow(dead_code))]
pub fn re2tokens(regex: &str) -> ReResult<Vec<ReToken>> {
    let tokens = primary_token_split(regex)?;       // 初始分割
    let tokens = char_class_token(regex, tokens)?;  // 构建char class

    Ok(tokens)
}

///
/// token初等分割，负责转义字符和元字符与字符常量解析
///
fn primary_token_split(regex: &str) -> ReResult<Vec<ReToken>> {
    let mut tokens = Vec::new();
    let mut chars = regex.chars().enumerate();

    while let Some((pos, chr)) = chars.next() {
        let token = match chr {
            '\\' => handle_escape(&mut chars),
            _ if ReToken::is_meta_char(chr) => Ok(ReToken::new(ReTokenType::from_char(chr).unwrap(), chr.to_string(), pos)),
            _ => Ok(ReToken::new(ReTokenType::Literal, chr.to_string(), pos))
        };

        match token {
            Ok(mut token) => { token.pos = pos;tokens.push(token) }
            Err(msg) => return Err(ReError::new(msg, pos, regex.into()))
        }
    }

    Ok(tokens)
}

///
/// 将十六进制数字字符串转换为char
///
fn codepoint2char(hex: &str) -> Result<char, String> {
    u32::from_str_radix(&hex, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or(format!("Unicode error can't decode bytes {}", hex))
}

///
/// 处理unicode码点 \uXXXX
///
fn unicode_escape(chars: &mut Enumerate<Chars>) -> Result<char, String> {
    let hex: String = chars.take(4).map(|(_, c)| c).collect();
    if hex.len() != 4 {
        return Err(format!("Unicode error wrong format {} (expect \\uXXXX)", hex));
    }
    codepoint2char(&hex)
}

///
/// 处理ascii \xXX
///
fn ascii_escape(chars: &mut Enumerate<Chars>) -> Result<char, String> {
    let hex: String = chars.take(2).map(|(_, c)| c).collect();
    if hex.len() != 4 {
        return Err(format!("Unicode error wrong format {} (expect \\xXX)", hex));
    }
    codepoint2char(&hex)
}

///
/// 处理所有逃逸字符 字符类 元字符 码点
///
fn handle_escape(chars: &mut Enumerate<Chars>) -> Result<ReToken, String> {

    let chr = match chars.next() {
        Some((_, chr)) => chr,
        None => return Err(String::from("Illegal escape"))
    };

    // pos字段由主循环填写
    let token = if ReToken::is_meta_char(chr) {
        ReToken::new(ReTokenType::Literal, chr.to_string(), 0)
    } else if ReToken::is_class_char(chr) {
        let typ = ReTokenType::from_char(chr).unwrap();
        ReToken::new(typ, chr.to_string(), 0)
    } else {
        let value = match chr {
            't' => Ok('\t'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            'u' => unicode_escape(chars),    // unicode码点
            'x' => ascii_escape(chars),    // ascii字符
            _ => Err(format!("Unknown escape character '{}'", chr))
        };
        ReToken::new(ReTokenType::Literal, value?.to_string(), 0)
    };

    Ok(token)
}


///
/// 构建char class
///
fn build_char_class_token(tokens: &Vec<ReToken>) -> Result<ReToken, String> {
    if tokens.len() <= 1 {
       return Err("Empty Bucket".to_string());
    }

    let value: String = tokens.iter().skip(1).map(|x| x.value.clone()).collect();

    Ok(ReToken::new(ReTokenType::CharClass, value, tokens[0].pos))
}

///
/// 将多个token转换成一个class token，实现较为简单，严格要求括号闭合，所以如果真的要匹配括号用\[
///
fn char_class_token(regex: &str, tokens: Vec<ReToken>) -> ReResult<Vec<ReToken>> {
    let mut n_tokens = Vec::new();
    let mut counter = 0; // 防括号嵌套
    let mut char_class = false; // 开始匹配char_class

    let mut class_tokens = vec![]; // 临时变量，一个char class里的token


    for token in tokens {
        match token.typ {
            ReTokenType::LBracket => { char_class = true; counter += 1; },
            ReTokenType::RBracket => counter -= 1,
            _ => {}
        };

        match counter.cmp(&0) {
            Ordering::Less => {  // 这种情况 ] 括号多余 [
                return Err(ReError::new("Bucket Not Match".to_string(), token.pos, regex.into()))
            }
            Ordering::Greater => { class_tokens.push(token); } // 忽略第一个括号 [
            Ordering::Equal if char_class => { // 计数为0且开启char_class模式则构建该token
                char_class = false;
                let token = build_char_class_token(&class_tokens)
                    .map_err(|msg| ReError::new(msg, token.pos, regex.into()))?;
                class_tokens.clear();  // 清空
                n_tokens.push(token);  // 存入
            }
            _ => { n_tokens.push(token); }
        }
    }

    if counter != 0 {
        // counter不为0，class_tokens至少存了一个元素
        return Err(ReError::new("Bucket Not Close".to_string(), class_tokens[0].pos, regex.into()))
    }

    Ok(n_tokens)
}
