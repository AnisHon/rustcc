/*
 * C89 (ANSI C) grammar for Bison/Yacc — grammar + precedence only.
 *
 * Assumptions & notes:
 * 1) Input is already preprocessed (no #include/#define, trigraphs handled).
 * 2) The lexer must return TYPE_NAME for identifiers declared via typedef.
 * 3) No semantic actions here; plug in your own (attributes, symbol table, errors).
 * 4) No extensions (no long long, restrict, inline, // comments, C99 initializers, etc.).
 * 5) Old-style (K&R) parameter declarations are accepted.
 */

/* ====== Tokens ====== */
%token ID TYPE_NAME
%token INT FLOAT CHARACTER_CONSTANT STRING_LITERAL
%token KEYWORD_SIZEOF

/* multi-char operators */
%token OP_ARROW      /* '->' */
%token OP_INC      /* '++' */
%token OP_DEC      /* '--' */
%token OP_L_SHIFT     /* '<<' */
%token OP_R_SHIFT    /* '>>' */
%token OP_LE       /* '<=' */
%token OP_GE       /* '>=' */
%token OP_EQ       /* '==' */
%token OP_NE       /* '!=' */
%token OP_AND      /* '&&' */
%token OP_OR       /* '||' */
%token OP_MUL_ASSIGN  /* '*=' */
%token OP_DIV_ASSIGN  /* '/=' */
%token OP_MOD_ASSIGN  /* '%=' */
%token OP_ADD_ASSIGN  /* '+=' */
%token OP_SUB_ASSIGN  /* '-=' */
%token OP_L_SHIFT_ASSIGN /* '<<=' */
%token OP_R_SHIFT_ASSIGN/* '>>=' */
%token OP_AND_ASSIGN  /* '&=' */
%token OP_XOR_ASSIGN  /* '^=' */
%token OP_OR_ASSIGN   /* '|=' */

/* keywords */
%token KEYWORD_TYPEDEF KEYWORD_EXTERN KEYWORD_STATIC KEYWORD_AUTO KEYWORD_REGISTER
%token KEYWORD_CHAR KEYWORD_SHORT KEYWORD_INT KEYWORD_LONG KEYWORD_SIGNED KEYWORD_UNSIGNED KEYWORD_FLOAT KEYWORD_DOUBLE KEYWORD_VOID
%token KEYWORD_CONST KEYWORD_VOLATILE
%token KEYWORD_STRUCT KEYWORD_UNION KEYWORD_ENUM
%token KEYWORD_CASE KEYWORD_DEFAULT KEYWORD_IF KEYWORD_ELSE KEYWORD_SWITCH KEYWORD_WHILE KEYWORD_DO KEYWORD_FOR KEYWORD_GOTO KEYWORD_CONTINUE KEYWORD_BREAK KEYWORD_RETURN
%token OP_ELLIPSIS /* '...' */

/* ====== Precedence & associativity ======
   From lowest to highest precedence. */
%left COMMA
%right OP_ASSIGN OP_ADD_ASSIGN OP_SUB_ASSIGN OP_MUL_ASSIGN OP_DIV_ASSIGN OP_MOD_ASSIGN OP_L_SHIFT_ASSIGN OP_R_SHIFT_ASSIGN OP_AND_ASSIGN OP_XOR_ASSIGN OP_OR_ASSIGN
%right QUESTION COLON             /* conditional operator is right-associative */
%left OP_OR                /* || */
%left OP_AND               /* && */
%left OP_BITOR                  /* bitwise OR */
%left OP_XOR                  /* bitwise XOR */
%left OP_BITAND                  /* bitwise AND */
%left OP_EQ OP_NE          /* == != */
%left OP_LT OP_GT OP_LE OP_GE  /* < > <= >= */
%left OP_L_SHIFT OP_R_SHIFT     /* << >> */
%left OP_PLUS OP_MINUS
%left OP_TIMES OP_DIVIDE OP_MOD
%right OP_INC OP_DEC KEYWORD_SIZEOF

/* Dangling else resolution */
%nonassoc KEYWORD_ELSE

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
    : declaration_specifiers init_declarator_list_opt SEMICOLON
    ;

init_declarator_list_opt
    : /* empty */
    | init_declarator_list
    ;

init_declarator_list
    : init_declarator
    | init_declarator_list COMMA init_declarator
    ;

