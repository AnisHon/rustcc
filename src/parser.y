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

%{
use crate::types::ast::ast_nodes::*;
use crate::types::ast::decl_info::*;
use crate::types::ast::parser_node::*;
use crate::types::ast::sema::*;
use crate::types::ast::temp::*;
use crate::types::ast::struct_union_info::*;
use crate::types::ast::initializer::*;
use crate::types::ast::func_info::*
%}

%type ParserNode

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
%left ','
%right '=' OP_ADD_ASSIGN OP_SUB_ASSIGN OP_MUL_ASSIGN OP_DIV_ASSIGN OP_MOD_ASSIGN OP_L_SHIFT_ASSIGN OP_R_SHIFT_ASSIGN OP_AND_ASSIGN OP_XOR_ASSIGN OP_OR_ASSIGN
%right '?' ':'             /* conditional operator is right-associative */
%left OP_OR                /* || */
%left OP_AND               /* && */
%left '|'                  /* bitwise OR */
%left '^'                  /* bitwise XOR */
%left '&'                  /* bitwise AND */
%left OP_EQ OP_NE          /* == != */
%left '<' '>' OP_LE OP_GE  /* < > <= >= */
%left OP_L_SHIFT OP_R_SHIFT     /* << >> */
%left '+' '-'
%left '*' '/' '%'
%right OP_INC OP_DEC KEYWORD_SIZEOF

/* Dangling else resolution */
%nonassoc KEYWORD_ELSE

%%
/* ====== Grammar ====== */

/* 6.9 Translation unit */
translation_unit
    : external_declaration                  {$$ = TranslationUnit::make_translation_unit($1);}
    | translation_unit external_declaration {$$ = TranslationUnit::insert_ext_decl($1, $2);}
    ;

external_declaration
    : function_definition   {$$ = ExternalDeclaration::make_func($1);}
    | declaration           {$$ = ExternalDeclaration::make_variable($1);}
    ;

/* 6.9.1 Function definition (C89 allows old-style parameter decls) */
function_definition
    : declaration_specifiers declarator declaration_list_opt compound_statement     { $$ = FunctionDefinition::make(Some($1), $2, $3, $4); }
    | declarator declaration_list_opt compound_statement                            { $$ = FunctionDefinition::make(None, $1, $2, $3); }
    ;

declaration_list_opt
    : /* empty */       { $$ = ParserNode::None; }
    | declaration_list  { $$ = $1; }
    ;

declaration_list
    : declaration                   { $$ = Decl::make_list($1); }
    | declaration_list declaration  { $$ = Decl::push($1, $2); }
    ;

/* 6.7 Declarations */

declaration
    : declaration_specifiers init_declarator_list_opt ';' { $$ = Decl::make($1, $2, $3) }
    ;

init_declarator_list_opt
    : /* empty */           {$$ = ParserNode::None;;}
    | init_declarator_list  {$$ = $1;}
    ;

init_declarator_list
    : init_declarator                               { $$ = InitDeclarator::make_list($1); }
    | init_declarator_list ',' init_declarator      { $$ = InitDeclarator::push($1, $2, $3); }
    ;

init_declarator
    : declarator                    { $$ = InitDeclarator::make($1, None, None); }
    | declarator '=' initializer    { $$ = InitDeclarator::($1, Some($2), Some($3)); }
    ;

/* specifiers and qualifiers */

declaration_specifiers
    : storage_class_specifier declaration_specifiers_opt    {$$ = DeclSpec::push_storage($1, $2);}
    | type_specifier        declaration_specifiers_opt      {$$ = DeclSpec::push_spec($1, $2);}
    | type_qualifier        declaration_specifiers_opt      {$$ = DeclSpec::push_qual($1, $2);}
    ;

declaration_specifiers_opt
    : /* empty */               {$$ = ParserNode::None;}
    | declaration_specifiers    {$$ = $1;}
    ;

storage_class_specifier
    : KEYWORD_TYPEDEF   {$$ = $1;}
    | KEYWORD_EXTERN    {$$ = $1;}
    | KEYWORD_STATIC    {$$ = $1;}
    | KEYWORD_AUTO      {$$ = $1;}
    | KEYWORD_REGISTER  {$$ = $1;}
    ;

