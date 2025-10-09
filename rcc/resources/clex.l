%{
use crate::types::lex::token_kind::TokenKind::*;
use crate::types::lex::token_kind::Keyword::*;
use crate::types::lex::token_kind::LiteralKind::*;
use crate::types::lex::token_kind::BinOp::*;
use crate::types::lex::token_kind::UnOp::*;
use crate::types::lex::token_kind::Delimiter::*;
use crate::lex::lex_core::LexMode;
use crate::lex::lex_core::Lex;
use std::io::Read;
%}

%type                   TokenKind
%param                  { lex: &mut Lex<impl Read> }


ID                      [a-zA-Z_]\w*

HEX                     0[xX][0-9a-fA-F]+([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
OCT                     0[0-7]*([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
INT                     (0|[1-9][0-9]*)([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
FLOAT                   (([0-9]+\.[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+)([fFlL])?

STRING_LITERAL          \"([^"\\\n]|\\.)*\"
CHARACTER_CONSTANT      '([^\\'\n]|\\.)*'

WHITESPACE              \s+

%%

auto 	                { return Some(TokenKind::Keyword(Auto)); }
break 	                { return Some(TokenKind::Keyword(Break)); }
case 	                { return Some(TokenKind::Keyword(Case)); }
char 	                { return Some(TokenKind::Keyword(Char)); }
const 	                { return Some(TokenKind::Keyword(Const)); }
continue 	            { return Some(TokenKind::Keyword(Continue)); }
default 	            { return Some(TokenKind::Keyword(Default)); }
do 	                    { return Some(TokenKind::Keyword(Do)); }
double 	                { return Some(TokenKind::Keyword(Double)); }
else 	                { return Some(TokenKind::Keyword(Else)); }
enum 	                { return Some(TokenKind::Keyword(Enum)); }
extern 	                { return Some(TokenKind::Keyword(Extern)); }
float 	                { return Some(TokenKind::Keyword(Float)); }
for 	                { return Some(TokenKind::Keyword(For)); }
goto 	                { return Some(TokenKind::Keyword(Goto)); }
if 	                    { return Some(TokenKind::Keyword(If)); }
int 	                { return Some(TokenKind::Keyword(Int)); }
long 	                { return Some(TokenKind::Keyword(Long)); }
register    	        { return Some(TokenKind::Keyword(Register)); }
return 	                { return Some(TokenKind::Keyword(Return)); }
short 	                { return Some(TokenKind::Keyword(Short)); }
signed 	                { return Some(TokenKind::Keyword(Signed)); }
sizeof 	                { return Some(TokenKind::Keyword(Sizeof)); }
static 	                { return Some(TokenKind::Keyword(Static)); }
struct 	                { return Some(TokenKind::Keyword(Struct)); }
switch 	                { return Some(TokenKind::Keyword(Switch)); }
typedef 	            { return Some(TokenKind::Keyword(Typedef)); }
union 	                { return Some(TokenKind::Keyword(Union)); }
unsigned    	        { return Some(TokenKind::Keyword(Unsigned)); }
void        	        { return Some(TokenKind::Keyword(Void)); }
volatile    	        { return Some(TokenKind::Keyword(Volatile)); }
while 	                { return Some(TokenKind::Keyword(While)); }


"..." 	                { return Some(TokenKind::Ellipsis; }
"->" 	                { return Some(TokenKind::Arrow); }
"++" 	                { return Some(TokenKind::Inc); }
"--" 	                { return Some(TokenKind::Dec); }


"+=" 	                { return Some(PlusEq); }
"-=" 	                { return Some(MinusEq); }
"*=" 	                { return Some(StarEq); }
"/=" 	                { return Some(SlashEq); }
"%=" 	                { return Some(PercentEq); }
"<<=" 	                { return Some(ShlEq); }
">>=" 	                { return Some(ShrEq); }
"&=" 	                { return Some(AmpEq); }
"^=" 	                { return Some(CaretEq); }
"|=" 	                { return Some(PipeEq); }


"==" 	                { return Some(Eq); }
"!=" 	                { return Some(Ne); }
">=" 	                { return Some(Ge); }
"<=" 	                { return Some(Le); }
"&&" 	                { return Some(And); }
"||" 	                { return Some(Or); }
"<<" 	                { return Some(Shl); }
">>" 	                { return Some(Shr); }


"+"                     { return Some(Plus); }
"-"                     { return Some(Minus); }
"*"                     { return Some(Star); }
"/"                     { return Some(Slash); }
"%"                     { return Some(Percent); }
"!"                     { return Some(Amp); }
"&"                     { return Some(Pipe); }
"|"                     { return Some(Caret); }
"^"                     { return Some(Caret); }
"~"                     { return Some(Tilde); }
"="                     { return Some(Assign); }
"<"                     { return Some(Lt); }
">"                     { return Some(Gt); }


"("                     { return Some(LParen); }
")"                     { return Some(RParen); }
"{"                     { return Some(LBrace); }
"}"                     { return Some(RBrace); }
"["                     { return Some(LBracket); }
"]"                     { return Some(RBracket); }
";"                     { return Some(Semi); }
","                     { return Some(Comma); }
"."                     { return Some(Dot); }
"?"                     { return Some(Question); }
":"                     { return Some(Colon); }

{ID}                    { return Some(make_ident(self.slice())); }

{HEX}                   { return Some(make_hex(self.slice())); }
{OCT}                   { return Some(make_oct(self.slice())); }
{INT}                   { return Some(make_int(self.slice())); }
{FLOAT}                 { return Some(make_float(self.slice())); }

{STRING_LITERAL}        { return Some(make_string(self.slice())); }
{CHARACTER_CONSTANT}    { return Some(make_char(self.slice())); }

"/*"                    { lex.set_mode(LexMode::BlockComment); }
"//"                    { lex.set_mode(LexMode::LineComment); }

{WHITESPACE}            { }

%%


