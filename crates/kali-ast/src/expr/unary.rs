//! Unary expressions.

use crate::Expr;

/// A unary expression.
#[derive(Debug, Clone)]
pub struct UnaryExpr<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The unary operator.
    pub operator: UnaryOp,
    /// The inner expression.
    pub inner: Box<Expr<Meta>>,
}

/// An enumeration of unary operators.
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    /// The negation operator.
    Negate,
    /// The logical not operator.
    LogicalNot,
    /// The bitwise not operator.
    BitwiseNot,
}