type_specifier
    : KEYWORD_VOID              {$$ = TypeSpec::make($1);}
    | KEYWORD_CHAR              {$$ = TypeSpec::make($1);}
    | KEYWORD_SHORT             {$$ = TypeSpec::make($1);}
    | KEYWORD_INT               {$$ = TypeSpec::make($1);}
    | KEYWORD_LONG              {$$ = TypeSpec::make($1);}
    | KEYWORD_SIGNED            {$$ = TypeSpec::make($1);}
    | KEYWORD_UNSIGNED          {$$ = TypeSpec::make($1);}
    | KEYWORD_FLOAT             {$$ = TypeSpec::make($1);}
    | KEYWORD_DOUBLE            {$$ = TypeSpec::make($1);}
    | struct_or_union_specifier {$$ = TypeSpec::make_struct_or_union($1);}
    | enum_specifier            {$$ = TypeSpec::make_enum($1);}
    | TYPE_NAME                 {$$ = TypeSpec::make($1);}      /* resolved by lexer using typedef table */
    ;

type_qualifier
    : KEYWORD_CONST     {$$ = $1;}
    | KEYWORD_VOLATILE  {$$ = $1;}
    ;

struct_or_union_specifier
    : struct_or_union identifier_opt '{' struct_declaration_list '}'  {$$ = StructUnionSpec::make_def($1, $2, $3, $4, $5);}
    | struct_or_union ID                                              {$$ = StructUnionSpec::make_decl($1, $2);}
    ;

struct_or_union
    : KEYWORD_STRUCT    {$$ = $1;}
    | KEYWORD_UNION     {$$ = $1;}
    ;

identifier_opt
    : /* empty */   {$$ = ParserNode::None;}
    | ID            {$$ = $1;}
    ;

struct_declaration_list
    : struct_declaration                            {$$ = StructDecl::make_list(None, $1);}
    | struct_declaration_list struct_declaration    {$$ = StructDecl::push(Some($1), $2);}
    ;

struct_declaration
    : specifier_qualifier_list struct_declarator_list ';' { $$ = StructDecl::make($1, $2, $3); }
    ;

specifier_qualifier_list
    : type_specifier specifier_qualifier_list_opt   { $$ = DeclSpec::push_spec($1, $2); }
    | type_qualifier specifier_qualifier_list_opt   { $$ = DeclSpec::push_qual($1, $2); }
    ;

specifier_qualifier_list_opt
    : /* empty */               {$$ = ParserNode::None;}
    | specifier_qualifier_list  {$$ = $1;}
    ;

struct_declarator_list
    : struct_declarator                               { $$ = StructDeclarator::make_list($1); }
    | struct_declarator_list ',' struct_declarator    { $$ = StructDeclarator::push($1, $2, $3); }
    ;

struct_declarator
    : declarator                          { $$ = StructDeclarator::make(Some($1), None, None); }
    | ':' constant_expression             { $$ = StructDeclarator::make(None, Some($1), Some($2)); }
    | declarator ':' constant_expression  { $$ = StructDeclarator::make(Some($1), Some($2), Some($3)); }
    ;

enum_specifier
    : KEYWORD_ENUM identifier_opt '{' enumerator_list '}' {$$ = EnumSpec::make_anon($1, $2, $3, $4, $5);}
    | KEYWORD_ENUM ID                                     {$$ = EnumSpec::make_named($1, $2);}
    ;

enumerator_list
    : enumerator                      {$$ = Enumerator::make_list($1);}
    | enumerator_list ',' enumerator  {$$ = Enumerator::push($1, $2, $3);}
    ;

enumerator
    : ID                          {$$ = Enumerator::make($1, None);}
    | ID '=' constant_expression  {$$ = Enumerator::make($1, Some($3));}
    ;

/* declarators */

declarator
    : pointer_opt direct_declarator {$$ = Declarator::make($1, $2);}
    ;

pointer_opt
    : /* empty */   {$$ = ParserNode::None;}
    | pointer       {$$ = $1;}
    ;

pointer
    : '*'                              {$$ = PointerChunk::make_list( PointerChunk::make_pointer($1, None) );}
    | '*' type_qualifier_list          {$$ = PointerChunk::make_list( PointerChunk::make_pointer($1, $2) );}
    | '*' pointer                      {$$ = PointerChunk::push_front( PointerChunk::make_pointer($1, None), $2 );}
    | '*' type_qualifier_list pointer  {$$ = PointerChunk::push_front( PointerChunk::make_pointer($1, $2), $3 );}
    ;

type_qualifier_list
    : type_qualifier                        {$$ = TypeQual::make(None, $1);}
    | type_qualifier_list type_qualifier    {$$ = TypeQual::make(Some($1), $2);}
    ;

