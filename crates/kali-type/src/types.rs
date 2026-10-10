use std::{fmt, ops::Deref};

/// A type variable is a placeholder for a type that is not yet known. It is
/// used during type inference to represent types that are not yet determined.
/// Each type variable is a thin wrapper around a unique identifier.
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct TypeVar(pub usize);

impl Deref for TypeVar {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An enumeration of all possible types in the type system.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    /// The integer type, representing whole numbers.
    Int,
    /// The floating-point type, representing real numbers.
    Float,
    /// The boolean type, representing true or false values.
    Bool,
    /// The string type, representing sequences of characters.
    String,
    /// The unit type, representing a value that carries no information.
    Unit,
    /// A tuple type, representing a fixed-size collection of values of
    /// potentially different types.
    Tuple(Vec<Type>),
    /// A list type, representing a collection of values of the same type.
    List(Box<Type>),
    /// A record type, representing a collection of named fields, each with its
    /// own type.
    Record(indexmap::IndexMap<lasso::Spur, Type>),
    /// A function type, representing a mapping from one type to another.
    Fn(Box<Type>, Box<Type>),
    /// A type variable, representing a type that is not yet known.
    Var(TypeVar),
    /// An error type, representing a type that could not be inferred or is
    /// invalid.
    Error,
}

impl Type {
    /// Returns a `DisplayType` that can be used to format the type with its
    /// original names.
    pub fn display<'a>(&'a self, rodeo: &'a lasso::Rodeo) -> DisplayType<'a> {
        DisplayType { inner: self, rodeo }
    }

    /// Returns true if the type satisfies the given constraint.
    pub fn satisfies(&self, constraint: &Constraint) -> bool {
        match self {
            // A variable is not decided yet, and an error was already reported.
            Type::Var(_) | Type::Error => true,
            _ => match constraint {
                Constraint::Num => matches!(self, Type::Int | Type::Float),
            },
        }
    }

    /// Returns the type variable if this type is a `Type::Var`, otherwise
    /// returns `None`.
    pub fn as_var(&self) -> Option<TypeVar> {
        if let Type::Var(var) = self {
            Some(var.clone())
        } else {
            None
        }
    }
}

/// A thin wrapper around a reference to a `Type` and a `lasso::Rodeo`, used for
/// displaying types with their original names.
pub struct DisplayType<'a> {
    inner: &'a Type,
    rodeo: &'a lasso::Rodeo,
}

impl<'a> fmt::Display for DisplayType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Unit => write!(f, "()"),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty.display(self.rodeo))?;
                }
                write!(f, ")")
            }
            Type::List(ty) => write!(f, "[{}]", ty.display(self.rodeo)),
            Type::Record(fields) => {
                let fields_str = fields
                    .iter()
                    .map(|(name, ty)| {
                        format!("{}: {}", self.rodeo.resolve(name), ty.display(self.rodeo))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{}}}", fields_str)
            }
            Type::Fn(param_ty, return_ty) => {
                // Parenthesise the parameter type if it's a function type to
                // avoid ambiguity in the printed representation.
                if matches!(**param_ty, Type::Fn(_, _)) {
                    return write!(
                        f,
                        "({}) -> {}",
                        param_ty.display(self.rodeo),
                        return_ty.display(self.rodeo)
                    );
                }
                write!(f, "{} -> {}", param_ty.display(self.rodeo), return_ty.display(self.rodeo))
            }
            Type::Var(var) => write!(f, "'{0}", var.0),
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// An enumeration of synthetic constraints applied to types during type
/// inference. TODO: This is a placeholder for the future trait system.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Constraint {
    /// A numeric constraint, satisfied only by numeric types (Int and Float).
    Num,
}

/// A type scheme represents a type along with its universally quantified type
/// variables.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Scheme {
    /// The type variables that are universally quantified in this scheme.
    pub vars: Vec<TypeVar>,
    /// The type associated with this scheme.
    pub ty: Type,
    /// A list of constraints that must be satisfied by the type variables in
    /// this scheme.
    pub constraints: Vec<(TypeVar, Constraint)>,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::types::{Type, TypeVar};

    #[test]
    fn test_display_fn() {
        //
        let fn_ty = Type::Fn(
            Box::new(Type::Int),
            Box::new(Type::Fn(Box::new(Type::Float), Box::new(Type::Bool))),
        );
        assert_eq!("int -> float -> bool", fn_ty.display(&lasso::Rodeo::new()).to_string());

        let nested_fn_ty = Type::Fn(
            Box::new(Type::Fn(Box::new(Type::Int), Box::new(Type::Float))),
            Box::new(Type::Bool),
        );

        assert_eq!(
            "(int -> float) -> bool",
            nested_fn_ty.display(&lasso::Rodeo::new()).to_string()
        );
    }

    #[test]
    fn test_display_type_var() {
        let type_var = Type::Var(TypeVar(1));
        assert_eq!("'1", type_var.display(&lasso::Rodeo::new()).to_string());
    }
}
