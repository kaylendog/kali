//! Call expressions.

use super::Expr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Call<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The function being called.
    pub fun: Box<Expr<Meta>>,
    pub args: Vec<Expr<Meta>>,
}
