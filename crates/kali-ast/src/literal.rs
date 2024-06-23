use std::collections::BTreeMap;

use kali_type::{Constant, InferenceContext, Type, TypeInferenceError, Typed, Unify};

use crate::expr::Expr;

/// A literal value.
#[derive(Debug, Clone)]
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

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Int(a), Literal::Int(b)) => a == b,
            (Literal::Float(a), Literal::Float(b)) => a == b,
            (Literal::Bool(a), Literal::Bool(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            (Literal::Unit, Literal::Unit) => true,
            (Literal::Array(a), Literal::Array(b)) => a == b,
            (Literal::Tuple(a), Literal::Tuple(b)) => a == b,
            (Literal::Struct(a), Literal::Struct(b)) => a == b,
            _ => false,
        }
    }
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
                // get the types of all elements
                let types = Typed::ty_all(exprs, context)?;
                // then unify
                Unify::unify_all(types.clone(), context)
                    .map_err(|e| TypeInferenceError::UnificationFailed(types, e))?
            }
            Literal::Tuple(exprs) => Type::Tuple(
                exprs
                    .iter()
                    .map(|expr| expr.ty(context))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Literal::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, expr)| Ok((name.clone(), expr.ty(context)?)))
                    .collect::<Result<BTreeMap<_, _>, _>>()?;
                Type::Record(fields)
            }
        })
    }
}
