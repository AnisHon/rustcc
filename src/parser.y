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

%type SemanticValue

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
    : external_declaration                  {$$ = TranslationUnit::make_single($1);}
    | translation_unit external_declaration {$$ = TranslationUnit::make_multi($1, $2);}
    ;

external_declaration
    : function_definition   {$$ = ExternalDeclaration::make_function_definition($1);}
    | declaration           {$$ = ExternalDeclaration::make_declaration($1);}
    ;

/* 6.9.1 Function definition (C89 allows old-style parameter decls) */
function_definition
    : declaration_specifiers declarator declaration_list_opt compound_statement     {$$ = FunctionDefinition::make_with_specifiers($1, $2, $3, $4);}
    | declarator declaration_list_opt compound_statement                            {$$ = FunctionDefinition::make_without_specifiers($1, $2, $3);}
    ;

declaration_list_opt
    : /* empty */       {$$ = SemanticValue::DeclarationListOpt(None);}
    | declaration_list  {$$ = make_declaration_list_opt($1);}
    ;

declaration_list
    : declaration                   {$$ = DeclarationList::make_decl_list($1);}
    | declaration_list declaration  {$$ = DeclarationList::insert($1, $2);}
    ;

/* 6.7 Declarations */

declaration
    : declaration_specifiers init_declarator_list_opt SEMICOLON {$$ = Declaration::make_declaration($1, $2);}
    ;

init_declarator_list_opt
    : /* empty */           {$$ = SemanticValue::InitDeclaratorListOpt(None);}
    | init_declarator_list  {$$ = make_init_declarator_list_opt($1);}
    ;

init_declarator_list
    : init_declarator                               {$$ = InitDeclaratorList::make_init_decl_list($1);}
    | init_declarator_list COMMA init_declarator    {$$ = InitDeclaratorList::insert($1, $3);}
    ;

init_declarator
    : declarator                        {$$ = InitDeclarator::make_plain($1);}
    | declarator OP_ASSIGN initializer  {$$ = InitDeclarator::make_initialized($1, $3);}
    ;

/* specifiers and qualifiers */

declaration_specifiers
    : storage_class_specifier declaration_specifiers_opt    {$$ = DeclarationSpecifiers::make_storage_class($1, $2);}
    | type_specifier        declaration_specifiers_opt      {$$ = DeclarationSpecifiers::make_type_specifier($1, $2);}
    | type_qualifier        declaration_specifiers_opt      {$$ = DeclarationSpecifiers::make_type_qualifier($1, $2);}
    ;

declaration_specifiers_opt
    : /* empty */               {$$ = SemanticValue::DeclarationSpecifiersOpt(None);}
    | declaration_specifiers    {$$ = make_declarator_list_opt($1);}
    ;

storage_class_specifier
    : KEYWORD_TYPEDEF   {$$ = StorageClassSpecifier::make_typedef();}
    | KEYWORD_EXTERN    {$$ = StorageClassSpecifier::make_extern();}
    | KEYWORD_STATIC    {$$ = StorageClassSpecifier::make_static();}
    | KEYWORD_AUTO      {$$ = StorageClassSpecifier::make_auto();}
    | KEYWORD_REGISTER  {$$ = StorageClassSpecifier::make_register();}
    ;

type_specifier
    : KEYWORD_VOID              {$$ = TypeSpecifier::make_void();}
    | KEYWORD_CHAR              {$$ = TypeSpecifier::make_char();}
    | KEYWORD_SHORT             {$$ = TypeSpecifier::make_short();}
    | KEYWORD_INT               {$$ = TypeSpecifier::make_int();}
    | KEYWORD_LONG              {$$ = TypeSpecifier::make_long();}
    | KEYWORD_SIGNED            {$$ = TypeSpecifier::make_signed();}
    | KEYWORD_UNSIGNED          {$$ = TypeSpecifier::make_unsigned();}
    | KEYWORD_FLOAT             {$$ = TypeSpecifier::make_float();}
    | KEYWORD_DOUBLE            {$$ = TypeSpecifier::make_double();}
    | struct_or_union_specifier {$$ = TypeSpecifier::make_struct($1);}
    | enum_specifier            {$$ = TypeSpecifier::make_enum($1);}
    | TYPE_NAME                 {$$ = TypeSpecifier::make_type_name($1);}      /* resolved by lexer using typedef table */
    ;

