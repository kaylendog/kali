use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::{literal::Literal, BinaryExpr};

/// An expression in the Kali language.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A literal value.
    Literal(Literal),
    /// An identifier.
    Identifier(String),
    /// A binary expression.
    BinaryExpr(BinaryExpr),
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Literal(a), Expr::Literal(b)) => a == b,
            // identifiers are excluded - they are not values
            _ => false,
        }
    }
}

impl Typed for Expr {
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        match self {
            Expr::Literal(literal) => literal.ty(&context),
            Expr::Identifier(name) => Ok(context
                .variables
                .get(name)
                .cloned()
                .unwrap_or(Type::Infer(name.clone()))),
            Expr::BinaryExpr(binary_expr) => binary_expr.ty(context),
        }
    }
}
