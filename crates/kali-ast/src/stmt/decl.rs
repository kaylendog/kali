//! Declarations.

use kali_type::Typed;

use crate::{Expr, Node};

/// A declaration in the AST.
#[derive(Debug, Clone)]
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

impl Typed for Decl {
    fn ty(
        &self,
        context: &mut kali_type::Context,
    ) -> Result<kali_type::Type, kali_type::TypeInferenceError> {
        self.value.ty(context)?;
        Ok(kali_type::Type::Never)
    }
}
