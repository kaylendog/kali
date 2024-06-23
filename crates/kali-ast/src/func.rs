use crate::{Expr, TypeExpr};

/// An anonymous function definition.
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Box<Expr>,
}

/// A parameter to a function, with an optional type annotation.
pub struct Parameter {
    pub name: String,
    pub ty: Option<TypeExpr>,
}
