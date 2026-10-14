//! Types, type variables, schemes and constraints: the data the checker works
//! on, and how it is displayed.

use std::{collections::BTreeMap, fmt};

/// A type variable is a placeholder for a type that is not yet known. It is
/// used during type inference to represent types that are not yet determined.
/// Each type variable is a thin wrapper around a unique identifier.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct TypeVar(pub usize);

/// An enumeration of synthetic constraints applied to types during type
/// inference. TODO: This is a placeholder for the future trait system.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Constraint {
    /// A numeric constraint, satisfied only by numeric types (Int and Float).
    Num,
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Num => write!(f, "num"),
        }
    }
}

/// A type scheme represents a type along with its universally quantified type
/// variables. It is used to express polymorphic types in the type system,
/// allowing for the generalisation and instantiation of types during type
/// inference.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Scheme {
    /// The type variables that are universally quantified in this scheme, i.e.,
    /// unbound variables that can be instantiated with any type, depending on
    /// the context in which the scheme is used.
    pub vars: Vec<TypeVar>,
    /// The type associated with this scheme.
    pub ty: Type,
    /// A list of constraints that must be satisfied by the type variables in
    /// this scheme.
    pub constraints: Vec<(TypeVar, Constraint)>,
}

impl Scheme {
    /// Returns the constraint associated with the given type variable, if any.
    pub fn constraint_of(&self, var: &TypeVar) -> Option<&Constraint> {
        self.constraints.iter().find_map(|(v, c)| (v == var).then_some(c))
    }

    /// Returns a `Display` wrapper for the scheme, which can be used to format
    /// the scheme with its original names.
    pub fn display<'a>(&'a self, rodeo: &'a lasso::Rodeo) -> SchemeDisplay<'a> {
        SchemeDisplay(self, rodeo)
    }
}

/// A wrapper around a scheme for pretty-printing. Formats as `num 'a => 'a ->
/// 'a` with quantified variables implied by their use, as in OCaml.
pub struct SchemeDisplay<'a>(&'a Scheme, &'a lasso::Rodeo);

impl fmt::Display for SchemeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Scheme { constraints, ty, .. } = self.0;
        let names = VarNames::of(ty);

        for (i, (var, constraint)) in constraints.iter().enumerate() {
            write!(f, "{}{constraint} ", if i > 0 { ", " } else { "" })?;
            names.write(f, var)?;
        }
        if !constraints.is_empty() {
            write!(f, " => ")?;
        }
        write_type(f, ty, self.1, &names)
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
    /// Returns a `DisplayType` that formats the type with its original names.
    /// Variables are denoted `'a`, `'b`, ... in order of first appearance.
    pub fn display<'a>(&'a self, rodeo: &'a lasso::Rodeo) -> DisplayType<'a> {
        DisplayType { inner: self, rodeo, names: VarNames::of(self) }
    }

    /// Returns true if the type satisfies the given constraint.
    pub fn satisfies(&self, constraint: &Constraint) -> bool {
        match self {
            // An uninferred variable satisfies any constraint, as do errors in order to prevent
            // cascading.
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
            Some(*var)
        } else {
            None
        }
    }

    /// Returns a tree-walking iterator over the variables in the type, left to
    /// right.
    pub fn walk_vars(&self) -> WalkVars<'_> {
        WalkVars { stack: vec![self] }
    }

    /// Recursively apply a function to all type variables in this type,
    /// returning the result as a freshly instansiated type.
    pub fn map_vars(&self, f: &mut impl FnMut(&TypeVar) -> Type) -> Type {
        match self {
            Type::Var(var) => f(var),
            Type::Tuple(items) => Type::Tuple(items.iter().map(|item| item.map_vars(f)).collect()),
            Type::List(item) => Type::List(item.map_vars(f).into()),
            Type::Record(fields) => {
                Type::Record(fields.iter().map(|(name, ty)| (*name, ty.map_vars(f))).collect())
            }
            Type::Fn(param, ret) => Type::Fn(param.map_vars(f).into(), ret.map_vars(f).into()),
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Error => {
                self.clone()
            }
        }
    }
}

/// A tree-walking iterator over the variables in a type, left to right.
pub struct WalkVars<'a> {
    stack: Vec<&'a Type>,
}

