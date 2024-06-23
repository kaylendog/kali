//! # kali-type
//!
//! The `kali-type` crate provides the type system for the Kali language.

mod typed;
mod unify;

pub use typed::*;
pub use unify::*;

use std::collections::BTreeMap;

/// A type in the Kali language.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    /// A constant type.
    Constant(Constant),
    /// An array type. Contains the type of the elements.
    Array(Box<Type>),
    /// A tuple type. Contains the types of the elements.
    Tuple(Vec<Type>),
    /// A record type. Contains the types of the fields.
    Record(BTreeMap<String, Type>),
    /// A parameterized type.
    Parameterized(String, Vec<Type>),
    /// A function type. Contains the types of the parameters and the return type.
    Lambda(Vec<Type>, Box<Type>),
    /// Represents a type that has not yet been inferred. Used during type checking.
    Infer(String),
    /// Represents an error in the type system.
    Error,
}

/// Constant types in the Kali language.
#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    /// A signed integer type.
    Int,
    /// An unsigned integer type.
    UnsignedInt,
    /// A floating-point type.
    Float,
    /// A boolean type.
    Bool,
    /// A string type.
    String,
    /// A unit type.
    Unit,
    /// A never type.
    Never,
}

impl Type {
    /// Infers the types of all items in a collection.
    pub fn ty_all<'iter, T>(
        items: impl IntoIterator<Item = &'iter T>,
        context: &mut InferenceContext,
    ) -> Result<Vec<Type>, TypeInferenceError>
    where
        T: Typed + 'iter,
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
