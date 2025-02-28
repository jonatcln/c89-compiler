use super::{escape_string_literal, DotTree, ToDot};
use crate::ast;
use std::iter;

impl ToDot for ast::Ast {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "translation unit".to_owned(),
            self.global_declarations
                .iter()
                .map(|d| ("ext decl", d.data.to_dot()))
                .collect(),
        )
    }
}

impl ToDot for ast::ExternalDeclaration {
    fn to_dot(&self) -> DotTree {
        match self {
            ast::ExternalDeclaration::FunctionDefinition(def) => def.to_dot(),
            ast::ExternalDeclaration::Declaration(decl) => decl.to_dot(),
        }
    }
}

impl ToDot for ast::Declaration {
    fn to_dot(&self) -> DotTree {
        match self {
            ast::Declaration::Variable(decl) => decl.to_dot(),
            ast::Declaration::FunctionDeclaration(decl) => decl.to_dot(),
        }
    }
}

impl ToDot for ast::VariableDeclaration {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "var decl".to_owned(),
            vec![
                ("type", self.type_name.to_dot()),
                ("ident", to_dot_ident(&self.ident.data)),
            ]
            .into_iter()
            .chain(
                self.array_parts
                    .iter()
                    .map(|array| ("array", array.data.to_dot())),
            )
            .chain(
                self.initializer
                    .iter()
                    .map(|initializer| ("rhs", initializer.1.data.to_dot())),
            )
            .collect(),
        )
    }
}

impl ToDot for ast::ArrayDeclaration {
    fn to_dot(&self) -> DotTree {
        match self {
            ast::ArrayDeclaration::Unknown => DotTree::new_leaf("unknown len".to_owned()),
            ast::ArrayDeclaration::Known(expr) => {
                DotTree::new("known len".to_owned(), vec![("expr", expr.data.to_dot())])
            }
        }
    }
}

impl ToDot for ast::FunctionDeclaration {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "fn decl".to_owned(),
            vec![
                ("ret type", self.return_type.to_dot()),
                ("ident", to_dot_ident(&self.ident.data)),
            ]
            .into_iter()
            .chain(self.params.iter().map(|p| ("", p.to_dot())))
            .chain(
                self.is_vararg
                    .then_some(("vararg", DotTree::new_leaf("...".to_owned()))),
            )
            .collect(),
        )
    }
}

impl ToDot for ast::FunctionParamNode {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "param".to_owned(),
            iter::once(("type", self.type_name.to_dot()))
                .chain(
                    self.ident
                        .iter()
                        .map(|ident| ("ident", to_dot_ident(&ident.data))),
                )
                .collect(),
        )
    }
}

impl ToDot for ast::FunctionDefinition {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "fn def".to_owned(),
            vec![
                ("return type", self.return_type.to_dot()),
                ("ident", to_dot_ident(&self.ident.data)),
            ]
            .into_iter()
            .chain(self.params.iter().map(|p| ("", p.to_dot())))
            .chain(
                self.is_vararg
                    .then_some(("vararg", DotTree::new_leaf("...".to_owned()))),
            )
            .chain(iter::once(("body", self.body.to_dot())))
            .collect(),
        )
    }
}

impl ToDot for ast::BlockStatementNode {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "block".to_owned(),
            self.stmts
                .iter()
                .map(|s| ("stmt", s.data.to_dot()))
                .collect(),
        )
    }
}

impl ToDot for ast::Statement {
    fn to_dot(&self) -> DotTree {
        match self {
            ast::Statement::Declaration(decl) => decl.to_dot(),
            ast::Statement::Expression(e) => e.data.to_dot(),
            ast::Statement::If(i) => i.to_dot(),
            ast::Statement::Switch(i) => i.to_dot(),
            ast::Statement::While(i) => i.to_dot(),
            ast::Statement::For(i) => i.to_dot(),
            ast::Statement::Break => DotTree::new_leaf("break".to_owned()),
            ast::Statement::Continue => DotTree::new_leaf("continue".to_owned()),
            ast::Statement::Return(_, expr) => DotTree::new(
                "return".to_owned(),
                expr.iter()
                    .map(|expr| ("expr", expr.data.to_dot()))
                    .collect(),
            ),
            ast::Statement::BlockStatement(bs) => bs.to_dot(),
        }
    }
}

