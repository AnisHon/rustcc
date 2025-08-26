/*
 * 这是原来的文法文件
 * C89 (ANSI C) grammar for Bison/Yacc — grammar + precedence only.
 *
 * Assumptions & notes:
 * 1) Input is already preprocessed (no #include/#define, trigraphs handled).
 * 2) The lexer must return TYPE_NAME for identifiers declared via typedef.
 * 3) No semantic actions here; plug in your own (attributes, symbol table, errors).
 * 4) No extensions (no long long, restrict, inline, // comments, C99 initializers, etc.).
 * 5) Old-style (K&R) parameter declarations are accepted.
 */

%start translation_unit

%{
/* No C prologue: add your headers or yylex prototype if you need. */
%}

/* ====== Tokens ====== */
%token IDENTIFIER TYPE_NAME
%token I_CONSTANT F_CONSTANT CHAR_CONSTANT STRING_LITERAL
%token SIZEOF

/* multi-char operators */
%token PTR_OP      /* '->' */
%token INC_OP      /* '++' */
%token DEC_OP      /* '--' */
%token LEFT_OP     /* '<<' */
%token RIGHT_OP    /* '>>' */
%token LE_OP       /* '<=' */
%token GE_OP       /* '>=' */
%token EQ_OP       /* '==' */
%token NE_OP       /* '!=' */
%token AND_OP      /* '&&' */
%token OR_OP       /* '||' */
%token MUL_ASSIGN  /* '*=' */
%token DIV_ASSIGN  /* '/=' */
%token MOD_ASSIGN  /* '%=' */
%token ADD_ASSIGN  /* '+=' */
%token SUB_ASSIGN  /* '-=' */
%token LEFT_ASSIGN /* '<<=' */
%token RIGHT_ASSIGN/* '>>=' */
%token AND_ASSIGN  /* '&=' */
%token XOR_ASSIGN  /* '^=' */
%token OR_ASSIGN   /* '|=' */

/* keywords */
%token TYPEDEF EXTERN STATIC AUTO REGISTER
%token CHAR SHORT INT LONG SIGNED UNSIGNED FLOAT DOUBLE VOID
%token CONST VOLATILE
%token STRUCT UNION ENUM
%token CASE DEFAULT IF ELSE SWITCH WHILE DO FOR GOTO CONTINUE BREAK RETURN
%token ELLIPSIS /* '...' */

/* ====== Precedence & associativity ======
   From lowest to highest precedence. */
%left ','
%right '=' ADD_ASSIGN SUB_ASSIGN MUL_ASSIGN DIV_ASSIGN MOD_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN AND_ASSIGN XOR_ASSIGN OR_ASSIGN
%right '?' ':'             /* conditional operator is right-associative */
%left OR_OP                /* || */
%left AND_OP               /* && */
%left '|'                  /* bitwise OR */
%left '^'                  /* bitwise XOR */
%left '&'                  /* bitwise AND */
%left EQ_OP NE_OP          /* == != */
%left '<' '>' LE_OP GE_OP  /* < > <= >= */
%left LEFT_OP RIGHT_OP     /* << >> */
%left '+' '-'
%left '*' '/' '%'
%right UMINUS              /* for unary minus/plus via %prec */
%right INC_OP DEC_OP SIZEOF

/* Dangling else resolution */
%nonassoc LOWER_THAN_ELSE
%nonassoc ELSE

%%
/* ====== Grammar ====== */

/* 6.9 Translation unit */
translation_unit
    : external_declaration
    | translation_unit external_declaration
    ;

external_declaration
    : function_definition
    | declaration
    ;

/* 6.9.1 Function definition (C89 allows old-style parameter decls) */
function_definition
    : declaration_specifiers declarator declaration_list_opt compound_statement
    | declarator declaration_list_opt compound_statement
    ;

declaration_list_opt
    : /* empty */
    | declaration_list
    ;

declaration_list
    : declaration
    | declaration_list declaration
    ;

/* 6.7 Declarations */

declaration
    : declaration_specifiers init_declarator_list_opt ';'
    ;

init_declarator_list_opt
    : /* empty */
    | init_declarator_list
    ;

init_declarator_list
    : init_declarator
    | init_declarator_list ',' init_declarator
    ;

init_declarator
    : declarator
    | declarator '=' initializer
    ;

/* specifiers and qualifiers */

declaration_specifiers
    : storage_class_specifier declaration_specifiers_opt
    | type_specifier        declaration_specifiers_opt
    | type_qualifier        declaration_specifiers_opt
    ;

declaration_specifiers_opt
    : /* empty */
    | declaration_specifiers
    ;

