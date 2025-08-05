AUTO                auto
BREAK               break
CASE                case
CHAR                char
CONST               const
CONTINUE            continue
DEFAULT             default
DO                  do
DOUBLE              double
ELSE                else
ENUM                enum
EXTERN              extern
FLOAT               float
FOR                 for
GOTO                goto
IF                  if
INT                 int
LONG                long
REGISTER            register
RETURN              return
SHORT               short
SIGNED              signed
SIZEOF              sizeof
STATIC              static
STRUCT              struct
SWITCH              switch
TYPEDEF             typedef
UNION               union
UNSIGNED            unsigned
VOID                void
VOLATILE            volatile
WHILE               while


ELLIPSIS            "..."
ARROW               "->"
INC                 "++"
DEC                 "--"


ADD_ASSIGN          "+="
SUB_ASSIGN          "-="
MUL_ASSIGN          "*="
DIV_ASSIGN          "/="
MOD_ASSIGN          "%="
LSHIFT_ASSIGN       "<<="
RSHIFT_ASSIGN       ">>="
AND_ASSIGN          "&="
XOR_ASSIGN          "^="
OR_ASSIGN           "|="


EQ                  "=="
NE                  "!="
GE                  ">="
LE                  "<="
AND                 "&&"
OR                  "||"


LSHIFT              "<<"
RSHIFT              ">>"


PLUS                "+"
MINUS               "-"
TIMES               "*"
DIVIDE              "/"
MOD                 "%"
NOT                 "!"
BITAND              "&"
BITOR               "|"
XOR                 "^"
BITNOT              "~"
ASSIGN              "="
GT                  ">"
LT                  "<"


LPAREN              "("
RPAREN              ")"
LBRACE              "{"
RBRACE              "}"
LBRACKET            "["
RBRACKET            "]"
SEMICOLON           ";"
COMMA               ","
DOT                 "."
QUESTION            "?"
COLON               ":"


ID                  [a-zA-Z_]\w*

HEX                 0[xX][0-9a-fA-F]+([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
OCT                 0[0-7]*([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
INT                 (0|[1-9][0-9]*)([uU]?[lL]{0,2}|[lL]{1,2}[uU]?)?
FLOAT               (([0-9]+\.[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+)([fFlL])?

STRING_LITERAL      \"([^"\\\n]|.)*\"
CHARACTER_CONSTANT  '(\\.|[^\\'\n])+'

LINE_COMMENT        //.*
BLOCK_COMMENT       /\*

WHITESPACE          \s+