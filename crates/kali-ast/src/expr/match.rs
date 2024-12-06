use indexmap::IndexMap;
use kali_type::{Context, Type, TypeInferenceError, Typed, TypedIterator};

use crate::{Node, Pattern};

use super::Expr;

/// A match expression.
#[derive(Debug, Clone)]
pub struct Match {
    /// THe expression to test.
    pub expr: Box<Node<Expr>>,
    /// Branches of the match expression.
    pub branches: IndexMap<Pattern, Node<Expr>>,
}

impl Match {
    pub fn new(expr: Node<Expr>, branches: Vec<(Vec<Pattern>, Node<Expr>)>) -> Self {
        Self {
            expr: Box::new(expr),
            branches: branches
                .into_iter()
                .flat_map(|(patterns, expr)| {
                    patterns
                        .into_iter()
                        .map(move |pattern| (pattern, expr.clone()))
                })
                .collect(),
        }
    }
}

impl Typed for Match {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.branches.values().fold_unify(context)
    }
}
