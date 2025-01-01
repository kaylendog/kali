//! Unary expressions.

use bitcode::{Decode, Encode};
use kali_type::{Context, Type, TypeInferenceError, Typed};

use crate::{Expr, Node};

/// A unary expression.
#[derive(Debug, Clone)]
pub struct UnaryExpr<Meta = ()> {
    /// The unary operator.
    pub operator: UnaryOp,
    /// The inner expression.
    pub inner: Box<Node<Expr, Meta>>,
}

/// An enumeration of unary operators.
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum UnaryOp {
    /// The negation operator.
    Negate,
    /// The logical not operator.
    LogicalNot,
    /// The bitwise not operator.
    BitwiseNot,
}

impl Typed for UnaryExpr {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        self.inner.ty(context)
    }
}
