use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::Expr;

/// A unary expression.
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
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

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Negate,
    LogicalNot,
    BitwiseNot,
}

impl Typed for UnaryExpr {
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        self.inner.ty(context)
    }
}