/* 最后的那个是老式声明 */
direct_declarator
    : ID                                                { $$ = DeclChunk::make_list( DeclChunk::make_ident($1) ); }
    | '(' declarator ')'                                { $$ = DeclChunk::make_list( DeclChunk::make_paren($1, $2, $3) ); }
    | direct_declarator '[' constant_expression_opt ']' { $$ = DeclChunk::push( $1, DeclChunk::make_array($2, $3, $4) ); }
    | direct_declarator '(' parameter_type_list ')'     { $$ = DeclChunk::push( $1, DeclChunk::make_function($2, $3, $4) ); }
    | direct_declarator '(' identifier_list_opt ')'     { $$ = DeclChunk::push( $1, DeclChunk::make_old_function($2, $3, $4) ); }
    ;

constant_expression_opt
    : /* empty */           {$$ = ParserNode::None;}
    | constant_expression   {$$ = $1;}
    ;

identifier_list_opt
    : /* empty */       {$$ = ParserNode::None;}
    | identifier_list   {$$ = $1;}
    ;

identifier_list
    : ID                        {$$ = make_ident_list($1);}
    | identifier_list ',' ID    {$$ = push_ident_list($1, $2, $3);}
    ;

parameter_type_list
    : parameter_list                    {$$ = $1;}
    | parameter_list ',' OP_ELLIPSIS    {$$ = ParamList::set_variadic($1, $2, $3);}
    ;

parameter_list
    : parameter_declaration                     { $$ = ParamDecl::make_list($1); }
    | parameter_list ',' parameter_declaration  { $$ = ParamDecl::push($1, $2, $3); }
    ;

parameter_declaration
    : declaration_specifiers declarator                 { $$ = ParamDecl::make($1, Some($2). true); }
    | declaration_specifiers abstract_declarator_opt    { $$ = ParamDecl::make($1, $2, false); }
    ;

abstract_declarator_opt
    : /* empty */           {$$ = ParserNode::None;}
    | abstract_declarator   {$$ = $1;}
    ;

abstract_declarator
    : pointer                                   {$$ = Declarator::make_pointer(Some($1), Some($2));}
    | pointer_opt direct_abstract_declarator    {$$ = Declarator::make_pointer($1, Some($2));}
    ;

direct_abstract_declarator
    : '(' abstract_declarator ')'                                 {$$ = Declarator::add_span($1, $2, $3);}
    | '[' constant_expression_opt ']'                             {$$ = Declarator::make_array(None, $1, $2, $3);}
    | direct_abstract_declarator '[' constant_expression_opt ']'  {$$ = Declarator::make_array(Some($1), $2, $3, $4);}
    | '(' parameter_type_list_opt ')'                             {$$ = Declarator::make_function(None, $1, $2, $3);}
    | direct_abstract_declarator '(' parameter_type_list_opt ')'  {$$ = Declarator::make_function(Some($1), $2, $3, $4);}
    ;

parameter_type_list_opt
    : /* empty */           {$$ = ParserNode::None;}
    | parameter_type_list   {$$ = $1;}
    ;

/* Initializers (C89) */
initializer
    : assignment_expression         { $$ = InitInfo::make_expr($1); }
    | '{' initializer_list '}'      { $$ = InitInfo::make_init_list($1, $2, None, $3) }
    | '{' initializer_list ',' '}'  { $$ = InitInfo::make_init_list($1, $2, Some($3), $4) }  /* trailing comma is widely accepted; tighten if needed */
    ;

initializer_list
    : initializer                       { $$ = InitInfo::make_list($1); }
    | initializer_list ',' initializer  { $$ = InitInfo::push($1, $2, $3); }
    ;

/* 6.8 Statements */
statement
    : labeled_statement     {$$ = Statement::make_labeled($1);}
    | compound_statement    {$$ = Statement::make_compound($1);}
    | expression_statement  {$$ = Statement::make_expression($1);}
    | selection_statement   {$$ = Statement::make_selection($1);}
    | iteration_statement   {$$ = Statement::make_iteration($1);}
    | jump_statement        {$$ = Statement::make_jump($1);}
    ;

labeled_statement
    : ID ':' statement                                {$$ = LabeledStatement::make_label($1, $3);}
    | KEYWORD_CASE constant_expression ':' statement  {$$ = LabeledStatement::make_case($2, $4);}
    | KEYWORD_DEFAULT ':' statement                   {$$ = LabeledStatement::make_default($3);}
    ;

compound_statement
    : '{' '}'                 { $$ = CompoundStatement::make($1, None, $2); }
    | '{' block_item_list '}' { $$ = CompoundStatement::make($1, Some($2), $3); }
    ;

