%{
use crate::types::lex::token_kind::TokenKind::*;
%}

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

auto 	                { return KeywordAuto.into(); }
break 	                { return KeywordBreak.into(); }
case 	                { return KeywordCase.into(); }
char 	                { return KeywordChar.into(); }
const 	                { return KeywordConst.into(); }
continue 	            { return KeywordContinue.into(); }
default 	            { return KeywordDefault.into(); }
do 	                    { return KeywordDo.into(); }
double 	                { return KeywordDouble.into(); }
else 	                { return KeywordElse.into(); }
enum 	                { return KeywordEnum.into(); }
extern 	                { return KeywordExtern.into(); }
float 	                { return KeywordFloat.into(); }
for 	                { return KeywordFor.into(); }
goto 	                { return KeywordGoto.into(); }
if 	                    { return KeywordIf.into(); }
int 	                { return KeywordInt.into(); }
long 	                { return KeywordLong.into(); }
register    	        { return KeywordRegister.into(); }
return 	                { return KeywordReturn.into(); }
short 	                { return KeywordShort.into(); }
signed 	                { return KeywordSigned.into(); }
sizeof 	                { return KeywordSizeof.into(); }
static 	                { return KeywordStatic.into(); }
struct 	                { return KeywordStruct.into(); }
switch 	                { return KeywordSwitch.into(); }
typedef 	            { return KeywordTypedef.into(); }
union 	                { return KeywordUnion.into(); }
unsigned    	        { return KeywordUnsigned.into(); }
void        	        { return KeywordVoid.into(); }
volatile    	        { return KeywordVolatile.into(); }
while 	                { return KeywordWhile.into(); }


"..." 	                { return OpEllipsis.into(); }
"->" 	                { return OpArrow.into(); }
"++" 	                { return OpInc.into(); }
"--" 	                { return OpDec.into(); }


"+=" 	                { return OpAddAssign.into(); }
"-=" 	                { return OpSubAssign.into(); }
"*=" 	                { return OpMulAssign.into(); }
"/=" 	                { return OpDivAssign.into(); }
"%=" 	                { return OpModAssign.into(); }
"<<=" 	                { return OpLShiftAssign.into(); }
">>=" 	                { return OpRShiftAssign.into(); }
"&=" 	                { return OpAndAssign.into(); }
"^=" 	                { return OpXorAssign.into(); }
"|=" 	                { return OpOrAssign.into(); }


"==" 	                { return OpEq.into(); }
"!=" 	                { return OpNe.into(); }
">=" 	                { return OpGe.into(); }
"<=" 	                { return OpLe.into(); }
"&&" 	                { return OpAnd.into(); }
"||" 	                { return OpOr.into(); }
"<<" 	                { return OpLShift.into(); }
">>" 	                { return OpRShift.into(); }


"+"                     { return  '+'.into(); }
"-"                     { return  '-'.into(); }
"*"                     { return  '*'.into(); }
"/"                     { return  '/'.into(); }
"%"                     { return  '%'.into(); }
"!"                     { return  '!'.into(); }
"&"                     { return  '&'.into(); }
"|"                     { return  '|'.into(); }
"^"                     { return  '^'.into(); }
"~"                     { return  '~'.into(); }
"="                     { return  '='.into(); }
">"                     { return  '>'.into(); }
"<"                     { return  '<'.into(); }


"("                     { return  '('.into(); }
")"                     { return  ')'.into(); }
"{"                     { return  LParen.into(); }
"}"                     { return  RParen.into(); }
"["                     { return  '['.into(); }
"]"                     { return  ']'.into(); }
";"                     { return  ';'.into(); }
","                     { return  ','.into(); }
"."                     { return  '.'.into(); }
"?"                     { return  '?'.into(); }
":"                     { return  ':'.into(); }

{ID}                    { return Id; }

{HEX}                   { return Hex; }
{OCT}                   { return Oct; }
{INT}                   { return Int; }
{FLOAT}                 { return Float; }

{STRING_LITERAL}        { return StringLiteral; }
{CHARACTER_CONSTANT}    { return CHARACTER_CONSTANT; }

"/*"                    { $self.mode = LexMode::BlockCommon; }
"//"                    { $self.mode = LexMode::LineCommon; }

{WHITESPACE}            { ; }

%%


