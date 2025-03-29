//! Provides a type system for the Kali language loosely based on the Hindley-Milner type system.

use std::{collections::BTreeMap, fmt::Display};

mod engine;
mod infer;
mod iter;
mod unify;

pub use engine::*;
pub use infer::*;
use kali_ast::{ConstantType, TypeExpr, TypeExprKind};
pub use unify::*;

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
    /// A lambda type. Contains the types of the parameters and the return type.
    Lambda(Vec<Type>, Box<Type>),
    /// Represents a type that has not yet been inferred, with a unique ID.
    Infer(usize),
    /// The never type.
    Never,
    /// Represents an error in the type system.
    Error,
}

impl Type {
    /// Returns whether the type is a monotype, i.e. a constant type, or a type constructor with monotype parameters recursively.
    pub fn is_monotype(&self) -> bool {
        match self {
            Type::Constant(_) => true,
            Type::Array(ty) => ty.is_monotype(),
            Type::Tuple(types) => types.iter().all(|ty| ty.is_monotype()),
            Type::Record(fields) => fields.values().all(|ty| ty.is_monotype()),
            Type::Parameterized(_, types) => types.iter().all(|ty| ty.is_monotype()),
            Type::Lambda(params, ret) => {
                params.iter().all(|ty| ty.is_monotype()) && ret.is_monotype()
            }
            _ => false,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Constant(constant) => write!(f, "{}", constant),
            Type::Array(ty) => write!(f, "{}[]", ty),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Record(_) => todo!(),
            Type::Parameterized(_, _) => todo!(),
            Type::Lambda(params, body) => {
                write!(f, "(")?;
                for (i, ty) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ") -> {}", body)
            }
            Type::Infer(id) => write!(f, "'{}", id),
            Type::Error => write!(f, "error"),
            Type::Never => write!(f, "never"),
        }
    }
}

impl<Meta> From<&TypeExpr<Meta>> for Type {
    fn from(value: &TypeExpr<Meta>) -> Self {
        match &value.kind {
            TypeExprKind::Constant(primitive) => match primitive {
                ConstantType::Int => Type::Constant(Constant::Integer),
                ConstantType::Float => Type::Constant(Constant::Float),
                ConstantType::Bool => Type::Constant(Constant::Bool),
                ConstantType::String => Type::Constant(Constant::String),
                ConstantType::Unit => Type::Constant(Constant::Unit),
            },
            TypeExprKind::Variable(name) => todo!("TypeExpr::Variable"),
            TypeExprKind::Function(params, ret) => {
                let params = params.iter().map(|param| param.into()).collect();
                Type::Lambda(params, Box::new(ret.into()))
            }
            TypeExprKind::Tuple(types) => {
                let types = types.iter().map(|ty| ty.into()).collect();
                Type::Tuple(types)
            }
            TypeExprKind::Array(ty) => Type::Array(Box::new(ty.into())),
            TypeExprKind::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, ty)| (name.value.clone(), ty.into()))
                    .collect();
                Type::Record(fields)
            }
        }
    }
}

impl<Meta> From<TypeExpr<Meta>> for Type {
    fn from(value: TypeExpr<Meta>) -> Self {
        Self::from(&value)
    }
}

impl<Meta> From<Box<TypeExpr<Meta>>> for Type {
    fn from(value: Box<TypeExpr<Meta>>) -> Self {
        Self::from(*value)
    }
}

impl<Meta> From<&Box<TypeExpr<Meta>>> for Type {
    fn from(value: &Box<TypeExpr<Meta>>) -> Self {
        Self::from(&**value)
    }
}

/// Constant types in the Kali language.
#[derive(Clone, Debug, PartialEq, strum::Display)]
pub enum Constant {
    /// A signed integer type.
    #[strum(serialize = "integer")]
    Integer,
    /// An unsigned integer type.
    #[strum(serialize = "natural")]
    Natural,
    /// A floating-point type.
    #[strum(serialize = "float")]
    Float,
    /// A boolean type.
    #[strum(serialize = "bool")]
    Bool,
    /// A string type.
    #[strum(serialize = "string")]
    String,
    /// A unit type.
    #[strum(serialize = "()")]
    Unit,
}

#[cfg(test)]
mod tests {
    #[test]
    fn display() {
        assert_eq!(
            format!("{}", crate::Type::Constant(crate::Constant::Integer)),
            "int"
        );
        // array
        assert_eq!(
            format!(
                "{}",
                crate::Type::Array(Box::new(crate::Type::Constant(crate::Constant::Integer)))
            ),
            "int[]"
        );
    }
}
