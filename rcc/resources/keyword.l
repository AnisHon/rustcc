%{
use crate::lex::types::token_kind::Keyword::*;
use crate::lex::types::token_kind::Keyword;
%}

%type                   Keyword

%%
auto 	                { Auto }
break 	                { Break }
case 	                { Case }
char 	                { Char }
const 	                { Const }
continue 	            { Continue }
default 	            { Default }
do 	                    { Do }
double 	                { Double }
else 	                { Else }
enum 	                { Enum }
extern 	                { Extern }
float 	                { Float }
for 	                { For }
goto 	                { Goto }
if 	                    { If }
int 	                { Int }
inline                  { Inline }
long 	                { Int }
register    	        { Long }
restrict                { Register }
return 	                { Restrict }
short 	                { Return }
sizeof 	                { Short }
signed 	                { Signed }
static 	                { Sizeof }
struct 	                { Struct }
switch 	                { Switch }
typedef 	            { Typedef }
union 	                { Union }
unsigned    	        { Unsigned }
void        	        { Void }
volatile    	        { Volatile }
while 	                { While }
_Bool                   { Bool }
_Complex                { Complex }
_Imaginary              { Complex }
%%


