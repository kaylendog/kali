//! Literal values.

use std::collections::BTreeMap;

use kali_type::{Constant, Context, Type, TypeInferenceError, Typed, TypedIterator};

use crate::{expr::Expr, Node};

/// A literal value.
#[derive(Debug, Clone)]
pub enum Literal {
    /// A natural number literal.
    Natural(u64),
    /// An integer literal.
    Integer(i64),
    /// A floating-point literal.
    Float(f64),
    /// A boolean literal.
    Bool(bool),
    /// A string literal.
    String(String),
    /// A unit literal.
    Unit,
    /// An array literal.
    Array(Vec<Node<Expr>>),
    /// A tuple literal.
    Tuple(Vec<Node<Expr>>),
    /// A struct literal.
    Struct(BTreeMap<String, Node<Expr>>),
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Integer(a), Literal::Integer(b)) => a == b,
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
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        Ok(match self {
            Literal::Natural(_) => Type::Constant(Constant::Natural),
            Literal::Integer(_) => Type::Constant(Constant::Integer),
            Literal::Float(_) => Type::Constant(Constant::Float),
            Literal::Bool(_) => Type::Constant(Constant::Bool),
            Literal::String(_) => Type::Constant(Constant::String),
            Literal::Unit => Type::Constant(Constant::Unit),
            Literal::Array(exprs) => exprs.iter().fold_unify(context)?,
            Literal::Tuple(exprs) => exprs
                .iter()
                .map_infer(context)
                .collect::<Result<Vec<_>, _>>()
                .map(Type::Tuple)?,
            Literal::Struct(_fields) => todo!(),
        })
    }
}
