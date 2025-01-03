use indexmap::IndexMap;
use kali_type::{Context, Type, TypeInferenceError, Typed, TypedIterator};

use crate::{Node, Pattern};

use super::Expr;

/// A match expression.
#[derive(Debug, Clone)]
pub struct Match<Meta = ()> {
    /// THe expression to test.
    pub expr: Box<Node<Expr<Meta>, Meta>>,
    /// Branches of the match expression.
    pub branches: IndexMap<Pattern, Node<Expr<Meta>, Meta>>,
}

impl Typed for Match {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.branches.values().fold_unify(context)
    }
}
