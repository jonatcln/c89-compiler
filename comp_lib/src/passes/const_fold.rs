use crate::ast::{
    ArrayDeclaration, Ast, BinaryOperator, BinaryOperatorNode, BlockStatementNode, Declaration,
    Expression, ExpressionNode, ExternalDeclaration, FunctionDefinition, Literal, LiteralNode,
    Statement, SwitchStatement, UnaryOperator, UnaryOperatorNode, VariableDeclaration,
};

pub fn const_fold(ast: &mut Ast) {
    Folder::new().fold(ast)
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Int(i128),
    Float(f64),
}

struct Folder {}

impl Folder {
    fn new() -> Self {
        Folder {}
    }

    fn fold(self, ast: &mut Ast) {
        for external_decl in &mut ast.global_declarations {
            self.fold_external_declaration(&mut external_decl.data);
        }
    }

    fn fold_external_declaration(&self, exdecl: &mut ExternalDeclaration) {
        match exdecl {
            ExternalDeclaration::FunctionDefinition(FunctionDefinition { body, .. }) => {
                self.fold_block_statement(body);
            }
            ExternalDeclaration::Declaration(decl) => {
                self.fold_declaration(decl, &None);
            }
        }
    }

    fn fold_declaration<'a>(
        &self,
        declaration: &'a mut Declaration,
        last_assign: &Option<(&'a str, Value)>,
    ) -> Option<(&'a str, Value)> {
        match declaration {
            Declaration::Variable(VariableDeclaration {
                ident,
                initializer,
                array_parts,
                ..
            }) => {
                let res = initializer.as_mut().and_then(|initializer| {
                    self.fold_expr_node(&mut initializer.1, last_assign)
                        .map(|v| (ident.data.as_str(), v))
                });
                if !array_parts.is_empty() {
                    for array_part in array_parts {
                        if let ArrayDeclaration::Known(expr) = &mut array_part.data {
                            self.fold_expr_node(expr, last_assign);
                        }
                    }
                    return None;
                }
                res
            }
            Declaration::FunctionDeclaration(_) => None,
        }
    }

    fn fold_block_statement(&self, bs: &mut BlockStatementNode) {
        let mut last_assign = None;
        for statement in &mut bs.stmts {
            last_assign = self.fold_statement(&mut statement.data, last_assign);
        }
    }

    fn fold_statement<'a>(
        &self,
        statement: &'a mut Statement,
        last_assign: Option<(&'a str, Value)>,
    ) -> Option<(&'a str, Value)> {
        match statement {
            Statement::Declaration(decl) => return self.fold_declaration(decl, &last_assign),
            Statement::Expression(expr_node) => {
                self.fold_expr_node(expr_node, &last_assign);
            }
            Statement::If(i) => {
                self.fold_expr_node(&mut i.condition, &None);
                self.fold_block_statement(&mut i.if_body);
                if let Some(else_body) = &mut i.else_body {
                    self.fold_block_statement(else_body);
                }
            }
            Statement::Switch(i) => {
                self.fold_switch(i);
            }
            Statement::While(i) => {
                self.fold_expr_node(&mut i.condition, &None);
                self.fold_block_statement(&mut i.body);
            }
            Statement::For(i) => {
                if let Some(init) = &mut i.init {
                    self.fold_statement(&mut init.data, None);
                }
                if let Some(condition) = &mut i.condition {
                    self.fold_expr_node(condition, &None);
                }
                if let Some(iter) = &mut i.iter {
                    self.fold_expr_node(iter, &None);
                }

                self.fold_block_statement(&mut i.body);
            }
            Statement::Break => {}
            Statement::Continue => {}
            Statement::Return(_, Some(expr_node)) => {
                self.fold_expr_node(expr_node, &last_assign);
            }
            Statement::Return(_, None) => {}
            Statement::BlockStatement(bs) => {
                self.fold_block_statement(bs);
            }
        }
        None
    }

