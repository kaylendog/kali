use kali_type::{InferenceContext, Type, TypeInferenceError, Typed, Unify};

use crate::Expr;

pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub struct BinaryExpr {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
    operator: BinaryOperator,
}

impl Typed for BinaryExpr {
    fn ty(&self, context: &InferenceContext) -> Result<Type, TypeInferenceError> {
        // combine results
        let lhs = self.lhs.ty(context);
        let rhs = self.rhs.ty(context);

        match (lhs, rhs) {
            (Ok(lhs), Ok(rhs)) => lhs
                .unify(&rhs, &context)
                .map_err(|error| TypeInferenceError::UnificationFailed(lhs, rhs, error)),
            (Err(lhs), Ok(_)) => Err(lhs),
            (Ok(_), Err(rhs)) => Err(rhs),
            (Err(lhs), Err(rhs)) => Err(lhs.combine(rhs)),
        }
    }
}