type_qualifier
    : KEYWORD_CONST     {$$ = TypeQualifier::make_const();}
    | KEYWORD_VOLATILE  {$$ = TypeQualifier::make_volatile();}
    ;

struct_or_union_specifier
    : struct_or_union identifier_opt LBRACE struct_declaration_list RBRACE  {$$ = StructOrUnionSpecifier::make_defined($1, $2, $4);}
    | struct_or_union ID                                                    {$$ = StructOrUnionSpecifier::make_named($1, $2);}
    ;

struct_or_union
    : KEYWORD_STRUCT    {$$ = StructOrUnion::make_struct();}
    | KEYWORD_UNION     {$$ = StructOrUnion::make_union();}
    ;

identifier_opt
    : /* empty */   {$$ = SemanticValue::IdentifierOpt(None);}
    | ID            {$$ = make_identifier_opt($1);}
    ;

struct_declaration_list
    : struct_declaration                            {$$ = make_struct_declaration_list($1);}
    | struct_declaration_list struct_declaration    {$$ = insert_struct_declaration_list($1, $2);}
    ;

struct_declaration
    : specifier_qualifier_list struct_declarator_list SEMICOLON {$$ = StructDeclaration::make_struct_declaration($1, $2);}
    ;

specifier_qualifier_list
    : type_specifier specifier_qualifier_list_opt   {$$ = SpecifierQualifierList::make_type_specifier($1, $2);}
    | type_qualifier specifier_qualifier_list_opt   {$$ = SpecifierQualifierList::make_type_qualifier($1, $2);}
    ;

specifier_qualifier_list_opt
    : /* empty */               {$$ = SemanticValue::SpecifierQualifierListOpt(None);}
    | specifier_qualifier_list  {$$ = make_specifier_qualifier_list_opt($1);}
    ;

struct_declarator_list
    : struct_declarator                                 {$$ = make_struct_declarator_list($1);}
    | struct_declarator_list COMMA struct_declarator    {$$ = insert_struct_declarator_list($1, $3);}
    ;

struct_declarator
    : declarator                            {$$ = StructDeclarator::make_declarator($1);}
    | COLON constant_expression             {$$ = StructDeclarator::make_bitfield(SemanticValue::None, $2);}
    | declarator COLON constant_expression  {$$ = StructDeclarator::make_bitfield($1, $3);}
    ;

enum_specifier
    : KEYWORD_ENUM identifier_opt LBRACE enumerator_list RBRACE {$$ = EnumSpecifier::make_defined($2, $4);}
    | KEYWORD_ENUM ID                                           {$$ = EnumSpecifier::make_named($2);}
    ;

enumerator_list
    : enumerator                        {$$ = make_enumerator_list($1);}
    | enumerator_list COMMA enumerator  {$$ = insert_enumerator_list($1, $3);}
    ;

enumerator
    : ID                                {$$ = Enumerator::make_plain($1);}
    | ID OP_ASSIGN constant_expression  {$$ = Enumerator::make_with_value($1, $3);}
    ;

/* declarators */

declarator
    : pointer_opt direct_declarator {$$ = Declarator::make_declarator($1, $2);}
    ;

pointer_opt
    : /* empty */   {$$ = SemanticValue::PointerOpt(None);}
    | pointer       {$$ = make_pointer_opt($1);}
    ;

