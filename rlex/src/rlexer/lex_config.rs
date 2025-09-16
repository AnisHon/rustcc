use pest::Parser;
use pest_derive::Parser;

// 词法分析器 结构
#[derive(Debug)]
pub struct LexStruct {
    pub name: String,
    pub regex: String,
    pub skip: bool,
}

#[derive(Parser)]
#[grammar = "lexer.pest"]
pub struct LexGrammar;

pub struct LexConfigParser {
    input: String,
}


impl LexConfigParser {
    pub fn new(input: String) -> LexConfigParser {
        LexConfigParser { input }
    }

    pub fn parse(self) {
        let file = LexGrammar::parse(Rule::l_file, self.input.as_str())
            .unwrap()
            .next()
            .unwrap();
        println!("{:?}", file);
    }

}

#[test]
pub fn t() {
    let input = r#"
%option noyywrap
// 注释
[a-z]+ { println!("identifier"); }
[0-9]+ { println!("number"); }
"#;

    LexConfigParser::new(input.to_string()).parse();
}
