//! Declarations.

use crate::{Expr, Node};

/// A declaration in the AST.
pub struct Decl {
    /// The name of the declaration.
    pub name: String,
    /// The value of the declaration.
    pub value: Node<Expr>,
}

impl Decl {
    /// Creates a new declaration.
    pub fn new(name: String, value: Node<Expr>) -> Self {
        Self { name, value }
    }
}
