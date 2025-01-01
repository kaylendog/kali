//! Binary expressions and associated types.

use bitcode::{Decode, Encode};
use kali_type::{Constant, Context, Type, TypeInferenceError, Typed};

use crate::{Expr, Node};

/// An enumeration of binary operators.
#[derive(Debug, Clone, Copy, Encode, Decode)]
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
    Cons,
}

/// A binary expression.
#[derive(Debug, Clone)]
pub struct BinaryExpr<Meta = ()> {
    /// The left-hand side of the expression.
    pub lhs: Box<Node<Expr, Meta>>,
    /// The right-hand side of the expression.
    pub rhs: Box<Node<Expr, Meta>>,
    /// The binary operator.
    pub operator: BinaryOp,
}

impl Typed for BinaryExpr {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        match (self.lhs.ty(context), self.rhs.ty(context)) {
            // both ok
            (Ok(lhs), Ok(rhs)) => lhs
                .unify(&rhs, context)
                .and_then(|inner| match self.operator {
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::LessThan
                    | BinaryOp::LessThanOrEqual
                    | BinaryOp::GreaterThan
                    | BinaryOp::GreaterThanOrEqual => {
                        // require the inner type to be a constant
                        inner
                            .unify(&Type::Constant(Constant::Integer), context)
                            .map(|_| Type::Constant(Constant::Bool))
                    }
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                        // require the inner type to be a boolean
                        inner.unify(&Type::Constant(Constant::Bool), context)
                    }
                    _ => Ok(inner),
                })
                .map_err(|error| TypeInferenceError::UnificationFailed(lhs, rhs, error)),
            (Err(err), Ok(_)) | (Ok(_), Err(err)) => Err(err),
            (Err(lhs), Err(rhs)) => Err(lhs.combine(rhs)),
        }
    }
}