impl ToDot for ast::IfStatement {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "if".to_owned(),
            [
                ("cond", self.condition.data.to_dot()),
                ("if body", self.if_body.to_dot()),
            ]
            .into_iter()
            .chain(self.else_body.as_ref().map(|b| ("else body", b.to_dot())))
            .collect(),
        )
    }
}

impl ToDot for ast::SwitchStatement {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "switch".to_owned(),
            iter::once(("expr", self.expr.data.to_dot()))
                .chain(self.cases.iter().map(|case| ("case", case.to_dot())))
                .collect(),
        )
    }
}

impl ToDot for ast::SwitchCase {
    fn to_dot(&self) -> DotTree {
        match self {
            ast::SwitchCase::Expr(case) => DotTree::new(
                "expr".to_owned(),
                vec![
                    ("expr", case.expr.data.to_dot()),
                    ("body", case.body.to_dot()),
                ],
            ),
            ast::SwitchCase::Default(case) => {
                DotTree::new("default".to_owned(), vec![("body", case.body.to_dot())])
            }
        }
    }
}

impl ToDot for ast::WhileStatement {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "while".to_owned(),
            vec![
                ("cond", self.condition.data.to_dot()),
                ("body", self.body.to_dot()),
            ],
        )
    }
}

impl ToDot for ast::ForStatement {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "for".to_owned(),
            self.init
                .iter()
                .map(|init| ("init", init.data.to_dot()))
                .chain(
                    self.condition
                        .as_ref()
                        .map(|condition| ("cond", condition.data.to_dot())),
                )
                .chain(self.iter.as_ref().map(|iter| ("iter", iter.data.to_dot())))
                .chain(iter::once(("body", self.body.to_dot())))
                .collect(),
        )
    }
}

impl ToDot for ast::Expression {
    fn to_dot(&self) -> DotTree {
        use ast::{BinaryOperator, UnaryOperator};
        match self {
            Self::Assignment(e1, _, e2) => DotTree::new(
                "=".to_owned(),
                vec![("lhs", e1.data.to_dot()), ("rhs", e2.data.to_dot())],
            ),
            Self::Binary(e1, bo, e2) => DotTree::new(
                match bo.data {
                    BinaryOperator::Plus => "+",
                    BinaryOperator::Minus => "-",
                    BinaryOperator::Star => "*",
                    BinaryOperator::Slash => "/",
                    BinaryOperator::Pipe => "|",
                    BinaryOperator::Caret => "^",
                    BinaryOperator::Ampersand => "&",
                    BinaryOperator::AngleLeft => "<",
                    BinaryOperator::AngleRight => ">",
                    BinaryOperator::DoubleEquals => "==",
                    BinaryOperator::DoubleAmpersand => "&&",
                    BinaryOperator::DoublePipe => "||",
                    BinaryOperator::BangEquals => "!=",
                    BinaryOperator::Percent => "%",
                    BinaryOperator::AngleLeftEquals => "<=",
                    BinaryOperator::AngleRightEquals => ">=",
                    BinaryOperator::DoubleAngleLeft => "<<",
                    BinaryOperator::DoubleAngleRight => ">>",
                }
                .to_owned(),
                vec![("lhs", e1.data.to_dot()), ("rhs", e2.data.to_dot())],
            ),
            Self::ArraySubscript(lhs, rhs) => DotTree::new(
                "◌[◌]".to_owned(),
                vec![("lhs", lhs.data.to_dot()), ("rhs", rhs.data.to_dot())],
            ),
            Self::Unary(uo, e) => DotTree::new(
                match &uo.data {
                    UnaryOperator::Bang => "!◌",
                    UnaryOperator::Plus => "+◌",
                    UnaryOperator::Minus => "-◌",
                    UnaryOperator::Tilde => "~◌",
                    UnaryOperator::Ampersand => "&◌",
                    UnaryOperator::Star => "*◌",
                    UnaryOperator::DoublePlusPrefix => "++◌",
                    UnaryOperator::DoubleMinusPrefix => "--◌",
                    UnaryOperator::DoublePlusPostfix => "◌++",
                    UnaryOperator::DoubleMinusPostfix => "◌--",
                }
                .to_owned(),
                vec![("", e.data.to_dot())],
            ),
            Self::Cast(t, e) => DotTree::new(
                "cast".to_owned(),
                vec![("type", t.to_dot()), ("expr", e.data.to_dot())],
            ),
            Self::FunctionCall(fc) => fc.to_dot(),
            Self::Ident(i) => to_dot_ident(&i.data),
            Self::Literal(lit) => lit.data.to_dot(),
        }
    }
}