storage_class_specifier
    : TYPEDEF
    | EXTERN
    | STATIC
    | AUTO
    | REGISTER
    ;

type_specifier
    : VOID
    | CHAR
    | SHORT
    | INT
    | LONG
    | SIGNED
    | UNSIGNED
    | FLOAT
    | DOUBLE
    | struct_or_union_specifier
    | enum_specifier
    | TYPE_NAME              /* resolved by lexer using typedef table */
    ;

type_qualifier
    : CONST
    | VOLATILE
    ;

struct_or_union_specifier
    : struct_or_union identifier_opt '{' struct_declaration_list '}'
    | struct_or_union '{' struct_declaration_list '}'
    | struct_or_union IDENTIFIER
    ;

struct_or_union
    : STRUCT
    | UNION
    ;

identifier_opt
    : /* empty */
    | IDENTIFIER
    ;

struct_declaration_list
    : struct_declaration
    | struct_declaration_list struct_declaration
    ;

struct_declaration
    : specifier_qualifier_list struct_declarator_list ';'
    ;

specifier_qualifier_list
    : type_specifier specifier_qualifier_list_opt
    | type_qualifier specifier_qualifier_list_opt
    ;

specifier_qualifier_list_opt
    : /* empty */
    | specifier_qualifier_list
    ;

struct_declarator_list
    : struct_declarator
    | struct_declarator_list ',' struct_declarator
    ;

struct_declarator
    : declarator
    | ':' constant_expression
    | declarator ':' constant_expression
    ;

enum_specifier
    : ENUM identifier_opt '{' enumerator_list '}'
    | ENUM IDENTIFIER
    ;

enumerator_list
    : enumerator
    | enumerator_list ',' enumerator
    ;

enumerator
    : IDENTIFIER
    | IDENTIFIER '=' constant_expression
    ;

/* declarators */

declarator
    : pointer_opt direct_declarator
    ;

pointer_opt
    : /* empty */
    | pointer
    ;

pointer
    : '*'
    | '*' type_qualifier_list
    | '*' pointer
    | '*' type_qualifier_list pointer
    ;

type_qualifier_list
    : type_qualifier
    | type_qualifier_list type_qualifier
    ;

direct_declarator
    : IDENTIFIER
    | '(' declarator ')'
    | direct_declarator '[' constant_expression_opt ']'
    | direct_declarator '(' parameter_type_list ')'
    | direct_declarator '(' identifier_list_opt ')'
    ;

constant_expression_opt
    : /* empty */
    | constant_expression
    ;

identifier_list_opt
    : /* empty */
    | identifier_list
    ;

identifier_list
    : IDENTIFIER
    | identifier_list ',' IDENTIFIER
    ;

parameter_type_list
    : parameter_list
    | parameter_list ',' ELLIPSIS
    ;

parameter_list
    : parameter_declaration
    | parameter_list ',' parameter_declaration
    ;

parameter_declaration
    : declaration_specifiers declarator
    | declaration_specifiers abstract_declarator_opt
    ;

abstract_declarator_opt
    : /* empty */
    | abstract_declarator
    ;

abstract_declarator
    : pointer
    | pointer_opt direct_abstract_declarator
    ;

direct_abstract_declarator
    : '(' abstract_declarator ')'
    | '[' constant_expression_opt ']'
    | direct_abstract_declarator '[' constant_expression_opt ']'
    | '(' parameter_type_list_opt ')'
    | direct_abstract_declarator '(' parameter_type_list_opt ')'
    ;

parameter_type_list_opt
    : /* empty */
    | parameter_type_list
    ;

/* Initializers (C89) */
initializer
    : assignment_expression
    | '{' initializer_list '}'
    | '{' initializer_list ',' '}'  /* trailing comma is widely accepted; tighten if needed */
    ;

initializer_list
    : initializer
    | initializer_list ',' initializer
    ;

/* 6.8 Statements */
statement
    : labeled_statement
    | compound_statement
    | expression_statement
    | selection_statement
    | iteration_statement
    | jump_statement
    ;

labeled_statement
    : IDENTIFIER ':' statement
    | CASE constant_expression ':' statement
    | DEFAULT ':' statement
    ;

compound_statement
    : '{' '}'
    | '{' block_item_list '}'
    ;

block_item_list
    : block_item
    | block_item_list block_item
    ;

block_item
    : declaration
    | statement
    ;

expression_statement
    : ';'
    | expression ';'
    ;

selection_statement
    : IF '(' expression ')' statement %prec LOWER_THAN_ELSE
    | IF '(' expression ')' statement ELSE statement
    | SWITCH '(' expression ')' statement
    ;

