//! Pattern matching.

use std::collections::BTreeMap;

use crate::Expr;

/// Represents a pattern in the AST.
pub enum Pattern {
    /// A wildcard pattern.
    Wildcard,
    /// An identifier pattern.
    Identifier(String),
    /// A literal pattern.
    Tuple(Vec<Pattern>),
    /// A struct pattern.
    Record(Vec<(String, Pattern)>),
    List(Vec<Pattern>),
    /// A tuple pattern.
    Cons(Box<Pattern>, Box<Pattern>),
    /// A range pattern.
    Range(Box<Pattern>, Box<Pattern>),
}

/// A match expression.
pub struct Match {
    /// Branches of the match expression.
    pub branches: BTreeMap<Pattern, Expr>,
}
