//! Expressions.

use kali_type::{Context, Type, TypeInferenceError, Typed};

use crate::{conditional::Conditional, literal::Literal, unary::UnaryExpr, BinaryExpr, Lambda};

/// An expression in the Kali language.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A literal value.
    Literal(Literal),
    /// An identifier.
    Identifier(String),
    /// A binary expression.
    BinaryExpr(BinaryExpr),
    /// A unary expression.
    UnaryExpr(UnaryExpr),
    /// A conditional expression.
    Conditional(Conditional),
    /// A lambda expression.
    Lambda(Lambda),
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
    fn ty(&self, mut context: &mut Context) -> Result<Type, TypeInferenceError> {
        match self {
            Expr::Literal(literal) => literal.ty(&mut context),
            Expr::Identifier(name) => Ok(context
                .get_known(name)
                .cloned()
                .unwrap_or_else(|| context.declare_inferred())),
            Expr::BinaryExpr(binary_expr) => binary_expr.ty(context),
            Expr::UnaryExpr(unary_expr) => unary_expr.ty(context),
            Expr::Conditional(conditional) => conditional.ty(context),
            Expr::Lambda(lambda) => lambda.ty(context),
        }
    }
}
