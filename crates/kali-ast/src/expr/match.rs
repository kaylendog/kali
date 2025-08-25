use std::collections::HashMap;

use crate::Pattern;

use super::Expr;

/// A match expression.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Match<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// THe expression to test.
    pub expr: Box<Expr<Meta>>,
    /// Branches of the match expression.
    pub branches: HashMap<Pattern<Meta>, Expr<Meta>>,
}
