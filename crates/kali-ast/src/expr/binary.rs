//! Binary expressions.

use super::Expr;

/// An enumeration of binary operators.
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiate,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    Cons,
}

/// A binary expression.
#[derive(Debug, Clone)]
pub struct BinaryExpr<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The left-hand side of the expression.
    pub lhs: Box<Expr<Meta>>,
    /// The right-hand side of the expression.
    pub rhs: Box<Expr<Meta>>,
    /// The binary operator.
    pub operator: BinaryOp,
}
