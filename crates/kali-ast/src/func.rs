//! Function-related AST nodes.

use crate::{Expr, TypeExpr};

/// A lambda expression.
pub struct Lambda {
    /// The parameters to the function.
    pub params: Vec<Parameter>,
    pub body: Box<Expr>,
}

/// A parameter to the lambda.
pub struct Parameter {
    /// The parameter name.
    pub name: String,
    /// An optional type annotation.
    pub ty: Option<TypeExpr>,
}