iteration_statement
    : WHILE '(' expression ')' statement
    | DO statement WHILE '(' expression ')' ';'
    | FOR '(' expression_opt ';' expression_opt ';' expression_opt ')' statement
    ;

expression_opt
    : /* empty */
    | expression
    ;

jump_statement
    : GOTO IDENTIFIER ';'
    | CONTINUE ';'
    | BREAK ';'
    | RETURN ';'
    | RETURN expression ';'
    ;

/* 6.5 Expressions */
primary_expression
    : IDENTIFIER
    | constant
    | string
    | '(' expression ')'
    ;

constant
    : I_CONSTANT
    | F_CONSTANT
    | CHAR_CONSTANT
    ;

/* adjacent string literal concatenation */
string
    : STRING_LITERAL
    | string STRING_LITERAL
    ;

postfix_expression
    : primary_expression
    | postfix_expression '[' expression ']'
    | postfix_expression '(' argument_expression_list_opt ')'
    | postfix_expression '.' IDENTIFIER
    | postfix_expression PTR_OP IDENTIFIER
    | postfix_expression INC_OP
    | postfix_expression DEC_OP
    ;

argument_expression_list_opt
    : /* empty */
    | argument_expression_list
    ;

argument_expression_list
    : assignment_expression
    | argument_expression_list ',' assignment_expression
    ;

unary_expression
    : postfix_expression
    | INC_OP unary_expression
    | DEC_OP unary_expression
    | unary_operator cast_expression
    | SIZEOF unary_expression
    | SIZEOF '(' type_name ')'
    ;

unary_operator
    : '&'
    | '*'
    | '+' %prec UMINUS
    | '-' %prec UMINUS
    | '~'
    | '!'
    ;

cast_expression
    : '(' type_name ')' cast_expression
    | unary_expression
    ;

multiplicative_expression
    : multiplicative_expression '*' cast_expression
    | multiplicative_expression '/' cast_expression
    | multiplicative_expression '%' cast_expression
    | cast_expression
    ;

additive_expression
    : additive_expression '+' multiplicative_expression
    | additive_expression '-' multiplicative_expression
    | multiplicative_expression
    ;

shift_expression
    : shift_expression LEFT_OP additive_expression
    | shift_expression RIGHT_OP additive_expression
    | additive_expression
    ;

relational_expression
    : relational_expression '<' shift_expression
    | relational_expression '>' shift_expression
    | relational_expression LE_OP shift_expression
    | relational_expression GE_OP shift_expression
    | shift_expression
    ;

equality_expression
    : equality_expression EQ_OP relational_expression
    | equality_expression NE_OP relational_expression
    | relational_expression
    ;

and_expression
    : and_expression '&' equality_expression
    | equality_expression
    ;

exclusive_or_expression
    : exclusive_or_expression '^' and_expression
    | and_expression
    ;

inclusive_or_expression
    : inclusive_or_expression '|' exclusive_or_expression
    | exclusive_or_expression
    ;

logical_and_expression
    : logical_and_expression AND_OP inclusive_or_expression
    | inclusive_or_expression
    ;

logical_or_expression
    : logical_or_expression OR_OP logical_and_expression
    | logical_and_expression
    ;

conditional_expression
    : logical_or_expression
    | logical_or_expression '?' expression ':' conditional_expression
    ;

assignment_expression
    : conditional_expression
    | unary_expression assignment_operator assignment_expression
    ;

assignment_operator
    : '='
    | MUL_ASSIGN
    | DIV_ASSIGN
    | MOD_ASSIGN
    | ADD_ASSIGN
    | SUB_ASSIGN
    | LEFT_ASSIGN
    | RIGHT_ASSIGN
    | AND_ASSIGN
    | XOR_ASSIGN
    | OR_ASSIGN
    ;

expression
    : assignment_expression
    | expression ',' assignment_expression
    ;

constant_expression
    : conditional_expression
    ;

/* 6.7.6 Type names (for casts/sizeof) */

type_name
    : specifier_qualifier_list abstract_declarator_opt
    ;

%%
/*
 * Lexer expectations (suggested):
 *  - return single-character tokens for: ()[]{};,:?~!+-*/%&^|<>.=
 *  - return multi-char tokens listed above for compound ops/assignments.
 *  - map all typedef names to TYPE_NAME using your symbol table.
 *  - handle string literal concatenation by returning STRING_LITERAL per segment; parser folds them.
 *
 * Typical conflicts: after adding the IF/ELSE precedence hack, this should report 0 S/R conflicts
 * under a standard setup where TYPE_NAME is provided by the lexer.
 */
