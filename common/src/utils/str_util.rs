use std::fmt::{Display};

/// 默认的四个空格的锁进
const IDENT_STR: &str = "    ";

/// 将origin推入dest，返回推入的字符数
fn push_count(dest: &mut String, origin: String) -> usize{
    let mut count = 0;
    for x in origin.chars() {
        count += 1;
        dest.push(x)
    }
    count
}


/// 将Option输出为rust可编译代码，最大宽度100
///
/// # Arguments
/// - `vec`: 列表
/// - `cvt`: 转换器，T将转换为代码字符串
///
/// # Returns
/// - `String`: 转换的代码
/// - `usize`: 原集合的大小
pub fn vec_to_code<T>(vec: impl Iterator<Item=T>, cvt: fn(T) -> String) -> (String, usize) {
    const MAX_WIDTH: usize = 100;
    let mut sz = 0;
    let mut code: String = String::new(); 
    let mut line_width = IDENT_STR.len(); // 带锁进，4个空格

    let options = vec.into_iter().map(|x| cvt(x));

    code.push_str("[\n");
    code.push_str(IDENT_STR);
    for code_patten in options.into_iter() {
        sz += 1;
        let count = push_count(&mut code, code_patten);
        code.push_str(", "); // 分割符
        line_width += count + 2;

        if line_width > MAX_WIDTH {
            code.push('\n');
            code.push_str(IDENT_STR);
            line_width = IDENT_STR.len();
        }
    }
    code.push_str("\n]");

    (code, sz)
}

/// Option 转换为 代码Some(..), None
pub fn option_cvt<T: Display>(value: Option<T>) -> String {
    match value {
        None => "None".to_string(),
        Some(x) => format!("Some({})", x),
    }
}

pub fn string_cvt(value: String) -> String {
    format!("\"{}\"", value)
}

/// 使用Display转换
pub fn default_cvt<T: Display>(value: T) -> String {
    format!("{}", value)
}


///
/// 按照条件输出换行符
/// # Returns
/// `cond ? "\n    " : ""`
pub fn new_line_ident(cond: bool) -> &'static str {
    match cond {
        true => "\n    ",
        false => ""
    }
}
