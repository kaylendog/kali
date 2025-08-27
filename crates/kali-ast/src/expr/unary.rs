//! Unary expressions.

use crate::Expr;

/// A unary expression.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnaryExpr<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The unary operator.
    pub operator: UnaryOp,
    /// The inner expression.
    pub inner: Box<Expr<Meta>>,
}

/// An enumeration of unary operators.
#[derive(Debug, strum::Display, Clone, Copy, Eq, PartialEq)]
pub enum UnaryOp {
    /// The negation operator.
    #[strum(to_string = "-")]
    Negate,
    /// The logical not operator.
    #[strum(to_string = "!")]
    LogicalNot,
    /// The bitwise not operator.
    #[strum(to_string = "~")]
    BitwiseNot,
}
