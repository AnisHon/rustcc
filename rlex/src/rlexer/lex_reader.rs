use crate::rlexer::lex_config::LexStruct;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

pub struct LexReader {
    buff: BufReader<File>,
}

impl LexReader {
    pub fn new(path: &str) -> LexReader {
        let file = match File::open(path) {
            Ok(x) => x,
            Err(err) => panic!("{}", err),
        };

        let buff = BufReader::new(file);

        LexReader { buff }
    }

    pub fn read_from_file(self) -> io::Result<Vec<LexStruct>> {
        let mut lex: Vec<LexStruct> = Vec::new();

        for line in self.buff.lines() {
            let line = line?;
            let vec: Vec<_> = line.split_whitespace().collect();

            if vec.is_empty() {
                continue;
            }
            let name: String = vec[0].to_string();
            let mut regex: String = vec[1].to_string();

            if regex.starts_with('"') && regex.ends_with('"') {
                regex = escape_regex_meta(&regex[1..regex.len() - 1]);
            }

            lex.push(LexStruct { name, regex });
        }

        Ok(lex)
    }
}
fn escape_regex_meta(s: &str) -> String {
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
