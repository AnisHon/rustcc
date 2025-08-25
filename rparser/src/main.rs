use rparser::file_parser::reader::{get_grammar, GrammarConfigParser};
use rparser::file_parser::table_builder::{LRTableBuilder, TableType};
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
    // let grammar = GrammarConfigParser::new(input.to_owned()).parse();
    let builder = LRTableBuilder::new(TableType::LALR1, input.to_string());
    let (action, goto, init) = builder.build_lr_table();

    for item in action {
        for x in item {
            print!("{:?} ", x);
        }
        println!();
    }

    
    
    

    // let x = LR0Builder::new(&grammar).build_table();

    // println!("{:?}", x);


    // println!("{:#?}", grammar);
    // println!("{:?}", x);

}