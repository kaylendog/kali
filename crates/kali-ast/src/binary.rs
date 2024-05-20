use kali_type::{InferenceContext, Type, TypeInferenceError, Typed, Unify};

use crate::Expr;

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiate,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub operator: BinaryOperator,
}

impl BinaryExpr {
    pub fn new(lhs: Expr, operator: BinaryOperator, rhs: Expr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            operator,
        }
    }
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
