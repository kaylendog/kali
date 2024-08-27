//! Pattern matching.

use std::collections::BTreeMap;

use kali_type::{Context, Type, TypeInferenceError, Typed, TypedIterator};

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
    /// A list pattern.
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

impl Typed for Match {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.branches.values().fold_unify(context)
    }
}