block_item_list
    : block_item                    { $$ = BlockItem::make($1); }
    | block_item_list block_item    { $$ = BlockItem::push($1, $2); }
    ;

block_item
    : declaration   { $$ = BlockItem::make_decl($1); }
    | statement     { $$ = BlockItem::make_stmt($1); }
    ;

expression_statement
    : ';'             {$$ = Statement::make_expression(None);}
    | expression ';'  {$$ = Statement::make_expression(Some($1));}
    ;

selection_statement
    : KEYWORD_IF '(' expression ')' statement                         {$$ = Statement::make_if($1, $3, $5, None);} %prec nonassoc
    | KEYWORD_IF '(' expression ')' statement KEYWORD_ELSE statement  {$$ = Statement::make_if($1, $3, $5, Some($7));}
    | KEYWORD_SWITCH '(' expression ')' statement                     {$$ = Statement::make_switch($1, $3, $5);}
    ;

iteration_statement
    : KEYWORD_WHILE '(' expression ')' statement                                          {$$ = Statement::make_while($1, $3, $5, None);}
    | KEYWORD_DO statement KEYWORD_WHILE '(' expression ')' ';'                           {$$ = Statement::make_while($1, $2, $5, Some($6));}
    | KEYWORD_FOR '(' expression_opt ';' expression_opt ';' expression_opt ')' statement  {$$ = Statement::make_for($1, $3, $5, $7, $9);}
    ;

expression_opt
    : /* empty */   {$$ = ParserNode::None;}
    | expression    {$$ = $1;}
    ;

jump_statement
    : KEYWORD_GOTO ID ';'             {$$ = Statement::make_goto($1, $2);}
    | KEYWORD_CONTINUE ';'            {$$ = Statement::make_continue_break($1);}
    | KEYWORD_BREAK ';'               {$$ = Statement::make_continue_break($1);}
    | KEYWORD_RETURN ';'              {$$ = Statement::make_return($1, None);}
    | KEYWORD_RETURN expression ';'   {$$ = Statement::make_return($1, $2);}
    ;

/* 6.5 Expressions */
primary_expression
    : ID                  {$$ = Expression::make_id($1);}
    | constant            {$$ = Expression::make_literal($1);}
    | string              {$$ = Expression::make_literal($1);}
    | '(' expression ')'  {$$ = $2;}
    ;

constant
    : INT                   {$$ = Constant::make($1);}
    | FLOAT                 {$$ = Constant::make($1);}
    | CHARACTER_CONSTANT    {$$ = Constant::make($1);}
    ;

/* adjacent string literal concatenation */
string
    : STRING_LITERAL        {$$ = Constant::make($1);}
    | string STRING_LITERAL {$$ = Constant::insert_str($1, $2);}
    ;

postfix_expression
    : primary_expression                                        {$$ = $1;}
    | postfix_expression '[' expression ']'                     {$$ = Expression::make_array_access($1, $3, $4);}
    | postfix_expression '(' argument_expression_list_opt ')'   {$$ = Expression::make_call($1, $3, $4);}
    | postfix_expression '.' ID                                 {$$ = Expression::make_field($1, $3);}
    | postfix_expression OP_ARROW ID                            {$$ = Expression::make_arrow($1, $3);}
    | postfix_expression OP_INC                                 {$$ = Expression::make_update($1, $2, true);}
    | postfix_expression OP_DEC                                 {$$ = Expression::make_update($1, $2, true);}
    ;

argument_expression_list_opt
    : /* empty */               { $$ = ParserNode::None; }
    | argument_expression_list  { $$ = $1; }
    ;

argument_expression_list
    : assignment_expression                                 { $$ = $1; }
    | argument_expression_list ',' assignment_expression    { $$ = Expression::make_assign($1, $2, $3); }
    ;

unary_expression
    : postfix_expression                {$$ = $1;}
    | OP_INC unary_expression           {$$ = Expression::make_update($2, $1, false);}
    | OP_DEC unary_expression           {$$ = Expression::make_update($2, $1, true);}
    | unary_operator cast_expression    {$$ = Expression::make_unary($1, $2);}
    | KEYWORD_SIZEOF unary_expression   {$$ = Expression::make_sizeof_expr($1, $2);}
    | KEYWORD_SIZEOF '(' type_name ')'  {$$ = Expression::make_sizeof_type($1, $3, $4);}
    ;

unary_operator
    : '&'   {$$ = $1;}
    | '*'   {$$ = $1;}
    | '+'   {$$ = $1;}      %prec right
    | '-'   {$$ = $1;}      %prec right
    | '~'   {$$ = $1;}
    | '!'   {$$ = $1;}
    ;