    fn fold_switch(&self, switch: &mut SwitchStatement) {
        for case in &mut switch.cases {
            let body = match case {
                crate::ast::SwitchCase::Expr(case) => {
                    self.fold_expr(&mut case.expr.data, &None);
                    &mut case.body
                }
                crate::ast::SwitchCase::Default(case) => &mut case.body,
            };

            self.fold_block_statement(body);
        }
    }

    fn fold_expr_node(
        &self,
        expr_node: &mut ExpressionNode,
        last_assign: &Option<(&str, Value)>,
    ) -> Option<Value> {
        if let Expression::Literal(ref lit) = expr_node.data {
            // Literals don't need to be folded since they are already as folded as possible
            return self.fold_literal(&lit.data);
        }

        let folded = self.fold_expr(&mut expr_node.data, last_assign)?;
        replace_with_literal(expr_node, folded);
        Some(folded)
    }

    fn fold_expr(
        &self,
        expr: &mut Expression,
        last_assign: &Option<(&str, Value)>,
    ) -> Option<Value> {
        match expr {
            Expression::Assignment(_, _, rhs) => {
                let folded = self.fold_expr(&mut rhs.data, last_assign)?;
                replace_with_literal(rhs, folded);
                None // Assignment expression itself is not const-folded
            }
            Expression::Binary(lhs, op, rhs) => self.fold_binary_op(op, lhs, rhs, last_assign),
            Expression::ArraySubscript(lhs, rhs) => {
                if let Some(folded) = self.fold_expr(&mut lhs.data, last_assign) {
                    replace_with_literal(lhs, folded);
                }
                if let Some(folded) = self.fold_expr(&mut rhs.data, last_assign) {
                    replace_with_literal(rhs, folded);
                }
                None // ArraySubscript expression itself is not const-folded
            }
            Expression::Unary(op, expr) => self.fold_unary_op(op, expr, last_assign),
            Expression::Cast(_, expr_node) => {
                let inner_folded = self.fold_expr(&mut expr_node.data, last_assign)?;
                replace_with_literal(expr_node, inner_folded);
                None // Cast expression itself is not const-folded
            }
            Expression::FunctionCall(fc) => {
                for arg in &mut fc.args {
                    if let Some(folded) = self.fold_expr(&mut arg.data, last_assign) {
                        replace_with_literal(arg, folded);
                    }
                }
                None // Function call expression itself is not const-folded
            }
            // This case should be unreachable, since it is handled in fold_expr_node already.
            Expression::Literal(lit) => self.fold_literal(&lit.data),
            Expression::Ident(ident) => last_assign
                .as_ref()
                .and_then(|(name, value)| (*name == ident.data).then_some(*value)),
        }
    }

