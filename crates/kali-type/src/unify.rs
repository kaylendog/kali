//! Unification of types.

use thiserror::Error;

use crate::{InferenceContext, Type};

/// A trait for types that can be unified.
pub trait Unify {
    /// The context in which the types are being unified.
    type Context;

    /// The error that occurs during unification.
    type Error;

    /// Unifies two instances.
    fn unify(&self, other: &Self, context: &Self::Context) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// An error that occurs during unification of types.
#[derive(Error, Debug)]
pub enum TypeUnificationError {
    /// The types have mismatched lengths, usually when unifying tuples.
    #[error("mismatched lengths: {0} != {1}")]
    MismatchedLength(usize, usize),
    /// A struct type has mismatched fields.
    #[error("mismatched fields: {0}")]
    MismatchedFields(String),
}

impl Unify for Type {
    type Context = InferenceContext;
    type Error = TypeUnificationError;

    fn unify(&self, other: &Self, context: &Self::Context) -> Result<Self, <Type as Unify>::Error>
    where
        Self: Sized,
    {
        match (self, other) {
            // If either type is an inference type, return the other type.
            (Type::Infer(name), x) | (x, Type::Infer(name)) => context
                .variable(name)
                .map(|a| a.unify(&x, &context))
                .unwrap_or(Ok(x.clone())),
            // Arrays unify if their element types unify.
            (Type::Array(a), Type::Array(b)) => a.unify(b, &context).map(|t| Type::Array(t.into())),
            // Tuples unify if their element types unify.
            (Type::Tuple(a), Type::Tuple(b)) => {
                if a.len() != b.len() {
                    return Err(TypeUnificationError::MismatchedLength(a.len(), b.len()));
                }
                let mut types = Vec::new();
                for (a, b) in a.iter().zip(b) {
                    types.push(a.unify(b, &context)?);
                }
                Ok(Type::Tuple(types))
            }
            (Type::Record(a), Type::Record(b)) => {
                if a.len() != b.len() {
                    return Err(TypeUnificationError::MismatchedLength(a.len(), b.len()));
                }
                let mut fields = Vec::new();

                // unify fields - no need to sort, as we're using a BTreeMap
                for ((a_name, a_type), (b_name, b_type)) in a.into_iter().zip(b.into_iter()) {
                    if a_name != b_name {
                        return Err(TypeUnificationError::MismatchedFields(a_name.clone()));
                    }
                    fields.push((a_name.clone(), a_type.unify(&b_type, &context)?));
                }

                Ok(Type::Record(fields.into_iter().collect()))
            }
            // otherwise, the types must be equal
            (x, y) => {
                if x == y {
                    Ok(x.clone())
                } else {
                    Err(TypeUnificationError::MismatchedFields(format!(
                        "{:?} != {:?}",
                        x, y
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{unify::Unify, Constant, InferenceContext, Type};

    #[test]
    fn unify_literals() {
        let int = Type::Constant(Constant::Int);
        let float = Type::Constant(Constant::Float);
        let bool = Type::Constant(Constant::Bool);
        let string = Type::Constant(Constant::String);
        let unit = Type::Constant(Constant::Unit);

        assert_eq!(int.unify(&int, &InferenceContext::default()).unwrap(), int);
        assert_eq!(
            float.unify(&float, &InferenceContext::default()).unwrap(),
            float
        );
        assert_eq!(
            bool.unify(&bool, &InferenceContext::default()).unwrap(),
            bool
        );
        assert_eq!(
            string.unify(&string, &InferenceContext::default()).unwrap(),
            string
        );
        assert_eq!(
            unit.unify(&unit, &InferenceContext::default()).unwrap(),
            unit
        );

        assert!(int.unify(&float, &InferenceContext::default()).is_err());
        assert!(int.unify(&bool, &InferenceContext::default()).is_err());
        assert!(int.unify(&string, &InferenceContext::default()).is_err());
        assert!(int.unify(&unit, &InferenceContext::default()).is_err());
        assert!(float.unify(&bool, &InferenceContext::default()).is_err());
        assert!(float.unify(&string, &InferenceContext::default()).is_err());
        assert!(float.unify(&unit, &InferenceContext::default()).is_err());
        assert!(bool.unify(&string, &InferenceContext::default()).is_err());
        assert!(bool.unify(&unit, &InferenceContext::default()).is_err());
        assert!(string.unify(&unit, &InferenceContext::default()).is_err());
    }

    #[test]
    fn unify_inferred_tuples() {
        let int = Type::Constant(Constant::Int);
        let tuple_a = Type::Tuple(vec![
            Type::Infer("x".to_string()),
            Type::Infer("y".to_string()),
        ]);
        let tuple_b = Type::Tuple(vec![int.clone(), int.clone()]);

        assert_eq!(
            tuple_a
                .unify(&tuple_b, &InferenceContext::default())
                .unwrap(),
            Type::Tuple(vec![int.clone(), int.clone()])
        );

        let tuple_a = Type::Tuple(vec![Type::Infer("x".to_string()), int.clone()]);
        let tuple_b = Type::Tuple(vec![int.clone(), Type::Infer("y".to_string())]);
        assert_eq!(
            tuple_a
                .unify(&tuple_b, &InferenceContext::default())
                .unwrap(),
            Type::Tuple(vec![int.clone(), int.clone()])
        );
    }

    #[test]
    fn unify_inferred_structs() {
        let int = Type::Constant(Constant::Int);
        let struct_a = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), Type::Infer("x".to_string())),
            ("b".to_string(), Type::Infer("y".to_string())),
        ]));
        let struct_b = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), int.clone()),
            ("b".to_string(), int.clone()),
        ]));

        assert_eq!(
            struct_a
                .unify(&struct_b, &InferenceContext::default())
                .unwrap(),
            Type::Record(BTreeMap::from_iter(vec![
                ("a".to_string(), int.clone()),
                ("b".to_string(), int.clone()),
            ]))
        );

        let struct_a = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), Type::Infer("x".to_string())),
            ("b".to_string(), int.clone()),
        ]));
        let struct_b = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), int.clone()),
            ("b".to_string(), Type::Infer("y".to_string())),
        ]));
        assert_eq!(
            struct_a
                .unify(&struct_b, &InferenceContext::default())
                .unwrap(),
            Type::Record(BTreeMap::from_iter(vec![
                ("a".to_string(), int.clone()),
                ("b".to_string(), int.clone()),
            ]))
        );
    }
}