cast_expression
    : '(' type_name ')' cast_expression {$$ = Expression::make_cast($1, $2, $4);}
    | unary_expression                  {$$ = $1;}
    ;

multiplicative_expression
    : multiplicative_expression '*' cast_expression     {$$ = Expression::make_binary($1, $2, $3);}
    | multiplicative_expression '/' cast_expression     {$$ = Expression::make_binary($1, $2, $3);}
    | multiplicative_expression '%' cast_expression     {$$ = Expression::make_binary($1, $2, $3);}
    | cast_expression                                   {$$ = $1;}
    ;

additive_expression
    : additive_expression '+' multiplicative_expression     {$$ = Expression::make_binary($1, $2, $3);}
    | additive_expression '-' multiplicative_expression     {$$ = Expression::make_binary($1, $2, $3);}
    | multiplicative_expression                             {$$ = $1;}
    ;

shift_expression
    : shift_expression OP_L_SHIFT additive_expression       {$$ = Expression::make_binary($1, $2, $3);}
    | shift_expression OP_R_SHIFT additive_expression       {$$ = Expression::make_binary($1, $2, $3);}
    | additive_expression                                   {$$ = $1;}
    ;

relational_expression
    : relational_expression '<' shift_expression            {$$ = Expression::make_binary($1, $2, $3);}
    | relational_expression '>' shift_expression            {$$ = Expression::make_binary($1, $2, $3);}
    | relational_expression OP_LE shift_expression          {$$ = Expression::make_binary($1, $2, $3);}
    | relational_expression OP_GE shift_expression          {$$ = Expression::make_binary($1, $2, $3);}
    | shift_expression                                      {$$ = $1;}
    ;

equality_expression
    : equality_expression OP_EQ relational_expression       {$$ = Expression::make_binary($1, $2, $3);}
    | equality_expression OP_NE relational_expression       {$$ = Expression::make_binary($1, $2, $3);}
    | relational_expression                                 {$$ = $1;}
    ;

and_expression
    : and_expression '&' equality_expression                {$$ = Expression::make_binary($1, $2, $3);}
    | equality_expression                                   {$$ = $1;}
    ;

exclusive_or_expression
    : exclusive_or_expression '^' and_expression            {$$ = Expression::make_binary($1, $2, $3);}
    | and_expression                                        {$$ = $1;}
    ;

inclusive_or_expression
    : inclusive_or_expression '|' exclusive_or_expression   {$$ = Expression::make_binary($1, $2, $3);}
    | exclusive_or_expression                               {$$ = $1;}
    ;

logical_and_expression
    : logical_and_expression OP_AND inclusive_or_expression {$$ = Expression::make_binary($1, $2, $3);}
    | inclusive_or_expression                               {$$ = $1;}
    ;

logical_or_expression
    : logical_or_expression OP_OR logical_and_expression    {$$ = Expression::make_binary($1, $2, $3);}
    | logical_and_expression                                {$$ = $1;}
    ;

conditional_expression
    : logical_or_expression                                             {$$ = $1;}
    | logical_or_expression '?' expression ':' conditional_expression   {$$ = Expression::make_conditional($1, $3, $5);}
    ;

assignment_expression
    : conditional_expression                                        {$$ = $1;}
    | unary_expression assignment_operator assignment_expression    {$$ = Expression::make_assign($1, $2, $3)}
    ;

assignment_operator
    : '='                   {$$ = $1;}
    | OP_MUL_ASSIGN         {$$ = $1;}
    | OP_DIV_ASSIGN         {$$ = $1;}
    | OP_MOD_ASSIGN         {$$ = $1;}
    | OP_ADD_ASSIGN         {$$ = $1;}
    | OP_SUB_ASSIGN         {$$ = $1;}
    | OP_L_SHIFT_ASSIGN     {$$ = $1;}
    | OP_R_SHIFT_ASSIGN     {$$ = $1;}
    | OP_AND_ASSIGN         {$$ = $1;}
    | OP_XOR_ASSIGN         {$$ = $1;}
    | OP_OR_ASSIGN          {$$ = $1;}
    ;

expression
    : assignment_expression                     {$$ = $1;}
    | expression ',' assignment_expression      {$$ = Expression::make_comma($1, $3);}
    ;

constant_expression
    : conditional_expression                    {$$ = $1;}
    ;

/* 6.7.6 Type names (for casts/sizeof) */

type_name
    : specifier_qualifier_list abstract_declarator_opt  { $$ = CompleteDecl::make($1, $2); }
    ;

%%