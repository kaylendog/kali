//! Traits for working with iterators of typed values.

use crate::{InferenceContext, Type, TypeInferenceError, Typed, Unify};

/// Utility trait for working with iterators of typed values.
pub trait TypedIterator<T>: Iterator<Item = T>
where
    Self: Sized,
    T: Typed,
{
    /// Folds the iterator into a single type.
    fn fold_unify(mut self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        self.try_fold(Type::Infer("array".to_owned()), |acc, ty| {
            let ty = ty.ty(context)?;
            acc.unify(&ty, context)
                .map_err(|e| TypeInferenceError::UnificationFailed(ty, acc, e))
        })
    }

    /// Maps the iterator into a new iterator of types.
    fn map_infer(
        self,
        context: &InferenceContext,
    ) -> impl Iterator<Item = Result<Type, TypeInferenceError>> {
        self.map(move |ty| ty.ty(context))
    }
}

impl<T, I> TypedIterator<T> for I
where
    T: Typed,
    I: Iterator<Item = T>,
{
}
