use crate::ast::{
    Ast, BinaryOperator, BinaryOperatorNode, BlockStatement, Expression, ExpressionNode, Literal,
    LiteralNode, LiteralValue, Statement, UnaryOperator, UnaryOperatorNode,
};

pub fn const_fold(ast: &mut Ast) {
    Folder::new().fold(ast)
}

struct Folder {}

impl Folder {
    fn new() -> Self {
        Folder {}
    }

    fn fold(self, ast: &mut Ast) {
        self.fold_block_statement(&mut ast.global);
    }

    fn fold_block_statement(&self, bs: &mut BlockStatement) {
        let mut last_assign = None;
        for statement in &mut bs.0 {
            last_assign = self.fold_statement(&mut statement.data, last_assign);
        }
    }

    fn fold_statement<'a>(
        &self,
        statement: &'a mut Statement,
        last_assign: Option<(&'a str, LiteralValue)>,
    ) -> Option<(&'a str, LiteralValue)> {
        match statement {
            Statement::Declaration {
                ident, initializer, ..
            } => initializer.as_mut().and_then(|initializer| {
                self.fold_expr_node(initializer, &last_assign)
                    .map(|v| (ident.data.as_str(), v))
            }),
            Statement::Assignment { ident, rhs } => self
                .fold_expr_node(rhs, &last_assign)
                .map(|v| (ident.data.as_str(), v)),
            Statement::Expression(expr_node) => {
                self.fold_expr_node(expr_node, &last_assign);
                last_assign
            }
            Statement::BlockStatement(bs) => {
                self.fold_block_statement(bs);
                None
            }
        }
    }

    fn fold_expr_node(
        &self,
        expr_node: &mut ExpressionNode,
        last_assign: &Option<(&str, LiteralValue)>,
    ) -> Option<LiteralValue> {
        if let Expression::Literal(ref value) = expr_node.data {
            // Literals don't need to be folded since they are already as folded as possible
            return Some(value.data.value.clone());
        }

        let folded = self.fold_expr(&mut expr_node.data, last_assign)?;
        replace_with_literal(expr_node, folded.clone());
        Some(folded)
    }

    fn fold_expr(
        &self,
        expr: &mut Expression,
        last_assign: &Option<(&str, LiteralValue)>,
    ) -> Option<LiteralValue> {
        match expr {
            Expression::Binary(lhs, op, rhs) => self.fold_binary_op(op, lhs, rhs, last_assign),
            Expression::Unary(op, expr) => self.fold_unary_op(op, expr, last_assign),
            Expression::Cast(_, expr_node) => {
                let inner_folded = self.fold_expr(&mut expr_node.data, last_assign)?;
                replace_with_literal(expr_node, inner_folded);
                None // Cast expression itself is not const-folded
            }
            Expression::Literal(lit) => Some(lit.data.value.clone()),
            Expression::Ident(ident) => last_assign
                .as_ref()
                .and_then(|(name, value)| (*name == ident.data).then(|| value.clone())),
        }
    }

    fn fold_binary_op(
        &self,
        op_node: &mut BinaryOperatorNode,
        lhs_node: &mut ExpressionNode,
        rhs_node: &mut ExpressionNode,
        last_assign: &Option<(&str, LiteralValue)>,
    ) -> Option<LiteralValue> {
        let folded1 = self.fold_expr(&mut lhs_node.data, last_assign)?;
        let folded2 = self.fold_expr(&mut rhs_node.data, last_assign)?;

        macro_rules! do_op_custom {
            (|$a:ident, $b:ident| $op_i:expr $(; $op_f:expr)?) => {{
                use LiteralValue::*;
                #[allow(unreachable_patterns)]
                match (&folded1, &folded2) {
                    (&Integer($a), &Integer($b)) => $op_i,
                $(
                    (&Integer($a), &Float($b)) => { let $a = $a as f64; $op_f }
                    (&Float($a), &Integer($b)) => { let $b = $b as f64; $op_f }
                    (&Float($a), &Float($b)) => $op_f,
                )?
                    _ => None
                }
            }};
        }

        macro_rules! do_op {
            (|$a:ident, $b:ident| $op:expr) => {
                do_op_custom!(|$a, $b| Some(Integer(($op) as i128)); Some(Float($op)))
            };
            (int; |$a:ident, $b:ident| $op:expr) => {
                do_op_custom!(|$a, $b| Some(Integer(($op) as i128)))
            };
            (bool; |$a:ident, $b:ident| $op_i:expr $(; $op_f:expr)?) => {
                do_op_custom!(
                    |$a, $b| Some(Integer(($op_i) as i128))
                    $(; Some(Integer(($op_f) as i128)))?
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
        last_assign: &Option<(&str, LiteralValue)>,
    ) -> Option<LiteralValue> {
        let inner_folded = self.fold_expr(&mut expr_node.data, last_assign)?;

        use LiteralValue::*;

        let folded = match op_node.data {
            UnaryOperator::Bang => Some(match inner_folded {
                Integer(i) => Integer((i == 0) as i128),
                Float(f) => Integer((f == 0.0) as i128),
            }),
            UnaryOperator::Plus => Some(inner_folded.clone()),
            UnaryOperator::Minus => Some(match inner_folded {
                Integer(i) => Integer(-i),
                Float(f) => Float(-f),
            }),
            UnaryOperator::DoublePlusPrefix => None,
            UnaryOperator::DoubleMinusPrefix => None,
            UnaryOperator::DoublePlusPostfix => None,
            UnaryOperator::DoubleMinusPostfix => None,
            UnaryOperator::Tilde => match inner_folded {
                Integer(i) => Some(Integer(!i)),
                Float(_) => None,
            },
            UnaryOperator::Ampersand => None,
            UnaryOperator::Star => None,
        };

        folded.or_else(|| {
            replace_with_literal(expr_node, inner_folded);
            None
        })
    }
}

fn replace_with_literal(expr_node: &mut ExpressionNode, value: LiteralValue) {
    expr_node.data = Expression::Literal(LiteralNode {
        span: expr_node.span,
        data: Literal { value },
    })
}