pointer
    : OP_TIMES                              {$$ = Pointer::make_pointer(SemanticValue::None, SemanticValue::None);}
    | OP_TIMES type_qualifier_list          {$$ = Pointer::make_pointer($2, SemanticValue::None);}
    | OP_TIMES pointer                      {$$ = Pointer::make_pointer(SemanticValue::None, $3);}
    | OP_TIMES type_qualifier_list pointer  {$$ = Pointer::make_pointer($2, $3);}
    ;

type_qualifier_list
    : type_qualifier                        {$$ = make_type_qualifier_list($1);}
    | type_qualifier_list type_qualifier    {$$ = insert_type_qualifier_list($1, $2);}
    ;

direct_declarator
    : ID                                                            {$$ = DirectDeclarator::make_id($1);}
    | LPAREN declarator RPAREN                                      {$$ = DirectDeclarator::make_paren($2);}
    | direct_declarator LBRACKET constant_expression_opt RBRACKET   {$$ = DirectDeclarator::make_array($1, $3);}
    | direct_declarator LPAREN parameter_type_list RPAREN           {$$ = DirectDeclarator::make_func_params($1, $3);}
    | direct_declarator LPAREN identifier_list_opt RPAREN           {$$ = DirectDeclarator::make_func_identifiers($1, $3);}
    ;

constant_expression_opt
    : /* empty */           {$$ = SemanticValue::ConstantExpressionOpt(None);}
    | constant_expression   {$$ = make_constant_expression_opt($1);}
    ;

identifier_list_opt
    : /* empty */       {$$ = SemanticValue::IdentifierListOpt(None);}
    | identifier_list   {$$ = make_identifier_list_opt($1);}
    ;

identifier_list
    : ID                        {$$ = IdentifierList::make_list($1);}
    | identifier_list COMMA ID  {$$ = IdentifierList::insert($1, $3);}
    ;

parameter_type_list
    : parameter_list                    {$$ = ParameterTypeList::make_params($1);}
    | parameter_list COMMA OP_ELLIPSIS  {$$ = ParameterTypeList::make_variadic($1);}
    ;

parameter_list
    : parameter_declaration                         {$$ = make_parameter_list($1);}
    | parameter_list COMMA parameter_declaration    {$$ = insert_parameter_list($1, $3);}
    ;

parameter_declaration
    : declaration_specifiers declarator                 {$$ = ParameterDeclaration::make_declarator($1, $2);}
    | declaration_specifiers abstract_declarator_opt    {$$ = ParameterDeclaration::make_abstract($1, $2);}
    ;

abstract_declarator_opt
    : /* empty */           {$$ = SemanticValue::AbstractDeclaratorOpt(None);}
    | abstract_declarator   {$$ = make_abstract_declarator_opt($1);}
    ;

abstract_declarator
    : pointer                                   {$$ = AbstractDeclarator::make_pointer($1);}
    | pointer_opt direct_abstract_declarator    {$$ = AbstractDeclarator::make_direct($1, $2);}
    ;

direct_abstract_declarator
    : LPAREN abstract_declarator RPAREN                                     {$$ = DirectAbstractDeclarator::make_paren($2);}
    | LBRACKET constant_expression_opt RBRACKET                             {$$ = DirectAbstractDeclarator::make_array($2);}
    | direct_abstract_declarator LBRACKET constant_expression_opt RBRACKET  {$$ = DirectAbstractDeclarator::make_array_nested($1, $3);}
    | LPAREN parameter_type_list_opt RPAREN                                 {$$ = DirectAbstractDeclarator::make_func($2);}
    | direct_abstract_declarator LPAREN parameter_type_list_opt RPAREN      {$$ = DirectAbstractDeclarator::make_func_nested($1, $3);}
    ;

parameter_type_list_opt
    : /* empty */           {$$ = SemanticValue::ParameterTypeListOpt(None);}
    | parameter_type_list   {$$ = make_parameter_type_list_opt($1);}
    ;

/* Initializers (C89) */
initializer
    : assignment_expression                 {$$ = Initializer::make_assignment($1);}
    | LBRACE initializer_list RBRACE        {$$ = Initializer::make_list($2);}
    | LBRACE initializer_list COMMA RBRACE  {$$ = Initializer::make_list($2);}  /* trailing comma is widely accepted; tighten if needed */
    ;

