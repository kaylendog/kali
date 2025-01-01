//! Metadata for nodes.
//!
//! TODO: It would be nice to make use of the type-state pattern to store metadata alongside [Node]s,
//! but I currently can't work out how to propagate metadata changes higher up the AST.

use kali_type::{Type, Typed};

use crate::Node;

impl<T> Typed for Node<T>
where
    T: Typed,
{
    fn ty(&self, context: &mut kali_type::Context) -> Result<Type, kali_type::TypeInferenceError> {
        self.inner.ty(context)
    }
}
