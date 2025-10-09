%{
use crate::lex::types::token_kind::TokenKind::*;
use crate::lex::types::token_kind::TokenKind;
%}

%type                   TokenKind

%%
"..." 	                { Ellipsis }
"->" 	                { Arrow }
"++" 	                { Inc }
"--" 	                { Dec }


"+=" 	                { PlusEq }
"-=" 	                { MinusEq }
"*=" 	                { StarEq }
"/=" 	                { SlashEq }
"%=" 	                { PercentEq }
"<<=" 	                { ShlEq }
">>=" 	                { ShrEq }
"&=" 	                { AmpEq }
"^=" 	                { CaretEq }
"|=" 	                { PipeEq }


"==" 	                { Eq }
"!=" 	                { Ne }
">=" 	                { Ge }
"<=" 	                { Le }
"&&" 	                { And }
"||" 	                { Or }
"<<" 	                { Shl }
">>" 	                { Shr }


"+"                     { Plus }
"-"                     { Minus }
"*"                     { Star }
"/"                     { Slash }
"%"                     { Percent }
"!"                     { Amp }
"&"                     { Pipe }
"|"                     { Caret }
"^"                     { Caret }
"~"                     { Tilde }
"="                     { Assign }
"<"                     { Lt }
">"                     { Gt }


"("                     { LParen }
")"                     { RParen }
"{"                     { LBrace }
"}"                     { RBrace }
"["                     { LBracket }
"]"                     { RBracket }
";"                     { Semi }
","                     { Comma }
"."                     { Dot }
"?"                     { Question }
":"                     { Colon }
%%