init_declarator
    : declarator
    | declarator OP_ASSIGN initializer
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
    : KEYWORD_TYPEDEF
    | KEYWORD_EXTERN
    | KEYWORD_STATIC
    | KEYWORD_AUTO
    | KEYWORD_REGISTER
    ;

type_specifier
    : KEYWORD_VOID
    | KEYWORD_CHAR
    | KEYWORD_SHORT
    | KEYWORD_INT
    | KEYWORD_LONG
    | KEYWORD_SIGNED
    | KEYWORD_UNSIGNED
    | KEYWORD_FLOAT
    | KEYWORD_DOUBLE
    | struct_or_union_specifier
    | enum_specifier
    | TYPE_NAME              /* resolved by lexer using typedef table */
    ;

type_qualifier
    : KEYWORD_CONST
    | KEYWORD_VOLATILE
    ;

struct_or_union_specifier
    : struct_or_union identifier_opt LBRACE struct_declaration_list RBRACE
    | struct_or_union LBRACE struct_declaration_list RBRACE
    | struct_or_union ID
    ;

struct_or_union
    : KEYWORD_STRUCT
    | KEYWORD_UNION
    ;

identifier_opt
    : /* empty */
    | ID
    ;

struct_declaration_list
    : struct_declaration
    | struct_declaration_list struct_declaration
    ;

struct_declaration
    : specifier_qualifier_list struct_declarator_list SEMICOLON
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
    | struct_declarator_list COMMA struct_declarator
    ;

struct_declarator
    : declarator
    | COLON constant_expression
    | declarator COLON constant_expression
    ;

enum_specifier
    : KEYWORD_ENUM identifier_opt LBRACE enumerator_list RBRACE
    | KEYWORD_ENUM ID
    ;

enumerator_list
    : enumerator
    | enumerator_list COMMA enumerator
    ;

enumerator
    : ID
    | ID OP_ASSIGN constant_expression
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
    : OP_TIMES
    | OP_TIMES type_qualifier_list
    | OP_TIMES pointer
    | OP_TIMES type_qualifier_list pointer
    ;

type_qualifier_list
    : type_qualifier
    | type_qualifier_list type_qualifier
    ;

direct_declarator
    : ID
    | LPAREN declarator RPAREN
    | direct_declarator LBRACKET constant_expression_opt RBRACKET
    | direct_declarator LPAREN parameter_type_list RPAREN
    | direct_declarator LPAREN identifier_list_opt RPAREN
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
    : ID
    | identifier_list COMMA ID
    ;

parameter_type_list
    : parameter_list
    | parameter_list COMMA OP_ELLIPSIS
    ;

parameter_list
    : parameter_declaration
    | parameter_list COMMA parameter_declaration
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
    : LPAREN abstract_declarator RPAREN
    | LBRACKET constant_expression_opt RBRACKET
    | direct_abstract_declarator LBRACKET constant_expression_opt RBRACKET
    | LPAREN parameter_type_list_opt RPAREN
    | direct_abstract_declarator LPAREN parameter_type_list_opt RPAREN
    ;

parameter_type_list_opt
    : /* empty */
    | parameter_type_list
    ;

/* Initializers (C89) */
initializer
    : assignment_expression
    | LBRACE initializer_list RBRACE
    | LBRACE initializer_list COMMA RBRACE  /* trailing comma is widely accepted; tighten if needed */
    ;

initializer_list
    : initializer
    | initializer_list COMMA initializer
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
    : ID COLON statement
    | KEYWORD_CASE constant_expression COLON statement
    | KEYWORD_DEFAULT COLON statement
    ;

compound_statement
    : LBRACE RBRACE
    | LBRACE block_item_list RBRACE
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
    : SEMICOLON
    | expression SEMICOLON
    ;

selection_statement
    : KEYWORD_IF LPAREN expression RPAREN statement %prec nonassoc
    | KEYWORD_IF LPAREN expression RPAREN statement KEYWORD_ELSE statement
    | KEYWORD_SWITCH LPAREN expression RPAREN statement
    ;

iteration_statement
    : KEYWORD_WHILE LPAREN expression RPAREN statement
    | KEYWORD_DO statement KEYWORD_WHILE LPAREN expression RPAREN SEMICOLON
    | KEYWORD_FOR LPAREN expression_opt SEMICOLON expression_opt SEMICOLON expression_opt RPAREN statement
    ;

