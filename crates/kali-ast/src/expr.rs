use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::literal::Literal;

/// An expression in the Kali language.
pub enum Expr {
    /// A literal value.
    Literal(Literal),
    /// An identifier.
    Identifier(String),
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
        }
    }
}
