//! Unification of types.

use std::cmp::Ordering;

use thiserror::Error;
use tracing::trace;

use crate::{Context, Type};

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

impl Type {
    /// Unified two types.
    pub fn unify(&self, other: &Self, context: &mut Context) -> Result<Self, TypeUnificationError> {
        trace!("unify");
        match (self, other) {
            // enforce ordering of inference types
            (Type::Infer(idx), Type::Infer(idy)) => match idx.cmp(idy) {
                Ordering::Less => {
                    context.infer(*idx, other.clone());
                    Ok(other.clone())
                }
                Ordering::Greater => {
                    context.infer(*idy, self.clone());
                    Ok(self.clone())
                }
                Ordering::Equal => Ok(self.clone()),
            },
            // if either type is an inference type, return the other type.
            (Type::Infer(idx), x) | (x, Type::Infer(idx)) => {
                context.infer(*idx, x.clone());
                Ok(x.clone())
            }
            // arrays unify if their element types unify.
            (Type::Array(a), Type::Array(b)) => a.unify(b, context).map(|t| Type::Array(t.into())),
            // tuples unify if their element types unify.
            (Type::Tuple(a), Type::Tuple(b)) => {
                if a.len() != b.len() {
                    return Err(TypeUnificationError::MismatchedLength(a.len(), b.len()));
                }
                let mut types = Vec::new();
                for (a, b) in a.iter().zip(b) {
                    types.push(a.unify(b, context)?);
                }
                Ok(Type::Tuple(types))
            }
            // records unify if their fields unify
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
                    fields.push((a_name.clone(), a_type.unify(&b_type, context)?));
                }

                Ok(Type::Record(fields.into_iter().collect()))
            }
            // otherwise, the types must be identical
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

    use crate::{Constant, Context, Type};

    #[test]
    fn unify_literals() {
        let int = Type::Constant(Constant::Int);
        let float = Type::Constant(Constant::Float);
        let bool = Type::Constant(Constant::Bool);
        let string = Type::Constant(Constant::String);
        let unit = Type::Constant(Constant::Unit);

        assert_eq!(int.unify(&int, &mut Context::default()).unwrap(), int);
        assert_eq!(float.unify(&float, &mut Context::default()).unwrap(), float);
        assert_eq!(bool.unify(&bool, &mut Context::default()).unwrap(), bool);
        assert_eq!(
            string.unify(&string, &mut Context::default()).unwrap(),
            string
        );
        assert_eq!(unit.unify(&unit, &mut Context::default()).unwrap(), unit);

        assert!(int.unify(&float, &mut Context::default()).is_err());
        assert!(int.unify(&bool, &mut Context::default()).is_err());
        assert!(int.unify(&string, &mut Context::default()).is_err());
        assert!(int.unify(&unit, &mut Context::default()).is_err());
        assert!(float.unify(&bool, &mut Context::default()).is_err());
        assert!(float.unify(&string, &mut Context::default()).is_err());
        assert!(float.unify(&unit, &mut Context::default()).is_err());
        assert!(bool.unify(&string, &mut Context::default()).is_err());
        assert!(bool.unify(&unit, &mut Context::default()).is_err());
        assert!(string.unify(&unit, &mut Context::default()).is_err());
    }

    #[test]
    fn unify_inferred_tuples() {
        let int = Type::Constant(Constant::Int);
        let tuple_a = Type::Tuple(vec![Type::Infer(0), Type::Infer(1)]);
        let tuple_b = Type::Tuple(vec![int.clone(), int.clone()]);

        assert_eq!(
            tuple_a.unify(&tuple_b, &mut Context::default()).unwrap(),
            Type::Tuple(vec![int.clone(), int.clone()])
        );

        let tuple_a = Type::Tuple(vec![Type::Infer(0), int.clone()]);
        let tuple_b = Type::Tuple(vec![int.clone(), Type::Infer(1)]);
        assert_eq!(
            tuple_a.unify(&tuple_b, &mut Context::default()).unwrap(),
            Type::Tuple(vec![int.clone(), int.clone()])
        );
    }

    #[test]
    fn unify_inferred_structs() {
        let int = Type::Constant(Constant::Int);
        let struct_a = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), Type::Infer(0)),
            ("b".to_string(), Type::Infer(1)),
        ]));
        let struct_b = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), int.clone()),
            ("b".to_string(), int.clone()),
        ]));

        assert_eq!(
            struct_a.unify(&struct_b, &mut Context::default()).unwrap(),
            Type::Record(BTreeMap::from_iter(vec![
                ("a".to_string(), int.clone()),
                ("b".to_string(), int.clone()),
            ]))
        );

        let struct_a = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), Type::Infer(0)),
            ("b".to_string(), int.clone()),
        ]));
        let struct_b = Type::Record(BTreeMap::from_iter(vec![
            ("a".to_string(), int.clone()),
            ("b".to_string(), Type::Infer(1)),
        ]));
        assert_eq!(
            struct_a.unify(&struct_b, &mut Context::default()).unwrap(),
            Type::Record(BTreeMap::from_iter(vec![
                ("a".to_string(), int.clone()),
                ("b".to_string(), int.clone()),
            ]))
        );
    }
}
