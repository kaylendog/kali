use std::collections::{BTreeMap, HashMap};

use crate::{Type, TypeUnificationError};

/// The context in which a type is being checked.
#[derive(Default, Debug)]

pub struct InferenceContext {
    /// The frames in the context.
    pub frames: Vec<ContextFrame>,
}

impl InferenceContext {
    /// Pushes a new frame onto the context.
    pub fn push(&mut self) -> &mut ContextFrame {
        self.frames.push(ContextFrame::default());
        self.frames.last_mut().unwrap()
    }

    /// Pops the last frame from the context.
    pub fn pop(&mut self) {
        self.frames.pop();
    }

    /// Gets the type of a variable in the context.
    pub fn variable(&self, name: &str) -> Option<&Type> {
        for frame in self.frames.iter().rev() {
            if let Some(ty) = frame.variables.get(name) {
                return Some(&ty);
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct ContextFrame {
    /// The types of the variables in scope.
    pub variables: HashMap<String, Type>,
    /// The types of inferred types.
    pub inferred: BTreeMap<String, Type>,
}

/// A type inference error.
pub enum TypeInferenceError {
    /// Unification of types failed.
    UnificationFailed(Type, Type, TypeUnificationError),
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
}

impl<T> Typed for &T
where
    T: Typed,
{
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        (*self).ty(context)
    }
}
