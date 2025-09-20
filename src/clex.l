%{
use crate::types::lex::token_kind::TokenKind::*;
use crate::lex::lex_core::LexMode;
use crate::lex::lex_core::Lex;
use std::io::Read;
%}

%param                  { lex: &mut Lex<impl Read> }

ID                      [a-zA-Z_]\w*

HEX                     0[xX][0-9a-fA-F]+([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
OCT                     0[0-7]*([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
INT                     (0|[1-9][0-9]*)([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
FLOAT                   (([0-9]+\.[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+)([fFlL])?

STRING_LITERAL          \"([^"\\\n]|\\.)*\"
CHARACTER_CONSTANT      '([^\\'\n]|\\.)*'

LINE_COMMENT
BLOCK_COMMENT

WHITESPACE              \s+

%%

auto 	                { return Some(KeywordAuto as usize); }
break 	                { return Some(KeywordBreak as usize); }
case 	                { return Some(KeywordCase as usize); }
char 	                { return Some(KeywordChar as usize); }
const 	                { return Some(KeywordConst as usize); }
continue 	            { return Some(KeywordContinue as usize); }
default 	            { return Some(KeywordDefault as usize); }
do 	                    { return Some(KeywordDo as usize); }
double 	                { return Some(KeywordDouble as usize); }
else 	                { return Some(KeywordElse as usize); }
enum 	                { return Some(KeywordEnum as usize); }
extern 	                { return Some(KeywordExtern as usize); }
float 	                { return Some(KeywordFloat as usize); }
for 	                { return Some(KeywordFor as usize); }
goto 	                { return Some(KeywordGoto as usize); }
if 	                    { return Some(KeywordIf as usize); }
int 	                { return Some(KeywordInt as usize); }
long 	                { return Some(KeywordLong as usize); }
register    	        { return Some(KeywordRegister as usize); }
return 	                { return Some(KeywordReturn as usize); }
short 	                { return Some(KeywordShort as usize); }
signed 	                { return Some(KeywordSigned as usize); }
sizeof 	                { return Some(KeywordSizeof as usize); }
static 	                { return Some(KeywordStatic as usize); }
struct 	                { return Some(KeywordStruct as usize); }
switch 	                { return Some(KeywordSwitch as usize); }
typedef 	            { return Some(KeywordTypedef as usize); }
union 	                { return Some(KeywordUnion as usize); }
unsigned    	        { return Some(KeywordUnsigned as usize); }
void        	        { return Some(KeywordVoid as usize); }
volatile    	        { return Some(KeywordVolatile as usize); }
while 	                { return Some(KeywordWhile as usize); }


"..." 	                { return Some(OpEllipsis as usize); }
"->" 	                { return Some(OpArrow as usize); }
"++" 	                { return Some(OpInc as usize); }
"--" 	                { return Some(OpDec as usize); }


"+=" 	                { return Some(OpAddAssign as usize); }
"-=" 	                { return Some(OpSubAssign as usize); }
"*=" 	                { return Some(OpMulAssign as usize); }
"/=" 	                { return Some(OpDivAssign as usize); }
"%=" 	                { return Some(OpModAssign as usize); }
"<<=" 	                { return Some(OpLShiftAssign as usize); }
">>=" 	                { return Some(OpRShiftAssign as usize); }
"&=" 	                { return Some(OpAndAssign as usize); }
"^=" 	                { return Some(OpXorAssign as usize); }
"|=" 	                { return Some(OpOrAssign as usize); }


"==" 	                { return Some(OpEq as usize); }
"!=" 	                { return Some(OpNe as usize); }
">=" 	                { return Some(OpGe as usize); }
"<=" 	                { return Some(OpLe as usize); }
"&&" 	                { return Some(OpAnd as usize); }
"||" 	                { return Some(OpOr as usize); }
"<<" 	                { return Some(OpLShift as usize); }
">>" 	                { return Some(OpRShift as usize); }


"+"                     { return Some('+' as usize); }
"-"                     { return Some('-' as usize); }
"*"                     { return Some('*' as usize); }
"/"                     { return Some('/' as usize); }
"%"                     { return Some('%' as usize); }
"!"                     { return Some('!' as usize); }
"&"                     { return Some('&' as usize); }
"|"                     { return Some('|' as usize); }
"^"                     { return Some('^' as usize); }
"~"                     { return Some('~' as usize); }
"="                     { return Some('=' as usize); }
">"                     { return Some('>' as usize); }
"<"                     { return Some('<' as usize); }


"("                     { return Some('(' as usize); }
")"                     { return Some(')' as usize); }
"{"                     { return Some('{' as usize); }
"}"                     { return Some('}' as usize); }
"["                     { return Some('[' as usize); }
"]"                     { return Some(']' as usize); }
";"                     { return Some(';' as usize); }
","                     { return Some(',' as usize); }
"."                     { return Some('.' as usize); }
"?"                     { return Some('?' as usize); }
":"                     { return Some(':' as usize); }

{ID}                    { return Some(ID as usize); }

{HEX}                   { return Some(Hex as usize); }
{OCT}                   { return Some(Oct as usize); }
{INT}                   { return Some(Int as usize); }
{FLOAT}                 { return Some(Float as usize); }

{STRING_LITERAL}        { return Some(StringLiteral as usize); }
{CHARACTER_CONSTANT}    { return Some(CharacterConstant as usize); }

"/*"                    { lex.set_mode(LexMode::BlockComment); }
"//"                    { lex.set_mode(LexMode::LineComment); }

{WHITESPACE}            { lex.set_mode(LexMode::WhileSpace); }

%%


