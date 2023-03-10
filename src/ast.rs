#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Ast {
    pub global: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct BlockStatement(pub Vec<StatementNode>);

#[derive(Debug, Clone)]
pub struct StatementNode {
    pub span: Span,
    pub data: Statement,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Declaration {
        type_name: QualifiedTypeNode,
        ident: IdentNode,
        initializer: Option<ExpressionNode>,
    },
    Assignment {
        ident: IdentNode,
        rhs: ExpressionNode,
    },
    Expression(ExpressionNode),
    BlockStatement(BlockStatement),
}

#[derive(Debug, Clone)]
pub struct QualifiedTypeNode {
    pub span: Span,
    pub data: QualifiedType,
}

#[derive(Debug, Clone)]
pub struct QualifiedType {
    pub is_const: Option<Span>,
    // pub is_volitile: bool,
    // pub is_restrict: bool,
    pub inner: UnqualifiedTypeNode,
}

#[derive(Debug, Clone)]
pub struct UnqualifiedTypeNode {
    pub span: Span,
    pub data: UnqualifiedType,
}

#[derive(Debug, Clone)]
pub enum UnqualifiedType {
    PointerType(Box<QualifiedTypeNode>),
    // ArrayType,
    // FunctionType,
    PlainType(PlainType),
}

#[derive(Debug, Clone)]
pub enum PlainType {
    Primitive(PrimitiveType),
    // StructType(String),
    // EnumType(String),
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Char,
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub struct ExpressionNode {
    pub span: Span,
    pub data: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<ExpressionNode>, BinaryOperatorNode, Box<ExpressionNode>),
    Unary(UnaryOperatorNode, Box<ExpressionNode>),
    Cast(QualifiedTypeNode, Box<ExpressionNode>),
    Literal(LiteralNode),
    Ident(IdentNode),
}

#[derive(Debug, Clone)]
pub struct LiteralNode {
    pub span: Span,
    pub data: Literal,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralValue,
    // pub t: Type,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Integer(i128), //TODO change this to big int?
    Float(f64),
    // Void,
}

#[derive(Debug, Clone)]
pub struct BinaryOperatorNode {
    pub span: Span,
    pub data: BinaryOperator,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Pipe,
    Caret,
    Ampersand,
    AngleLeft,
    AngleRight,
    DoubleEquals,
    DoubleAmpersand,
    DoublePipe,
    BangEquals,
    Percent,
    AngleLeftEquals,
    AngleRightEquals,
}

#[derive(Debug, Clone)]
pub struct UnaryOperatorNode {
    pub span: Span,
    pub data: UnaryOperator,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Bang,
    Plus,
    Minus,
    DoublePlusPrefix,
    DoubleMinusPrefix,
    DoublePlusPostfix,
    DoubleMinusPostfix,
    Tilde,
    Ampersand,
    Star,
}

#[derive(Debug, Clone)]
pub struct IdentNode {
    pub span: Span,
    pub data: String,
}