impl ToDot for ast::FunctionCall {
    fn to_dot(&self) -> DotTree {
        DotTree::new(
            "func call".to_owned(),
            iter::once(("name", to_dot_ident(&self.ident.data)))
                .chain(self.args.iter().map(|arg| ("arg", arg.data.to_dot())))
                .collect(),
        )
    }
}

impl ToDot for ast::Literal {
    fn to_dot(&self) -> DotTree {
        let (name, value) = match self {
            ast::Literal::Dec(i) => ("dec", i.to_string()),
            ast::Literal::Hex(i) => ("hex", i.to_string()),
            ast::Literal::Octal(i) => ("octal", i.to_string()),
            ast::Literal::Char(i) => ("char", i.to_string()),
            ast::Literal::Float(f) => ("float", f.to_string()),
            ast::Literal::String(s) => ("string", escape_string_literal(s)),
        };
        DotTree::new("literal".to_owned(), vec![(name, DotTree::new_leaf(value))])
    }
}

impl ToDot for ast::QualifiedTypeNode {
    fn to_dot(&self) -> DotTree {
        let mut children = Vec::new();
        if self.is_const.is_some() {
            children.push(("", DotTree::new_leaf("const".to_owned())));
        }
        children.push(("unqualified", self.unqualified.data.to_dot()));
        DotTree::new("q type".to_owned(), children)
    }
}

impl ToDot for ast::UnqualifiedType {
    fn to_dot(&self) -> DotTree {
        match self {
            Self::PointerType(ty) => DotTree::new("pointer".to_owned(), vec![("to", ty.to_dot())]),
            Self::Void => DotTree::new_leaf("void".to_owned()),
            Self::Float => DotTree::new_leaf("float".to_owned()),
            Self::Double => DotTree::new_leaf("double".to_owned()),
            Self::LongDouble => DotTree::new_leaf("long double".to_owned()),
            Self::Char => DotTree::new_leaf("char".to_owned()),
            Self::SignedChar => DotTree::new_leaf("signed char".to_owned()),
            Self::UnsignedChar => DotTree::new_leaf("unsigned char".to_owned()),
            Self::SignedShortInt => DotTree::new_leaf("short int".to_owned()),
            Self::SignedInt => DotTree::new_leaf("int".to_owned()),
            Self::SignedLongInt => DotTree::new_leaf("long int".to_owned()),
            Self::UnsignedShortInt => DotTree::new_leaf("unsigned short int".to_owned()),
            Self::UnsignedInt => DotTree::new_leaf("unsigned int".to_owned()),
            Self::UnsignedLongInt => DotTree::new_leaf("unsigned long int".to_owned()),
        }
    }
}

fn to_dot_ident(i: &str) -> DotTree {
    DotTree::new_leaf(i.to_owned())
}
