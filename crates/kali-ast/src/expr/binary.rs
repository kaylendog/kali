//! Binary expressions.

use super::Expr;

/// An enumeration of binary operators.
#[derive(Debug, strum::Display, Clone, Copy, Eq, PartialEq)]
pub enum BinaryOp {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Subtract,
    #[strum(to_string = "*")]
    Multiply,
    #[strum(to_string = "/")]
    Divide,
    #[strum(to_string = "**")]
    Exponentiate,
    #[strum(to_string = "%")]
    Modulo,
    #[strum(to_string = "==")]
    Equal,
    #[strum(to_string = "!=")]
    NotEqual,
    #[strum(to_string = "<")]
    LessThan,
    #[strum(to_string = "<=")]
    LessThanOrEqual,
    #[strum(to_string = ">")]
    GreaterThan,
    #[strum(to_string = ">=")]
    GreaterThanOrEqual,
    #[strum(to_string = "&&")]
    LogicalAnd,
    #[strum(to_string = "||")]
    LogicalOr,
    #[strum(to_string = "&")]
    BitwiseAnd,
    #[strum(to_string = "|")]
    BitwiseOr,
    #[strum(to_string = "^")]
    BitwiseXor,
    #[strum(to_string = "<<")]
    BitwiseShiftLeft,
    #[strum(to_string = ">>")]
    BitwiseShiftRight,
    #[strum(to_string = "::")]
    Cons,
}

/// A binary expression.
#[derive(Debug, Clone, Eq, PartialEq)]
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
