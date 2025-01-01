//! Conditional expressions.

use kali_type::{Constant, Context, Type, TypeInferenceError, Typed};

use crate::{Expr, Node};

/// A conditional expression.
#[derive(Debug, Clone)]
pub struct Conditional<Meta = ()> {
    /// The condition to check.
    pub condition: Box<Node<Expr, Meta>>,
    /// The body of the conditional.
    pub body: Box<Node<Expr, Meta>>,
    /// The body of the else branch.
    pub otherwise: Box<Node<Expr, Meta>>,
}

impl PartialEq for Conditional {
    fn eq(&self, other: &Self) -> bool {
        self.condition == other.condition
            && self.body == other.body
            && self.otherwise == other.otherwise
    }
}

impl Typed for Conditional {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        // ensure the condition is a boolean
        let condition_ty = self.condition.ty(context)?;
        if condition_ty != Type::Constant(Constant::Bool) {
            return Err(TypeInferenceError::Mismatch {
                expected: Type::Constant(Constant::Bool),
                found: condition_ty,
            });
        }

        // infer the types of the body and otherwise branches
        let body_ty = self.body.ty(context)?;
        let otherwise_ty = self.otherwise.ty(context)?;

        let unified_ty = body_ty
            .unify(&otherwise_ty, context)
            .map_err(|e| TypeInferenceError::UnificationFailed(body_ty, otherwise_ty, e))?;

        Ok(unified_ty)
    }
}
