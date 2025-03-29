use crate::{Context, Type, TypeInferenceError};

pub trait TypeIterator: Iterator<Item = Type>
where
    Self: Sized,
{
    /// Folds the iterator into a single type.
    fn fold_unify(mut self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.try_fold(context.declare_inferred(), |acc, ty| {
            acc.unify(&ty, context)
                .map_err(|e| TypeInferenceError::UnificationFailed(ty, acc, e))
        })
    }
}

impl<I> TypeIterator for I where I: Iterator<Item = Type> {}

pub trait TypeRefIterator<'a>: Iterator<Item = &'a Type>
where
    Self: Sized,
{
    /// Folds the iterator into a single type.
    fn fold_unify(mut self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.try_fold(context.declare_inferred(), |acc, ty| {
            acc.unify(&ty, context)
                .map_err(|e| TypeInferenceError::UnificationFailed(ty.clone(), acc, e))
        })
    }
}

impl<'a, I> TypeRefIterator<'a> for I where I: Iterator<Item = &'a Type> {}