initializer_list
    : initializer                           {$$ = InitializerList::make($1);}
    | initializer_list COMMA initializer    {$$ = InitializerList::insert($1, $3);}
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
    : ID COLON statement                                {$$ = LabeledStatement::make_label($1, $3);}
    | KEYWORD_CASE constant_expression COLON statement  {$$ = LabeledStatement::make_case($2, $4);}
    | KEYWORD_DEFAULT COLON statement                   {$$ = LabeledStatement::make_default($3);}
    ;

compound_statement
    : LBRACE RBRACE                 {$$ = CompoundStatement::make_empty();}
    | LBRACE block_item_list RBRACE {$$ = CompoundStatement::make_expr($2);}
    ;

block_item_list
    : block_item                    {$$ = make_block_item($1);}
    | block_item_list block_item    {$$ = insert_block_item($1, $2);}
    ;

block_item
    : declaration   {$$ = BlockItem::make_declaration($1);}
    | statement     {$$ = BlockItem::make_statement($1);}
    ;

expression_statement
    : SEMICOLON             {$$ = ExpressionStatement::make_empty();}
    | expression SEMICOLON  {$$ = ExpressionStatement::make_expr($1);}
    ;

selection_statement
    : KEYWORD_IF LPAREN expression RPAREN statement                         {$$ = SelectionStatement::make_if($3, $5, SemanticValue::None);} %prec nonassoc
    | KEYWORD_IF LPAREN expression RPAREN statement KEYWORD_ELSE statement  {$$ = SelectionStatement::make_if($3, $5, $7);}
    | KEYWORD_SWITCH LPAREN expression RPAREN statement                     {$$ = SelectionStatement::make_switch($3, $5);}
    ;

iteration_statement
    : KEYWORD_WHILE LPAREN expression RPAREN statement                                                      {$$ = IterationStatement::make_while($3, $5);}
    | KEYWORD_DO statement KEYWORD_WHILE LPAREN expression RPAREN SEMICOLON                                 {$$ = IterationStatement::make_do_while($2, $5);}
    | KEYWORD_FOR LPAREN expression_opt SEMICOLON expression_opt SEMICOLON expression_opt RPAREN statement  {$$ = IterationStatement::make_for($3, $5, $7, $9);}
    ;

expression_opt
    : /* empty */   {$$ = SemanticValue::ExpressionOpt(None);}
    | expression    {$$ = make_expression_opt($1);}
    ;

jump_statement
    : KEYWORD_GOTO ID SEMICOLON             {$$ = JumpStatement::make_goto($2);}
    | KEYWORD_CONTINUE SEMICOLON            {$$ = JumpStatement::make_continue();}
    | KEYWORD_BREAK SEMICOLON               {$$ = JumpStatement::make_break();}
    | KEYWORD_RETURN SEMICOLON              {$$ = JumpStatement::make_return(SemanticValue::None);}
    | KEYWORD_RETURN expression SEMICOLON   {$$ = JumpStatement::make_return($2);}
    ;

/* 6.5 Expressions */
primary_expression
    : ID                        {$$ = PrimaryExpression::make_id($1);}
    | constant                  {$$ = PrimaryExpression::make_constant($1);}
    | string                    {$$ = PrimaryExpression::make_string_literal($1);}
    | LPAREN expression RPAREN  {$$ = PrimaryExpression::make_paren($2);}
    ;

constant
    : INT                   {$$ = Constant::make_int();}
    | FLOAT                 {$$ = Constant::make_float();}
    | CHARACTER_CONSTANT    {$$ = Constant::make_char();}
    ;

/* adjacent string literal concatenation */
string
    : STRING_LITERAL        {$$ = make_string($1);}
    | string STRING_LITERAL {$$ = insert_string($1, $2);}
    ;

