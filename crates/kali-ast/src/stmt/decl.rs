//! Declarations.

use crate::Expr;

/// A declaration in the AST.
#[derive(Debug, Clone)]
pub struct Decl<Meta> {
    /// The name of the declaration.
    pub name: String,
    /// The value of the declaration.
    pub value: Box<Expr<Meta>>,
}
