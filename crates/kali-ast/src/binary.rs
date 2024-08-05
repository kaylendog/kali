//! Binary expressions and associated types.

use kali_type::{Context, Type, TypeInferenceError, Typed};

use crate::{Expr, Node};

/// An enumeration of binary operators.
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
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
    Concatenate,
}

/// A binary expression.
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    /// The left-hand side of the expression.
    pub lhs: Box<Node<Expr>>,
    /// The right-hand side of the expression.
    pub rhs: Box<Node<Expr>>,
    /// The binary operator.
    pub operator: BinaryOp,
}

impl BinaryExpr {
    pub fn new(lhs: Node<Expr>, operator: BinaryOp, rhs: Node<Expr>) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            operator,
        }
    }
}

impl Typed for BinaryExpr {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        match (self.lhs.ty(context), self.rhs.ty(context)) {
            // both ok
            (Ok(lhs), Ok(rhs)) => lhs
                .unify(&rhs, context)
                .map_err(|error| TypeInferenceError::UnificationFailed(lhs, rhs, error)),
            (Err(lhs), Ok(_)) => Err(lhs),
            (Ok(_), Err(rhs)) => Err(rhs),
            (Err(lhs), Err(rhs)) => Err(lhs.combine(rhs)),
        }
    }
}
