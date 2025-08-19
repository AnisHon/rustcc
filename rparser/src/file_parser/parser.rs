use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser_pest.pest"]
struct BisonParser;

#[derive(Debug)]
pub enum AssocType {
    Left(Vec<String>),
    Right(Vec<String>),
    NonAssoc(Vec<String>),
}


#[derive(Debug)]
pub struct Production {
    name: String,
    rules: Vec<(Vec<String>, Option<String>)>,
}

pub struct GrammarParser {
    input: String,
    tokens: Vec<String>,
    assoc: Vec<AssocType>,
    productions: Vec<Production>
}

impl GrammarParser {
    pub fn new(input: String) -> Self {
        Self { input, tokens: Vec::new(), assoc: Vec::new(), productions: Vec::new() }
    }

    fn get_ast(mut self) {
        let input = self.input.clone();
        let pairs = BisonParser::parse(Rule::file, input.as_str()).unwrap();

        for pair in pairs {
            self.parse_file(pair);
        }

        println!("{:?}", self.tokens);
        println!("{:?}", self.assoc);
        println!("{:?}", self.productions);
    }

    fn parse_file(&mut self, file: Pair<Rule>) {
        for rule in file.into_inner() {
            match rule.as_rule() {
                Rule::decls => self.parse_decls(rule),
                Rule::rules => self.parse_rules(rule),
                Rule::user_code => self.parse_user_code(rule),
                _ => unreachable!()
            }
        }
    }
    /// 解析 decl
    fn parse_decls(&mut self, decls: Pair<Rule>) {
        for decl in decls.into_inner() {
            for pair in decl.into_inner() {
                match pair.as_rule() {
                    Rule::token_decl => self.parse_token_decl(pair),
                    Rule::assoc_decl => self.parse_assoc_decl(pair),
                    _ => unreachable!(),
                }
            }
        }

    }

    fn parse_token_decl(&mut self, decl: Pair<Rule>) {
        self.tokens.extend(decl.into_inner().into_iter().map(|x| x.as_str().to_string()));
    }
    fn parse_assoc_decl(&mut self, decl: Pair<Rule>) {
        let assoc_type = &decl.as_span().as_str().split_whitespace().next().unwrap()[1..];
        let assoc_values: Vec<String> = decl.into_inner().map(|x| x.as_span().as_str().to_string()).collect();
        let assoc = match assoc_type {
            "left" => AssocType::Left(assoc_values),
            "right" => AssocType::Right(assoc_values),
            "nonassoc" => AssocType::NonAssoc(assoc_values),
            _ => unreachable!()
        };
        self.assoc.push(assoc);
    }

    fn parse_rules(&mut self, rules: Pair<Rule>) {
        for rule_decl in rules.into_inner() {
            let production = Self::parse_rule_decl(rule_decl);
            self.productions.push(production);
        }
    }

    fn parse_rule_decl(rule_decl: Pair<Rule>) -> Production {
        let mut pairs = rule_decl.into_inner();
        // pairs.for_each(|pair| println!("{:?}", pair));
        let mut production = Production {
            name: pairs.next().unwrap().as_str().to_string(),
            rules: Vec::new(),
        };

        for pair in pairs.into_iter() {
            let mut symbols = Vec::new();
            let mut action = None;
            for item in pair.into_inner() {
                match item.as_rule() {
                    Rule::symbol => symbols.push(item.as_str().to_string()),
                    Rule::action => action = Some(item.as_str().to_string()),
                    _ => unreachable!()
                }
            }
            production.rules.push((symbols, action));
        }

        production
    }

    fn parse_user_code(&mut self, rules: Pair<Rule>) {

    }

}










#[test]
fn test() {
    let input = r#"%token NUMBER
%token PLUS MINUS
%left PLUS MINUS
%right PLUS MINUS
%nonassoc PLUS MINUS

%%
expr:
    expr PLUS term { $$ = $1 + $3; }
  | expr MINUS term { $$ = $1 - $3; }
  | term
  ;

term:
    NUMBER
  ;
%%
/* user C code */
int main() { return yyparse(); }
"#;
    let x = GrammarParser::new(input.to_owned()).get_ast();
}
