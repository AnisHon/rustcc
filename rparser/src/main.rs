use rparser::file_parser::reader::{get_grammar, GrammarParser};
use rparser::lr::lr0::LR0Builder;

fn main() {

    let input = r#"%token NUMBER
%token PLUS MINUS
%left  MINUS
%right PLUS

%%
expr:
    expr PLUS term { $$ = $1 + $3; }
  | expr MINUS term { $$ = $1 - $3; }
  | term
  |
  ;

term:
    NUMBER
  ;
%%
/* user C code */
int main() { return yyparse(); }
"#;
    let grammar = GrammarParser::new(input.to_owned()).parse();

    let x = LR0Builder::new(&grammar).build_table();

    println!("{:?}", x);


    // println!("{:#?}", grammar);
    // println!("{:?}", x);

}