use std::cell::OnceCell;

use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::Node;

/// Empty metadata.
pub struct Empty;

/// Metadata containing a type, used to memoise the result of type inference.
pub struct TypeMetadata {
    /// The type of the node.
    pub ty: OnceCell<Type>,
}

impl<'src, T> Node<'src, T, Empty>
where
    T: Typed,
{
    /// Infers the type of the node, returning the node with type metadata.
    pub fn infer(
        self,
        context: &InferenceContext,
    ) -> Result<Node<'src, T, TypeMetadata>, TypeInferenceError> {
        let ty = self.inner.ty(context)?;
        Ok(Node {
            inner: self.inner,
            meta: TypeMetadata {
                ty: OnceCell::from(ty),
            },
            span: self.span,
        })
    }
}

impl<T> Typed for Node<'_, T, TypeMetadata>
where
    T: Typed,
{
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        match self.meta.ty.get() {
            Some(ty) => Ok(ty.clone()),
            None => {
                let ty = self.inner.ty(context)?;
                self.meta.ty.set(ty.clone()).unwrap();
                Ok(ty)
            }
        }
    }
}
