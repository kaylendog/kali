//! Conditional expressions.

use super::Expr;

/// A conditional expression.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Conditional<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The condition to check.
    pub condition: Box<Expr<Meta>>,
    /// The body of the conditional.
    pub body: Box<Expr<Meta>>,
    /// The body of the else branch.
    pub otherwise: Box<Expr<Meta>>,
}
