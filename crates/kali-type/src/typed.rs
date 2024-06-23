use std::collections::HashMap;

use crate::{Type, TypeUnificationError};

/// The context in which a type is being checked.
#[derive(Debug)]
pub struct InferenceContext {
    /// A list of frames in the context.
    pub frames: Vec<Frame>,
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self {
            frames: vec![Frame::default()],
        }
    }
}

impl InferenceContext {
    /// Push an empty frame onto the context.
    pub fn push_frame(&mut self) {
        self.frames.push(Frame::default());
    }

    /// Pop a frame from the context.
    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    /// Looks up a variable in the context.
    pub fn get(&self, name: &str) -> Option<&Type> {
        for frame in self.frames.iter().rev() {
            if let Some(ty) = frame.variables.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Declares a variable in the context.
    pub fn insert(&mut self, name: String, ty: Type) {
        self.frames.last_mut().unwrap().variables.insert(name, ty);
    }
}

/// A frame in the context.
#[derive(Default, Debug)]
pub struct Frame {
    pub variables: HashMap<String, Type>,
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
    fn ty(&self, context: &mut InferenceContext) -> Result<Type, TypeInferenceError>;
}

impl<T> Typed for &T
where
    T: Typed,
{
    fn ty(&self, context: &mut InferenceContext) -> Result<Type, TypeInferenceError> {
        (*self).ty(context)
    }
}
