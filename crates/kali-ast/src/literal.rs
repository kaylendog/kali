use std::collections::BTreeMap;

use kali_type::{Constant, InferenceContext, Type, TypeInferenceError, Typed, Unify};

use crate::expr::Expr;

/// A literal value.
pub enum Literal {
    /// An integer literal.
    Int(i64),
    /// A floating-point literal.
    Float(f64),
    /// A boolean literal.
    Bool(bool),
    /// A string literal.
    String(String),
    /// A unit literal.
    Unit,
    /// An array literal.
    Array(Vec<Expr>),
    /// A tuple literal.
    Tuple(Vec<Expr>),
    /// A struct literal.
    Struct(BTreeMap<String, Expr>),
}

impl Typed for Literal {
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        Ok(match self {
            Literal::Int(_) => Type::Constant(Constant::Int),
            Literal::Float(_) => Type::Constant(Constant::Float),
            Literal::Bool(_) => Type::Constant(Constant::Bool),
            Literal::String(_) => Type::Constant(Constant::String),
            Literal::Unit => Type::Constant(Constant::Unit),
            Literal::Array(exprs) => {
                // infer types of all elements, keep track of errors
                let (types, err) = exprs.iter().map(|expr| expr.ty(context)).fold(
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

                // return error if any
                if let Some(err) = err {
                    return Err(err);
                }

                todo!("unify array types");

                // // attempt to unify all types
                // let ty = types
                //     .into_iter()
                //     // TODO: unique identifiers for uninferred types
                //     .fold(Ok(Type::Infer(String::from("empty array"))), |acc, ty| {
                //         acc.and_then(|acc| acc.unify(&ty, context))
                //     })
                //     .map_err(|error| TypeInferenceError::UnificationFailed((), (), ()))?;

                // Type::Array(Box::new(ty))
            }
            Literal::Tuple(exprs) => todo!(),
            Literal::Struct(fields) => todo!(),
        })
    }
}
