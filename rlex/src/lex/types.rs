use std::collections::HashMap;
use std::sync::OnceLock;

static STR_TO_ENUM: OnceLock<HashMap<char, ReTokenType>> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub enum ReTokenType {
    Literal,    //
    Dot,        // '.'
    Caret,      // '^'
    Dollar,     //
    Star,       // *
    Plus,       // +
    Question,   // ?
    Pipe,       // |
    Backslash,  // \

    LParen, RParen, // ()
    LBracket, RBracket, // []
    LBrace, RBrace, // {}

    CharClass,      // [c-f]字符类
    
    DigitClass,     // \d
    NonDigitClass,  // \D
    WordClass,      // \w [a-zA-Z0-9_]
    NonWordClass,   // \W
    SpaceClass,     // \s
    NonSpaceClass,  // \S
}

impl ReTokenType {

    pub fn from_char(chr: char) -> Option<ReTokenType> {
        // 只写接写在这里可维护和观感都差一点
        let table = STR_TO_ENUM.get_or_init(|| {
            [
                ('.', Self::Dot), ('^', Self::Caret), ('$', Self::Dollar), ('*', Self::Star),
                ('+', Self::Plus), ('?', Self::Question), ('|', Self::Pipe), ('\\', Self::Backslash),
                ('[', Self::LBracket), (']', Self::RBracket), ('{', Self::LBrace), ('}', Self::RBrace), 
                ('(', Self::LParen), (')', Self::RParen),
                ('d', Self::DigitClass), ('D', Self::NonDigitClass), ('w', Self::WordClass),
                ('W', Self::NonWordClass), ('s', Self::SpaceClass), ('S', Self::NonSpaceClass),
            ].iter().cloned().collect()
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

    pub fn is_meta_char(chr: char) -> bool {
        // 不优雅但是先这样
        matches!(chr, '.' | '^' | '$' | '*' | '+' | '?' | '|' | '\\' | '[' | ']' | '{' | '}' | '(' | ')')
    }

    pub fn is_class_char(chr: char) -> bool {
        // 不优雅但是先这样
        matches!(chr, 'd' | 'D' | 'w' | 'W' | 's' | 'S')
    }

}
