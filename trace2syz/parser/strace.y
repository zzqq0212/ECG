%{
//nolint
package parser

import (
    //"fmt"
)
%}

%start syscall

%union {
    data string
    val_int int64
    val_double float64
    val_uint uint64
    val_field *Field
    val_call *Call
    val_macro *macroType
    val_int_type *intType
    val_identifiers []*BufferType
    val_buf_type *BufferType
    val_group_type *GroupType
    val_pointer_type *PointerType
    val_flag_type *flagType
    val_type IrType
    val_types []IrType
    val_parenthetical *parenthetical
    val_syscall *Syscall
}

%token <data> STRING_LITERAL IPV4 IPV6 IDENTIFIER FLAG DATETIME SIGNAL_PLUS SIGNAL_MINUS MAC
%token <val_int> INT
%token <val_uint> UINT
%token <val_double> DOUBLE
%type <val_field> field_type
%type <val_identifiers> identifiers
%type <val_int_type> int_type
%type <val_buf_type> buf_type
%type <val_group_type> group_type
%type <val_flag_type> flag_type
%type <val_call> call_type
%type <val_parenthetical> parenthetical, parentheticals
%type <val_macro> macro_type
%type <val_type> type, expr_type, flags, ints
%type <val_pointer_type> pointer_type
%type <val_types> types
%type <val_syscall> syscall

%token STRING_LITERAL IPV4 IPV6 MAC IDENTIFIER FLAG INT UINT QUESTION DOUBLE ARROW
%token OR AND LOR TIMES LAND LEQUAL ONESCOMP LSHIFT RSHIFT TIMES NOT
%token COMMA LBRACKET RBRACKET LBRACKET_SQUARE RBRACKET_SQUARE LPAREN RPAREN EQUALS
%token UNFINISHED RESUMED
%token SIGNAL_PLUS SIGNAL_MINUS NULL AT COLON KEYWORD

%nonassoc NOTYPE
%nonassoc FLAG
%nonassoc NOFLAG

%nonassoc EQUAL
%nonassoc ARROW

%left LOR
%left LAND
%left OR
%left AND
%left LEQUAL
%left LSHIFT RSHIFT
%left TIMES
%left ONESCOMP

%%
syscall:
    IDENTIFIER LPAREN types UNFINISHED %prec NOFLAG { $$ = NewSyscall(-1, $1, $3, int64(-1), true, false);
                                                        Stracelex.(*Stracelexer).result = $$ }
    | RESUMED UNFINISHED RPAREN EQUALS QUESTION %prec NOFLAG
        {
            $$ = NewSyscall(-1, "tmp", nil, -1, true, true);
            Stracelex.(*Stracelexer).result = $$;
        }
    | IDENTIFIER LPAREN RESUMED RPAREN EQUALS INT %prec NOFLAG
        {
            $$ = NewSyscall(-1, $1, nil, int64($6), false, false);
            Stracelex.(*Stracelexer).result = $$;
        }

    | RESUMED types RPAREN EQUALS INT %prec NOFLAG { $$ = NewSyscall(-1, "tmp", $2, int64($5), false, true);
                                                        Stracelex.(*Stracelexer).result = $$ }
    | RESUMED types RPAREN EQUALS UINT %prec NOFLAG { $$ = NewSyscall(-1, "tmp", $2, int64($5), false, true);
                                                        Stracelex.(*Stracelexer).result = $$ }
    | RESUMED types RPAREN EQUALS QUESTION %prec NOFLAG { $$ = NewSyscall(-1, "tmp", $2, -1, false, true);
                                                        Stracelex.(*Stracelexer).result = $$ }
    | RESUMED types RPAREN EQUALS UINT LPAREN parentheticals RPAREN { $$ = NewSyscall(-1, "tmp", $2, int64($5), false, true);
                                                            Stracelex.(*Stracelexer).result = $$ }
    | RESUMED types RPAREN EQUALS INT LPAREN parentheticals RPAREN { $$ = NewSyscall(-1, "tmp", $2, $5, false, true);
                                                        Stracelex.(*Stracelexer).result = $$ }
    | RESUMED types RPAREN EQUALS UINT FLAG LPAREN parentheticals RPAREN { $$ = NewSyscall(-1, "tmp", $2, int64($5), false, true);
                                                            Stracelex.(*Stracelexer).result = $$ }
    | RESUMED types RPAREN EQUALS INT FLAG LPAREN parentheticals RPAREN { $$ = NewSyscall(-1, "tmp", $2, int64($5), false, true);
                                                            Stracelex.(*Stracelexer).result = $$ }

    | IDENTIFIER LPAREN types RPAREN EQUALS INT %prec NOFLAG{
                                                        $$ = NewSyscall(-1, $1, $3, $6, false, false);
                                                        Stracelex.(*Stracelexer).result = $$;}
    | IDENTIFIER LPAREN types RPAREN EQUALS UINT %prec NOFLAG {
                                                        $$ = NewSyscall(-1, $1, $3, int64($6), false, false);
                                                        Stracelex.(*Stracelexer).result = $$;}
    | IDENTIFIER LPAREN types RPAREN EQUALS QUESTION %prec NOFLAG {
                                                            $$ = NewSyscall(-1, $1, $3, -1, false, false);
                                                            Stracelex.(*Stracelexer).result = $$;}
    | IDENTIFIER LPAREN types RPAREN EQUALS INT FLAG LPAREN parentheticals RPAREN {
                                                              $$ = NewSyscall(-1, $1, $3, $6, false, false);
                                                              Stracelex.(*Stracelexer).result = $$;}
    | IDENTIFIER LPAREN types RPAREN EQUALS UINT FLAG LPAREN parentheticals RPAREN {
                                                              $$ = NewSyscall(-1, $1, $3, int64($6), false, false);
                                                              Stracelex.(*Stracelexer).result = $$;}
    | IDENTIFIER LPAREN types RPAREN EQUALS INT LPAREN parentheticals RPAREN {
                                                                  $$ = NewSyscall(-1, $1, $3, $6, false, false);
                                                                  Stracelex.(*Stracelexer).result = $$;}
    | IDENTIFIER LPAREN types RPAREN EQUALS UINT LPAREN parentheticals RPAREN {
                                                                  $$ = NewSyscall(-1, $1, $3, int64($6), false, false);
                                                                  Stracelex.(*Stracelexer).result = $$;}

    | INT syscall {call := $2; call.Pid = $1; Stracelex.(*Stracelexer).result = call}

