use std::collections::{BTreeMap, HashMap};

use crate::{Type, TypeUnificationError};

/// The context in which a type is being checked.
#[derive(Default, Debug)]
pub struct InferenceContext {
    /// The types of the variables in scope.
    pub variables: HashMap<String, Type>,
    /// The types of inferred types.
    pub inferred: BTreeMap<String, Type>,
}

/// A type inference error.
pub enum TypeInferenceError {
    /// Unification of types failed.
    UnificationFailed(Vec<Type>, TypeUnificationError),
    /// Multiple errors occurred.
    Multiple(Vec<TypeInferenceError>),
    /// A type mismatch occurred.
    Mismatch {
        /// The expected type.
        expected: Type,
        /// The found type.
        found: Type,
    },
}

impl TypeInferenceError {
    /// Combines two type inference errors.
    pub fn combine(self, other: TypeInferenceError) -> TypeInferenceError {
        match (self, other) {
            (TypeInferenceError::Multiple(mut errors), TypeInferenceError::Multiple(mut other)) => {
                errors.append(&mut other);
                TypeInferenceError::Multiple(errors)
            }
            (TypeInferenceError::Multiple(mut errors), other) => {
                errors.push(other);
                TypeInferenceError::Multiple(errors)
            }
            (self_, TypeInferenceError::Multiple(mut errors)) => {
                errors.push(self_);
                TypeInferenceError::Multiple(errors)
            }
            (self_, other) => TypeInferenceError::Multiple(vec![self_, other]),
        }
    }
}

/// Implemented on types that that have a type within the Kali language.
pub trait Typed {
    /// Returns the type of the value.
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError>;

    /// Infers the types of all items in a collection.
    fn ty_all(
        items: impl IntoIterator<Item = Self>,
        context: &InferenceContext,
    ) -> Result<Vec<Type>, TypeInferenceError>
    where
        Self: Sized,
    {
        let items = items.into_iter();
        let (types, err) = items.map(|item| item.ty(context)).fold(
            (vec![], None::<TypeInferenceError>),
            |(mut types, err), ty| match ty {
                Ok(ty) => {
                    types.push(ty);
                    (types, err)
                }
                Err(e) => (
                    types,
                    Some(match err {
                        Some(err) => err.combine(e),
                        None => e,
                    }),
                ),
            },
        );

        if let Some(err) = err {
            return Err(err);
        }

        Ok(types)
    }
}

impl<T> Typed for &T
where
    T: Typed,
{
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        (*self).ty(context)
    }
}
