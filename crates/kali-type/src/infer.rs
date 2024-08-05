use std::{cell::RefCell, collections::HashMap, rc::Rc};

use thiserror::Error;

use crate::{Type, TypeUnificationError};

/// The type inference context.
///
/// This struct is used to store the current state of the type inference algorithm, including
/// a map of known polymorphic types, a stack of scopes, and the state of the inference counter.
#[derive(Debug)]

pub struct Context {
    /// A stack of scopes.
    pub scope: Vec<Scope>,
    /// The next inference variable counter.
    pub counter: Rc<RefCell<usize>>,
    /// A map of inference types with known types.
    pub inferred: HashMap<usize, Type>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Creates a new inference context, with a single top-level frame.
    pub fn new() -> Self {
        let counter = Rc::new(RefCell::new(0));
        Self {
            scope: vec![Scope::new(counter.clone())],
            counter: counter.clone(),
            inferred: HashMap::new(),
        }
    }

    /// Pushes a new scope onto the stack.
    pub fn push(&mut self) -> &mut Self {
        self.scope.push(Scope::new(self.counter.clone()));
        self
    }

    /// Pops the current scope from the stack. Panics if there is only one scope.
    pub fn pop(&mut self) {
        if self.scope.len() == 1 {
            panic!("cannot pop the top-level scope");
        }
        self.scope.pop();
    }

    /// Returns the current inference frame.
    pub fn scope(&self) -> &Scope {
        self.scope.last().unwrap()
    }

    /// Returns a mutable reference to the current inference frame.
    pub fn scope_mut(&mut self) -> &mut Scope {
        self.scope.last_mut().unwrap()
    }

    /// Returns the type of a known type in the context.
    pub fn get_known(&self, name: &str) -> Option<&Type> {
        self.scope
            .iter()
            .rev()
            .find_map(|scope| scope.known.get(name))
    }

    /// Declares a known type in the context.
    pub fn declare_known(&mut self, name: String, ty: Type) {
        self.scope_mut().known.insert(name, ty);
    }

    /// Declares an iterator of known types in the context.
    pub fn declare_known_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (String, Type)>,
    {
        self.scope_mut().known.extend(iter.into_iter());
    }

    /// Returns the type of an inferred type in the context.
    pub fn get_inferred(&self, idx: usize) -> Option<&Type> {
        self.inferred.get(&idx)
    }

    /// Declares a variable in the current scope.
    pub fn declare_inferred(&mut self) -> Type {
        let counter = *self.counter.borrow();
        *self.counter.borrow_mut() += 1;
        Type::Infer(counter)
    }

    /// Infers a new type in the current scope.
    pub fn infer(&mut self, idx: usize, real: Type) {
        self.inferred.insert(idx, real);
    }
}

#[derive(Debug)]
pub struct Scope {
    /// A map of named types in the context.
    pub known: HashMap<String, Type>,

    /// A reference to the global inference counter.
    pub counter: Rc<RefCell<usize>>,
}

impl Scope {
    pub fn new(counter: Rc<RefCell<usize>>) -> Self {
        Self {
            known: HashMap::new(),
            counter,
        }
    }
}

/// A type inference error.
#[derive(Debug, Error)]
pub enum TypeInferenceError {
    /// Unification of types failed.
    #[error("unification failed: {0} and {1}: {2}")]
    UnificationFailed(Type, Type, TypeUnificationError),
    /// Multiple errors occurred.
    #[error("multiple errors occurred")]
    Multiple(Vec<TypeInferenceError>),
    /// A type mismatch occurred.
    #[error("expected type {expected}, found {found}")]
    Mismatch {
        /// The expected type.
        expected: Type,
        /// The found type.
        found: Type,
    },
    /// Resolution of a type failed.
    #[error("resolution failed: {0}")]
    ResolutionFailed(Type),
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
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError>;
}

impl<T> Typed for &T
where
    T: Typed,
{
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        (*self).ty(context)
    }
}

impl<T> Typed for Box<T>
where
    T: Typed,
{
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.as_ref().ty(context)
    }
}
