//! Expressions.

use kali_type::{Context, Type, TypeInferenceError, Typed};

mod binary;
mod call;
mod conditional;
mod lambda;
mod literal;
mod r#match;
mod unary;

pub use binary::*;
pub use call::*;
pub use conditional::*;
pub use lambda::*;
pub use literal::*;
pub use r#match::*;
pub use unary::*;

/// An expression in the Kali language.
#[derive(Debug, Clone)]
pub enum Expr<Meta = ()> {
    /// A literal value.
    Literal(Literal<Meta>),
    /// An identifier.
    Ident(String),
    /// A binary expression.
    BinaryExpr(BinaryExpr<Meta>),
    /// A unary expression.
    UnaryExpr(UnaryExpr<Meta>),
    /// A conditional expression.
    Conditional(Conditional<Meta>),
    /// A lambda expression.
    Lambda(Lambda<Meta>),
    /// A match expression.
    Match(Match<Meta>),
    /// A function call expression.
    Call(Call<Meta>),
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
            Expr::Ident(name) => Ok(context
                .get_known(name)
                .cloned()
                .unwrap_or_else(|| context.declare_inferred())),
            Expr::BinaryExpr(binary_expr) => binary_expr.ty(context),
            Expr::UnaryExpr(unary_expr) => unary_expr.ty(context),
            Expr::Conditional(conditional) => conditional.ty(context),
            Expr::Lambda(lambda) => lambda.ty(context),
            Expr::Match(m) => m.ty(context),
            Expr::Call(c) => c.ty(context),
        }
    }
}