postfix_expression
    : primary_expression                                            {$$ = PostfixExpression::make_primary($1);}
    | postfix_expression LBRACKET expression RBRACKET               {$$ = PostfixExpression::make_array($1, $3);}
    | postfix_expression LPAREN argument_expression_list_opt RPAREN {$$ = PostfixExpression::make_call($1, $3);}
    | postfix_expression DOT ID                                     {$$ = PostfixExpression::make_field($1, $3);}
    | postfix_expression OP_ARROW ID                                {$$ = PostfixExpression::make_arrow($1, $3);}
    | postfix_expression OP_INC                                     {$$ = PostfixExpression::make_inc($1);}
    | postfix_expression OP_DEC                                     {$$ = PostfixExpression::make_dec($1);}
    ;

argument_expression_list_opt
    : /* empty */               {$$ = SemanticValue::ArgumentExpressionListOpt(None)}
    | argument_expression_list  {$$ = make_argument_expression_list_opt($1);}
    ;

argument_expression_list
    : assignment_expression                                 {$$ = makeargument_expression_list($1);}
    | argument_expression_list COMMA assignment_expression  {$$ = insert_argument_expression_list($1, $3);}
    ;

unary_expression
    : postfix_expression                        {$$ = UnaryExpression::make_postfix($1);}
    | OP_INC unary_expression                   {$$ = UnaryExpression::make_pre_inc($2);}
    | OP_DEC unary_expression                   {$$ = UnaryExpression::make_pre_dec($2);}
    | unary_operator cast_expression            {$$ = UnaryExpression::make_unary_op($1, $2);}
    | KEYWORD_SIZEOF unary_expression           {$$ = UnaryExpression::make_sizeof_expr($2);}
    | KEYWORD_SIZEOF LPAREN type_name RPAREN    {$$ = UnaryExpression::make_sizeof_type($3);}
    ;

unary_operator
    : OP_BITAND             {$$ = UnaryOperator::address_of();}
    | OP_TIMES              {$$ = UnaryOperator::deref();}
    | OP_PLUS               {$$ = UnaryOperator::plus();}    %prec right
    | OP_MINUS              {$$ = UnaryOperator::minus();}   %prec right
    | OP_BIT_NOT            {$$ = UnaryOperator::bit_not();}
    | OP_NOT                {$$ = UnaryOperator::not();}
    ;

cast_expression
    : LPAREN type_name RPAREN cast_expression       {$$ = CastExpression::make_cast($2, $4);}
    | unary_expression                              {$$ = CastExpression::make_unary($1);}
    ;

multiplicative_expression
    : multiplicative_expression OP_TIMES cast_expression        {$$ = MultiplicativeExpression::make_mul($1, $3);}
    | multiplicative_expression OP_DIVIDE cast_expression       {$$ = MultiplicativeExpression::make_div($1, $3);}
    | multiplicative_expression OP_MOD cast_expression          {$$ = MultiplicativeExpression::make_mod($1, $3);}
    | cast_expression                                           {$$ = MultiplicativeExpression::make_cast($1);}
    ;

additive_expression
    : additive_expression OP_PLUS multiplicative_expression     {$$ = AdditiveExpression::make_add($1, $3);}
    | additive_expression OP_MINUS multiplicative_expression    {$$ = AdditiveExpression::make_sub($1, $3);}
    | multiplicative_expression                                 {$$ = AdditiveExpression::make_mul($1);}
    ;

shift_expression
    : shift_expression OP_L_SHIFT additive_expression           {$$ = ShiftExpression::make_shl($1, $3);}
    | shift_expression OP_R_SHIFT additive_expression           {$$ = ShiftExpression::make_shr($1, $3);}
    | additive_expression                                       {$$ = ShiftExpression::make_add($1);}
    ;

