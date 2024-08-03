//! Unary expressions.

use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::Expr;

/// A unary expression.
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    /// The unary operator.
    pub operator: UnaryOp,
    /// The inner expression.
    pub inner: Box<Expr>,
}

impl UnaryExpr {
    pub fn new(operator: UnaryOp, inner: Expr) -> Self {
        Self {
            operator,
            inner: Box::new(inner),
        }
    }
}

/// An enumeration of unary operators.
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    /// The negation operator.
    Negate,
    /// The logical not operator.
    LogicalNot,
    /// The bitwise not operator.
    BitwiseNot,
}

impl Typed for UnaryExpr {
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        self.inner.ty(context)
    }
}
