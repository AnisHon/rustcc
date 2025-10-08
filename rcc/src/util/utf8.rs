/// 解码一个 UTF-8 字符
/// 返回 Result<Option<char>, u8>
/// - Ok(Some(c)) -> 成功解码一个字符
/// - Ok(None) -> 输入为空（没有更多字节）
/// - Err(b) -> 非法 UTF-8，b 是起始字节
pub fn decode_char(bytes: &[u8]) -> Result<Option<(char, usize)>, u8> {
    if bytes.is_empty() {
        return Ok(None);
    }

    let b0 = bytes[0];

    // 根据首字节判断长度
    let width = match b0 {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => return Err(b0), // 非法起始字节
    };

    if bytes.len() < width {
        return Ok(None); // 不完整，需要更多字节
    }

    // 检查后续字节是否都是 10xxxxxx
    for &b in &bytes[1..width] {
        if b & 0b1100_0000 != 0b1000_0000 {
            return Err(b0);
        }
    }

    let slice = &bytes[..width];
    let ch = unsafe { std::str::from_utf8_unchecked(slice) }
        .chars()
        .next()
        .unwrap();

    Ok(Some((ch, width)))
}