    fn fold_binary_op(
        &self,
        op_node: &mut BinaryOperatorNode,
        lhs_node: &mut ExpressionNode,
        rhs_node: &mut ExpressionNode,
        last_assign: &Option<(&str, Value)>,
    ) -> Option<Value> {
        let folded1 = self.fold_expr(&mut lhs_node.data, last_assign)?;
        let folded2 = self.fold_expr(&mut rhs_node.data, last_assign)?;

        macro_rules! do_op_custom {
            (|$a:ident, $b:ident| $op_i:expr $(; $op_f:expr)?) => {{
                use Value::*;
                #[allow(unreachable_patterns)]
                match (&folded1, &folded2) {
                    (&Int($a), &Int($b)) => $op_i,
                $(
                    (&Int($a), &Float($b)) => { let $a = $a as f64; $op_f }
                    (&Float($a), &Int($b)) => { let $b = $b as f64; $op_f }
                    (&Float($a), &Float($b)) => $op_f,
                )?
                    _ => None
                }
            }};
        }

        macro_rules! do_op {
            (|$a:ident, $b:ident| $op:expr) => {
                do_op_custom!(|$a, $b| Some(Int(($op) as i128)); Some(Float($op)))
            };
            (int; |$a:ident, $b:ident| $op:expr) => {
                do_op_custom!(|$a, $b| Some(Int(($op) as i128)))
            };
            (bool; |$a:ident, $b:ident| $op_i:expr $(; $op_f:expr)?) => {
                do_op_custom!(
                    |$a, $b| Some(Int(($op_i) as i128))
                    $(; Some(Int(($op_f) as i128)))?
                )
            };
        }

        let folded = match op_node.data {
            BinaryOperator::Plus => do_op!(|a, b| a + b),
            BinaryOperator::Minus => do_op!(|a, b| a - b),
            BinaryOperator::Star => do_op!(|a, b| a * b),
            BinaryOperator::Slash => do_op!(|a, b| a / b),
            BinaryOperator::Pipe => do_op!(int; |a, b| a | b),
            BinaryOperator::Caret => do_op!(int; |a, b| a ^ b),
            BinaryOperator::Ampersand => do_op!(int; |a, b| a & b),
            BinaryOperator::AngleLeft => do_op!(bool; |a, b| a < b),
            BinaryOperator::AngleRight => do_op!(bool; |a, b| a > b),
            BinaryOperator::DoubleEquals => do_op!(bool; |a, b| a == b),
            BinaryOperator::DoubleAmpersand => {
                do_op!(bool; |a, b| (a != 0 && b != 0); (a != 0.0 && b != 0.0))
            }
            BinaryOperator::DoublePipe => {
                do_op!(bool; |a, b| (a != 0 || b != 0); (a != 0.0 || b != 0.0))
            }
            BinaryOperator::BangEquals => do_op!(bool; |a, b| a != b),
            BinaryOperator::Percent => do_op!(int; |a, b| a % b),
            BinaryOperator::AngleLeftEquals => do_op!(bool; |a, b| a <= b),
            BinaryOperator::AngleRightEquals => do_op!(bool; |a, b| a >= b),
            BinaryOperator::DoubleAngleLeft => do_op!(int; |a, b| a << b),
            BinaryOperator::DoubleAngleRight => None,
        };

        folded.or_else(|| {
            replace_with_literal(lhs_node, folded1);
            replace_with_literal(rhs_node, folded2);
            None
        })
    }

    fn fold_unary_op(
        &self,
        op_node: &mut UnaryOperatorNode,
        expr_node: &mut ExpressionNode,
        last_assign: &Option<(&str, Value)>,
    ) -> Option<Value> {
        let inner_folded = self.fold_expr(&mut expr_node.data, last_assign)?;

        use Value::*;

        let folded = match op_node.data {
            UnaryOperator::Bang => Some(match inner_folded {
                Int(i) => Int((i == 0) as i128),
                Float(f) => Int((f == 0.0) as i128),
            }),
            UnaryOperator::Plus => Some(inner_folded),
            UnaryOperator::Minus => Some(match inner_folded {
                Int(i) => Int(-i),
                Float(f) => Float(-f),
            }),
            UnaryOperator::Star => None,
            UnaryOperator::Tilde => match inner_folded {
                Int(i) => Some(Int(!i)),
                Float(_) => None,
            },
            UnaryOperator::DoublePlusPrefix
            | UnaryOperator::DoubleMinusPrefix
            | UnaryOperator::DoublePlusPostfix
            | UnaryOperator::DoubleMinusPostfix
            | UnaryOperator::Ampersand => {
                // Early return to avoid folding a lvalue
                return None;
            }
        };

        folded.or_else(|| {
            replace_with_literal(expr_node, inner_folded);
            None
        })
    }

    fn fold_literal(&self, literal: &Literal) -> Option<Value> {
        match literal {
            Literal::Dec(i) | Literal::Hex(i) | Literal::Octal(i) => Some(Value::Int(*i)),
            Literal::Char(i) => Some(Value::Int(*i as i128)),
            Literal::Float(f) => Some(Value::Float(*f)),
            Literal::String(_) => None,
        }
    }
}

fn replace_with_literal(expr_node: &mut ExpressionNode, value: Value) {
    let lit = match value {
        Value::Int(i) => Literal::Dec(i),
        Value::Float(f) => Literal::Float(f),
    };
    expr_node.data = Expression::Literal(LiteralNode {
        span: expr_node.span,
        data: lit,
    })
}
