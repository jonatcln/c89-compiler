parser grammar Expr;
import Type;

expr
    : value=assignExpr
    ;

assignExpr
    : value=condExpr                                            # AssignExprSingular
    | lhs=unaryExpr op=EQUALS rhs=assignExpr                    # AssignExprComposed
    ;

condExpr
    : value=logicalOrExpr                                       # CondExprSingular
    | cond=logicalOrExpr
      QUESTION_MARK if_br=expr
      COLON else_br=condExpr                                    # CondExprTernary
    ;

logicalOrExpr
    : value=logicalAndExpr                                      # LogicalOrExprSingular
    | lhs=logicalOrExpr op=DOUBLE_PIPE rhs=logicalAndExpr       # LogicalOrExprComposed
    ;

logicalAndExpr
    : value=bitwiseOrExpr                                       # LogicalAndExprSingular
    | lhs=logicalAndExpr op=DOUBLE_AMPERSAND rhs=bitwiseOrExpr  # LogicalAndExprComposed
    ;

bitwiseOrExpr
    : value=bitwiseXorExpr                                      # BitwiseOrExprSingular
    | lhs=bitwiseOrExpr op=PIPE rhs=bitwiseXorExpr              # BitwiseOrExprComposed
    ;

bitwiseXorExpr
    : value=bitwiseAndExpr                                      # BitwiseXorExprSingular
    | lhs=bitwiseXorExpr op=CARET rhs=bitwiseAndExpr            # BitwiseXorExprComposed
    ;


bitwiseAndExpr
    : value=equalityExpr                                        # BitwiseAndExprSingular
    | lhs=bitwiseAndExpr op=AMPERSAND rhs=equalityExpr          # BitwiseAndExprComposed
    ;

equalityExpr
    : value=inequalityExpr                                      # EqualityExprSingular
    | lhs=equalityExpr
      op=(DOUBLE_EQUALS | BANG_EQUALS)
      rhs=inequalityExpr                                        # EqualityExprComposed
    ;

inequalityExpr
    : value=shiftExpr                                           # InequalityExprSingular
    | lhs=inequalityExpr
      op=(ANGLE_LEFT_EQUALS | ANGLE_RIGHT_EQUALS | ANGLE_LEFT | ANGLE_RIGHT)
      rhs=shiftExpr                                             # InequalityExprComposed
    ;

shiftExpr
    : value=arithExpr                                           # ShiftExprSingular
    | lhs=shiftExpr op=(DOUBLE_ANGLE_LEFT | DOUBLE_ANGLE_RIGHT)
      rhs=arithExpr                                             # ShiftExprComposed
    ;

arithExpr
    : value=termExpr                                            # ArithExprSingular
    | lhs=arithExpr op=(PLUS | MINUS) rhs=termExpr              # ArithExprComposed
    ;

termExpr
    : value=castExpr                                            # TermExprSingular
    | lhs=termExpr op=(STAR | SLASH | PERCENT) rhs=castExpr     # TermExprComposed
    ;

castExpr
    : value=unaryExpr                                           # CastExprSingular
    | PAREN_LEFT type_name=typeName PAREN_RIGHT value=castExpr  # CastExprComposed
    ;

unaryExpr
    : value=postfixExpr                                         # UnaryExprPostfix
    | op=(DOUBLE_PLUS | DOUBLE_MINUS | BANG | PLUS | MINUS | AMPERSAND | STAR | TILDE)
      value=castExpr                                            # UnaryExprPrefix
    ;

postfixExpr
    : value=primaryExpr                                         # PostfixExprPrimary
    | ident=identifier PAREN_LEFT
      (args+=expr)? (COMMA args+=expr)* PAREN_RIGHT             # PostfixExprFunctionCall
    | value=postfixExpr BRACKET_LEFT rhs=expr BRACKET_RIGHT     # PostfixExprArraySubscript
    | value=postfixExpr op=(DOUBLE_PLUS | DOUBLE_MINUS)         # PostfixExprPostfix
    ;

primaryExpr
    : PAREN_LEFT inner=expr PAREN_RIGHT                         # PrimaryExprWrapped
    | value=literal                                             # PrimaryExprLiteral
    | ident=identifier                                          # PrimaryExprIdentifier
    ;

literal
    : value=CHAR_LITERAL                                        # LiteralChar
    | (value+=STRING_LITERAL)+                                  # LiteralString
    | value=FLOATING_POINT_LITERAL                              # LiteralFloatingPoint
    | value=integerLiteral                                      # LiteralInteger
    ;

integerLiteral
    : value=DECIMAL_LITERAL                                     # IntegerLiteralDecimal
    | value=OCTAL_LITERAL                                       # IntegerLiteralOctal
    | value=HEXADECIMAL_LITERAL                                 # IntegerLiteralHexadecimal
    ;

