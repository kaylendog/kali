//! Lambdas and anonymous functions.

use crate::TypeExpr;

use super::Expr;

/// A lambda expression.
#[derive(Debug, Clone)]
pub struct Lambda<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The parameters to the function.
    pub params: Vec<Parameter<Meta>>,
    /// The body of the function.
    pub body: Box<Expr<Meta>>,
}

/// A parameter to the lambda.
#[derive(Debug, Clone)]
pub struct Parameter<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The parameter name.
    pub name: String,
    /// An optional type annotation.
    pub ty: Option<TypeExpr<Meta>>,
}
