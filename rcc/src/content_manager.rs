use std::ops::Range;
use crate::util::utf8::decode_char;

pub struct ContentManager {
    content: Vec<u8>
}

impl ContentManager {
    pub fn new(content: Vec<u8>) -> ContentManager {
        Self {
            content
        }
    }

    /// [beg, end)
    pub fn str(&self, range: Range<usize>) -> &str {
        // 直接上unsafe
        unsafe { str::from_utf8_unchecked(&self.content[range]) }
    }

    pub fn decode(&self, idx: usize) -> Option<(char, usize)> {
        match decode_char(&self.content[idx..]) {
            Ok(x) => x,
            Err(_) => panic!("Not Valid UTF8 File")
        }
    }
}

