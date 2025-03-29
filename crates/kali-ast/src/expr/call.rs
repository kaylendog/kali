//! Call expressions.

use super::Expr;

#[derive(Clone, Debug)]
pub struct Call<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The function being called.
    pub fun: Box<Expr<Meta>>,
    pub args: Vec<Expr<Meta>>,
}
