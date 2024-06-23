//! Pattern matching.

use std::collections::BTreeMap;

use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

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
    pub branches: BTreeMap<Pattern, Expr>,
}

impl Typed for Match {
    fn ty(&self, context: &mut InferenceContext) -> Result<Type, TypeInferenceError> {
        let branches = Type::ty_all(self.branches.values(), context)?;
        Type::unify_all(&branches, context)
            .map_err(|e| TypeInferenceError::UnificationFailed(branches, e))
    }
}
