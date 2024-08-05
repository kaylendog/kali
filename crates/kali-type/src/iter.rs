//! Traits for working with iterators of typed values.

use crate::{Context, Type, TypeInferenceError, Typed};

/// Utility trait for working with iterators of typed values.
pub trait TypedIterator<T>: Iterator<Item = T>
where
    Self: Sized,
    T: Typed,
{
    /// Folds the iterator into a single type.
    fn fold_unify(mut self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.try_fold(context.declare_inferred(), |acc, ty| {
            let ty = ty.ty(context)?;
            acc.unify(&ty, context)
                .map_err(|e| TypeInferenceError::UnificationFailed(ty, acc, e))
        })
    }

    /// Maps the iterator into a new iterator of types.
    fn map_infer(
        self,
        context: &mut Context,
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

pub trait TypeIterator: Iterator<Item = Type>
where
    Self: Sized,
{
    fn fold_unify(mut self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.try_fold(context.declare_inferred(), |acc, ty| {
            acc.unify(&ty, context)
                .map_err(|e| TypeInferenceError::UnificationFailed(ty, acc, e))
        })
    }

    fn map_resolve(
        self,
        context: &mut Context,
    ) -> impl Iterator<Item = Result<Type, TypeInferenceError>> {
        self.map(move |ty| ty.resolve(context))
    }
}

impl<I> TypeIterator for I where I: Iterator<Item = Type> {}