impl<'a> Iterator for WalkVars<'a> {
    type Item = &'a TypeVar;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ty) = self.stack.pop() {
            match ty {
                Type::Var(var) => return Some(var),
                Type::Tuple(items) => self.stack.extend(items.iter().rev()),
                Type::List(item) => self.stack.push(item),
                Type::Record(fields) => self.stack.extend(fields.values().rev()),
                Type::Fn(param, ret) => {
                    self.stack.push(ret);
                    self.stack.push(param);
                }
                Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Error => {}
            }
        }
        None
    }
}

/// The display names of the variables in a type: `'a`, `'b`, ... in order of
/// first appearance, so the same type always prints the same way.
pub(crate) struct VarNames(BTreeMap<TypeVar, usize>);

impl VarNames {
    pub(crate) fn of(ty: &Type) -> Self {
        let mut names = BTreeMap::new();
        for var in ty.walk_vars() {
            let next = names.len();
            names.entry(*var).or_insert(next);
        }
        Self(names)
    }

    /// Write the name of `var`.
    pub(crate) fn write(&self, f: &mut fmt::Formatter<'_>, var: &TypeVar) -> fmt::Result {
        match self.0.get(var) {
            Some(&index) => {
                let letter = (b'a' + (index % 26) as u8) as char;
                match index / 26 {
                    0 => write!(f, "'{letter}"),
                    round => write!(f, "'{letter}{round}"),
                }
            }
            None => write!(f, "'?{}", var.0),
        }
    }
}

/// A thin wrapper around a reference to a `Type` and a `lasso::Rodeo`, used for
/// displaying types with their original names.
pub struct DisplayType<'a> {
    inner: &'a Type,
    rodeo: &'a lasso::Rodeo,
    names: VarNames,
}

impl fmt::Display for DisplayType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_type(f, self.inner, self.rodeo, &self.names)
    }
}

/// Write `ty` using one shared variable numbering for the whole tree.
pub(crate) fn write_type(
    f: &mut fmt::Formatter<'_>,
    ty: &Type,
    rodeo: &lasso::Rodeo,
    names: &VarNames,
) -> fmt::Result {
    match ty {
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
                write_type(f, ty, rodeo, names)?;
            }
            write!(f, ")")
        }
        Type::List(ty) => {
            write!(f, "[")?;
            write_type(f, ty, rodeo, names)?;
            write!(f, "]")
        }
        Type::Record(fields) => {
            write!(f, "{{")?;
            for (i, (name, ty)) in fields.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: ", rodeo.resolve(name))?;
                write_type(f, ty, rodeo, names)?;
            }
            write!(f, "}}")
        }
        Type::Fn(param, ret) => {
            // Parenthesise a function in parameter position, since `->` is
            // right associative.
            if matches!(**param, Type::Fn(_, _)) {
                write!(f, "(")?;
                write_type(f, param, rodeo, names)?;
                write!(f, ")")?;
            } else {
                write_type(f, param, rodeo, names)?;
            }
            write!(f, " -> ")?;
            write_type(f, ret, rodeo, names)
        }
        Type::Var(var) => names.write(f, var),
        Type::Error => write!(f, "<error>"),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::types::{Type, TypeVar};

    fn show(ty: &Type) -> String {
        ty.display(&lasso::Rodeo::new()).to_string()
    }

    #[test]
    fn test_display_fn() {
        let fn_ty = Type::Fn(
            Box::new(Type::Int),
            Box::new(Type::Fn(Box::new(Type::Float), Box::new(Type::Bool))),
        );
        assert_eq!("int -> float -> bool", show(&fn_ty));

        let nested_fn_ty = Type::Fn(
            Box::new(Type::Fn(Box::new(Type::Int), Box::new(Type::Float))),
            Box::new(Type::Bool),
        );
        assert_eq!("(int -> float) -> bool", show(&nested_fn_ty));
    }

    #[test]
    fn test_display_type_var() {
        // Variables are lettered by first appearance, not by index.
        let (a, b) = (Type::Var(TypeVar(7)), Type::Var(TypeVar(2)));
        assert_eq!("'a", show(&a));
        assert_eq!(
            "('a, 'b, 'a) -> ['b]",
            show(&Type::Fn(
                Type::Tuple(vec![a.clone(), b.clone(), a.clone()]).into(),
                Type::List(b.into()).into()
            ))
        );
    }
}
