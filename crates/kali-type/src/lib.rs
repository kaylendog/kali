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
    /// A struct type. Contains the types of the fields.
    Struct(BTreeMap<String, Type>),
    /// A parameterized type.
    Parameterized(String, Vec<Type>),
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
}
