
/// 将正则表达式内的所有meta字符转义
pub fn escape_regex_meta(s: &str) -> String {
    // 正则元字符集合
    const META_CHARS: &str = r".^$*+?()[]{}|\\";

    let mut escaped = String::with_capacity(s.len());

    for c in s.chars() {
        if META_CHARS.contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }

    escaped
}