relational_expression
    : relational_expression OP_LT shift_expression              {$$ = RelationalExpression::make_lt($1, $3);}
    | relational_expression OP_GT shift_expression              {$$ = RelationalExpression::make_gt($1, $3);}
    | relational_expression OP_LE shift_expression              {$$ = RelationalExpression::make_le($1, $3);}
    | relational_expression OP_GE shift_expression              {$$ = RelationalExpression::make_ge($1, $3);}
    | shift_expression                                          {$$ = RelationalExpression::make_shift($1);}
    ;

equality_expression
    : equality_expression OP_EQ relational_expression           {$$ = EqualityExpression::make_eq($1, $3);}
    | equality_expression OP_NE relational_expression           {$$ = EqualityExpression::make_ne($1, $3);}
    | relational_expression                                     {$$ = EqualityExpression::make_rel($1);}
    ;

and_expression
    : and_expression OP_BITAND equality_expression              {$$ = AndExpression::make_and($1, $3);}
    | equality_expression                                       {$$ = AndExpression::make_eq($1)}
    ;

exclusive_or_expression
    : exclusive_or_expression OP_XOR and_expression             {$$ = ExclusiveOrExpression::make_xor($1, $3);}
    | and_expression                                            {$$ = ExclusiveOrExpression::make_and($1);}
    ;

inclusive_or_expression
    : inclusive_or_expression OP_BITOR exclusive_or_expression  {$$ = InclusiveOrExpression::make_or($1, $3);}
    | exclusive_or_expression                                   {$$ = InclusiveOrExpression::make_xor($1);}
    ;

logical_and_expression
    : logical_and_expression OP_AND inclusive_or_expression     {$$ = LogicalAndExpression::make_and($1, $3);}
    | inclusive_or_expression                                   {$$ = LogicalAndExpression::make_or($1);}
    ;

logical_or_expression
    : logical_or_expression OP_OR logical_and_expression        {$$ = LogicalOrExpression::make_or($1, $3);}
    | logical_and_expression                                    {$$ = LogicalOrExpression::make_and($1);}
    ;

conditional_expression
    : logical_or_expression                                                     {$$ = ConditionalExpression::make_or($1);}
    | logical_or_expression QUESTION expression COLON conditional_expression    {$$ = ConditionalExpression::make_cond($1, $3, $5);}
    ;

assignment_expression
    : conditional_expression                                        {$$ = AssignmentExpression::make_conditional($1);}
    | unary_expression assignment_operator assignment_expression    {$$ = AssignmentExpression::make_assign($1, $2, $3)}
    ;

assignment_operator
    : OP_ASSIGN                 {$$ = AssignmentOperator::assign();}
    | OP_MUL_ASSIGN             {$$ = AssignmentOperator::mul_assign();}
    | OP_DIV_ASSIGN             {$$ = AssignmentOperator::div_assign();}
    | OP_MOD_ASSIGN             {$$ = AssignmentOperator::mod_assign();}
    | OP_ADD_ASSIGN             {$$ = AssignmentOperator::add_assign();}
    | OP_SUB_ASSIGN             {$$ = AssignmentOperator::sub_assign();}
    | OP_L_SHIFT_ASSIGN         {$$ = AssignmentOperator::shl_assign();}
    | OP_R_SHIFT_ASSIGN         {$$ = AssignmentOperator::shr_assign();}
    | OP_AND_ASSIGN             {$$ = AssignmentOperator::and_assign();}
    | OP_XOR_ASSIGN             {$$ = AssignmentOperator::xor_assign();}
    | OP_OR_ASSIGN              {$$ = AssignmentOperator::or_assign();}
    ;

expression
    : assignment_expression                     {$$ = Expression::make_single($1);}
    | expression COMMA assignment_expression    {$$ = Expression::make_comma($1, $3);}
    ;

constant_expression
    : conditional_expression                    {$$ = ConditionalExpression::make_constant($1);}
    ;

/* 6.7.6 Type names (for casts/sizeof) */

type_name
    : specifier_qualifier_list abstract_declarator_opt  {$$ = TypeName::make_type_name($1, $2);}
    ;

%%
use crate::parser::cst::*;