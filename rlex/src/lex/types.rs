use std::collections::BTreeMap;
use std::sync::OnceLock;

static STR_TO_ENUM: OnceLock<BTreeMap<char, ReTokenType>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReTokenType {
    Literal,   //
    Dot,       // '.'
    Caret,     // '^'
    Dollar,    //
    Star,      // *
    Plus,      // +
    Question,  // ?
    Pipe,      // |
    Backslash, // \

    LParen,
    RParen, // ()
    LBracket,
    RBracket, // []
    LBrace,
    RBrace, // {}

    Range,     // {1,2} {1,} {1}
    CharClass, // [c-f]字符类

    DigitClass,    // \d
    NonDigitClass, // \D
    WordClass,     // \w [a-zA-Z0-9_]
    NonWordClass,  // \W
    SpaceClass,    // \s
    NonSpaceClass, // \S
}

impl ReTokenType {
    pub fn from_char(chr: char) -> Option<ReTokenType> {
        // 只写接写在这里可维护和观感都差一点
        let table = STR_TO_ENUM.get_or_init(|| {
            [
                ('.', Self::Dot),
                ('^', Self::Caret),
                ('$', Self::Dollar),
                ('*', Self::Star),
                ('+', Self::Plus),
                ('?', Self::Question),
                ('|', Self::Pipe),
                ('\\', Self::Backslash),
                ('[', Self::LBracket),
                (']', Self::RBracket),
                ('{', Self::LBrace),
                ('}', Self::RBrace),
                ('(', Self::LParen),
                (')', Self::RParen),
                ('d', Self::DigitClass),
                ('D', Self::NonDigitClass),
                ('w', Self::WordClass),
                ('W', Self::NonWordClass),
                ('s', Self::SpaceClass),
                ('S', Self::NonSpaceClass),
            ]
            .iter()
            .cloned()
            .collect()
        });

        table.get(&chr).cloned()
    }
}
#[derive(Debug)]
pub struct ReToken {
    pub typ: ReTokenType,
    pub value: String,
    pub pos: usize,
}

impl ReToken {
    pub fn new(typ: ReTokenType, value: String, pos: usize) -> ReToken {
        ReToken { typ, value, pos }
    }

    /// 对于一些节点只有一个字符，该方法将返回第一个字符，如果不唯一则panic
    pub fn get_char(&self) -> char {
        let chr: Vec<char> = self.value.chars().collect();
        if chr.len() != 1 {
            panic!("oh no, something wrong...");
        };
        chr[0]
    }

    pub fn is_meta_char(chr: char) -> bool {
        // 不优雅但是先这样
        matches!(
            chr,
            '.' | '^' | '$' | '*' | '+' | '?' | '|' | '\\' | '[' | ']' | '{' | '}' | '(' | ')'
        )
    }

    pub fn is_class_char(chr: char) -> bool {
        // 不优雅但是先这样
        matches!(chr, 'd' | 'D' | 'w' | 'W' | 's' | 'S')
    }
}
