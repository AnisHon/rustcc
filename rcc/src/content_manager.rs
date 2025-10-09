use std::ops::Range;
use std::str::Chars;
use crate::util::utf8::decode_char;

pub struct ContentManager {
    content: String
}

impl ContentManager {
    pub fn new(content: String) -> ContentManager {
        Self {
            content
        }
    }

    /// [beg, end)
    pub fn str(&self, range: Range<usize>) -> &str {
        &self.content[range]
    }

    pub fn chars(&self) -> Chars {
        self.content.chars()
    }
    
}