parentheticals:
    parenthetical {$$ = newParenthetical();}
    | parentheticals parenthetical {$$ = newParenthetical();}

parenthetical:
    COMMA {$$=newParenthetical();}
    | OR {$$ = newParenthetical();}
    | AND {$$ = newParenthetical();}
    | LSHIFT {$$ = newParenthetical();}
    | RSHIFT {$$ = newParenthetical();}
    | IDENTIFIER {$$ = newParenthetical();}
    | group_type {$$ = newParenthetical();}
    | flag_type {$$ = newParenthetical();}
    | int_type {$$ = newParenthetical();}


types: {$$ = make([]IrType, 0)}
    | type {$$ = []IrType{$1}}
    | types COMMA type {$1 = append($1, $3); $$ = $1;}


type:
    buf_type {$$ = $1}
    | field_type {$$ = $1}
    | pointer_type {$$ = $1}
    | group_type {$$ = $1}
    | call_type {$$ = $1}
    | expr_type {$$ = $1}
    | expr_type ARROW type {$$ = newDynamicType($1, $3)}
    | ONESCOMP group_type {$$ = $2}


expr_type:
    flags {$$ = $1}
    | ints {$$ = $1}
    | macro_type {$$ = $1}
    | expr_type OR expr_type {$$ = newBinop($1, $3, orOp)}
    | expr_type AND expr_type {$$ = newBinop($1, $3, andOp)}
    | expr_type LSHIFT expr_type {$$ = newBinop($1, $3, lshiftOp)}
    | expr_type RSHIFT expr_type {$$ = newBinop($1, $3, rshiftOp)}
    | expr_type LOR expr_type {$$ = newBinop($1, $3, lorOp)}
    | expr_type LAND expr_type {$$ = newBinop($1, $3, landOp)}
    | expr_type LEQUAL expr_type {$$ = newBinop($1, $3, lequalOp)}
    | LPAREN expr_type RPAREN {$$ = $2}
    | expr_type TIMES expr_type {$$ = newBinop($1, $3, timesOp)}
    | ONESCOMP expr_type {$$ = newUnop($2, onescompOp)}

ints:
    int_type {i := make(Ints, 1); i[0] = $1; $$ = i}
    | ints int_type {$$ = append($1.(Ints), $2)}

flags:
    flag_type {f := make(Flags, 1); f[0] = $1; $$ = f}
    | flags flag_type {$$ = append($1.(Flags), $2)}

call_type:
    IDENTIFIER LPAREN types RPAREN {$$ = newCallType($1, $3)}

macro_type:
    FLAG LPAREN types RPAREN {$$ = newMacroType($1, $3)}
    | FLAG LPAREN identifiers RPAREN {$$ = newMacroType($1, nil)}
    | KEYWORD LPAREN KEYWORD IDENTIFIER RPAREN {$$ = newMacroType($4, nil)}

pointer_type:
    AND IDENTIFIER {$$ = nullPointer()}
    | AND UINT EQUALS type {$$ = NewPointerType($2, $4)}
    | NULL {$$ = nullPointer()}

group_type:
    LBRACKET_SQUARE types RBRACKET_SQUARE {$$ = newGroupType($2)}
    | LBRACKET types RBRACKET {$$ = newGroupType($2)}
    | LBRACKET types COMMA RBRACKET {$$ = newGroupType($2)}

field_type:
     IDENTIFIER EQUALS %prec NOTYPE {$$ = newField($1, nil);}
    | IDENTIFIER EQUALS type {$$ = newField($1, $3);}
    | IDENTIFIER COLON type {$$ = newField($1, $3);}
    | IDENTIFIER EQUALS AT type {$$ = newField($1, $4);}
    | IDENTIFIER LBRACKET_SQUARE FLAG RBRACKET_SQUARE EQUALS type {$$ = newField($1, $6)}

buf_type:
    STRING_LITERAL {$$ = newBufferType($1)}
    | DATETIME {$$ = newBufferType($1)}
    | MAC {$$ = newBufferType($1)}
    | IPV4 {$$ = newBufferType($1)}
    | IPV6 {$$ = newBufferType($1)}


int_type:
      INT {$$ = newIntType($1)}
      | UINT {$$ = newIntType(int64($1))}

flag_type:
      FLAG {$$ = newFlagType($1)}

identifiers:
    IDENTIFIER {ids := make([]*BufferType, 0); ids = append(ids, newBufferType($1)); $$ = ids}
    | IDENTIFIER identifiers {$2 = append($2, newBufferType($1)); $$ = $2}

