//! Expressions.

mod binary;
mod call;
mod conditional;
mod ident;
mod lambda;
mod literal;
mod r#match;
mod unary;

pub use binary::*;
pub use call::*;
pub use conditional::*;
pub use ident::*;
pub use lambda::*;
pub use literal::*;
pub use r#match::*;
pub use unary::*;

/// An expression in the Kali language.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr<Meta> {
    /// A literal value.
    Literal(Literal<Meta>),
    /// An identifier.
    Ident(Identifier<Meta>),
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

impl<Meta> Expr<Meta> {
    /// Get this expressions meta.
    pub fn meta(&self) -> &Meta {
        match self {
            Expr::Literal(literal) => &literal.meta,
            Expr::Ident(identifier) => &identifier.meta,
            Expr::BinaryExpr(binary_expr) => &binary_expr.meta,
            Expr::UnaryExpr(unary_expr) => &unary_expr.meta,
            Expr::Conditional(conditional) => &conditional.meta,
            Expr::Lambda(lambda) => &lambda.meta,
            Expr::Match(_match) => &_match.meta,
            Expr::Call(call) => &call.meta,
        }
    }

    /// Wrap this expression in a box.
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
