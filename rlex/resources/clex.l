KEYWORD_AUTO                auto
KEYWORD_BREAK               break
KEYWORD_CASE                case
KEYWORD_CHAR                char
KEYWORD_CONST               const
KEYWORD_CONTINUE            continue
KEYWORD_DEFAULT             default
KEYWORD_DO                  do
KEYWORD_DOUBLE              double
KEYWORD_ELSE                else
KEYWORD_ENUM                enum
KEYWORD_EXTERN              extern
KEYWORD_FLOAT               float
KEYWORD_FOR                 for
KEYWORD_GOTO                goto
KEYWORD_IF                  if
KEYWORD_INT                 int
KEYWORD_LONG                long
KEYWORD_REGISTER            register
KEYWORD_RETURN              return
KEYWORD_SHORT               short
KEYWORD_SIGNED              signed
KEYWORD_SIZEOF              sizeof
KEYWORD_STATIC              static
KEYWORD_STRUCT              struct
KEYWORD_SWITCH              switch
KEYWORD_TYPEDEF             typedef
KEYWORD_UNION               union
KEYWORD_UNSIGNED            unsigned
KEYWORD_VOID                void
KEYWORD_VOLATILE            volatile
KEYWORD_WHILE               while


OP_ELLIPSIS            "..."
OP_ARROW               "->"
OP_INC                 "++"
OP_DEC                 "--"


OP_ADD_ASSIGN          "+="
OP_SUB_ASSIGN          "-="
OP_MUL_ASSIGN          "*="
OP_DIV_ASSIGN          "/="
OP_MOD_ASSIGN          "%="
OP_L_SHIFT_ASSIGN      "<<="
OP_R_SHIFT_ASSIGN      ">>="
OP_AND_ASSIGN          "&="
OP_XOR_ASSIGN          "^="
OP_OR_ASSIGN           "|="


OP_EQ                  "=="
OP_NE                  "!="
OP_GE                  ">="
OP_LE                  "<="
OP_AND                 "&&"
OP_OR                  "||"


OP_L_SHIFT             "<<"
OP_R_SHIFT             ">>"


OP_PLUS                "+"
OP_MINUS               "-"
OP_TIMES               "*"
OP_DIVIDE              "/"
OP_MOD                 "%"
OP_NOT                 "!"
OP_BITAND              "&"
OP_BITOR               "|"
OP_XOR                 "^"
OP_BIT_NOT              "~"
OP_ASSIGN              "="
OP_GT                  ">"
OP_LT                  "<"


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