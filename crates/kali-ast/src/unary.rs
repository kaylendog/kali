use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::Expr;

/// A unary expression.
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    /// The operator.
    pub operator: UnaryOp,
    /// The operand.
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
    Negate,
    LogicalNot,
    BitwiseNot,
}

impl Typed for UnaryExpr {
    fn ty(&self, context: &mut InferenceContext) -> Result<Type, TypeInferenceError> {
        self.inner.ty(context)
    }
}