expression_opt
    : /* empty */
    | expression
    ;

jump_statement
    : KEYWORD_GOTO ID SEMICOLON
    | KEYWORD_CONTINUE SEMICOLON
    | KEYWORD_BREAK SEMICOLON
    | KEYWORD_RETURN SEMICOLON
    | KEYWORD_RETURN expression SEMICOLON
    ;

/* 6.5 Expressions */
primary_expression
    : ID
    | constant
    | string
    | LPAREN expression RPAREN
    ;

constant
    : INT
    | FLOAT
    | CHARACTER_CONSTANT
    ;

/* adjacent string literal concatenation */
string
    : STRING_LITERAL
    | string STRING_LITERAL
    ;

postfix_expression
    : primary_expression
    | postfix_expression LBRACKET expression RBRACKET
    | postfix_expression LPAREN argument_expression_list_opt RPAREN
    | postfix_expression DOT ID
    | postfix_expression OP_ARROW ID
    | postfix_expression OP_INC
    | postfix_expression OP_DEC
    ;

argument_expression_list_opt
    : /* empty */
    | argument_expression_list
    ;

argument_expression_list
    : assignment_expression
    | argument_expression_list COMMA assignment_expression
    ;

unary_expression
    : postfix_expression
    | OP_INC unary_expression
    | OP_DEC unary_expression
    | unary_operator cast_expression
    | KEYWORD_SIZEOF unary_expression
    | KEYWORD_SIZEOF LPAREN type_name RPAREN
    ;

unary_operator
    : OP_BITAND
    | OP_TIMES
    | OP_PLUS %prec right
    | OP_MINUS %prec right
    | OP_BIT_NOT
    | OP_NOT
    ;

cast_expression
    : LPAREN type_name RPAREN cast_expression
    | unary_expression
    ;

multiplicative_expression
    : multiplicative_expression OP_TIMES cast_expression
    | multiplicative_expression OP_DIVIDE cast_expression
    | multiplicative_expression OP_MOD cast_expression
    | cast_expression
    ;

additive_expression
    : additive_expression OP_PLUS multiplicative_expression
    | additive_expression OP_MINUS multiplicative_expression
    | multiplicative_expression
    ;

shift_expression
    : shift_expression OP_L_SHIFT additive_expression
    | shift_expression OP_R_SHIFT additive_expression
    | additive_expression
    ;

relational_expression
    : relational_expression OP_LT shift_expression
    | relational_expression OP_GT shift_expression
    | relational_expression OP_LE shift_expression
    | relational_expression OP_GE shift_expression
    | shift_expression
    ;

equality_expression
    : equality_expression OP_EQ relational_expression
    | equality_expression OP_NE relational_expression
    | relational_expression
    ;

and_expression
    : and_expression OP_BITAND equality_expression
    | equality_expression
    ;

exclusive_or_expression
    : exclusive_or_expression OP_XOR and_expression
    | and_expression
    ;

inclusive_or_expression
    : inclusive_or_expression OP_BITOR exclusive_or_expression
    | exclusive_or_expression
    ;

logical_and_expression
    : logical_and_expression OP_AND inclusive_or_expression
    | inclusive_or_expression
    ;

logical_or_expression
    : logical_or_expression OP_OR logical_and_expression
    | logical_and_expression
    ;

conditional_expression
    : logical_or_expression
    | logical_or_expression QUESTION expression COLON conditional_expression
    ;

assignment_expression
    : conditional_expression
    | unary_expression assignment_operator assignment_expression
    ;

assignment_operator
    : OP_ASSIGN
    | OP_MUL_ASSIGN
    | OP_DIV_ASSIGN
    | OP_MOD_ASSIGN
    | OP_ADD_ASSIGN
    | OP_SUB_ASSIGN
    | OP_L_SHIFT_ASSIGN
    | OP_R_SHIFT_ASSIGN
    | OP_AND_ASSIGN
    | OP_XOR_ASSIGN
    | OP_OR_ASSIGN
    ;

expression
    : assignment_expression
    | expression COMMA assignment_expression
    ;

constant_expression
    : conditional_expression
    ;

/* 6.7.6 Type names (for casts/sizeof) */

type_name
    : specifier_qualifier_list abstract_declarator_opt
    ;

%%

