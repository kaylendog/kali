use kali_type::{Type, TypeInferenceError, Typed, TypedIterator};

use crate::Node;

use super::Expr;

#[derive(Clone, Debug)]
pub struct Call<Meta = ()> {
    pub fun: Box<Node<Expr, Meta>>,
    pub args: Vec<Node<Expr, Meta>>,
}

impl Typed for Call {
    fn ty(
        &self,
        context: &mut kali_type::Context,
    ) -> Result<kali_type::Type, kali_type::TypeInferenceError> {
        // infer argument types
        let args = self
            .args
            .iter()
            .map_infer(context)
            .collect::<Result<Vec<_>, _>>()?;

        // infer the function type
        let fun_ty = self.fun.ty(context)?;
        let expected_ty = Type::Lambda(
            args.iter().map(|arg| arg.clone()).collect(),
            Box::new(context.declare_inferred()),
        );

        // infer the return type of the function
        let ret = context.declare_inferred();

        // unify with the function type
        fun_ty
            .unify(&Type::Lambda(args, Box::new(ret.clone())), context)
            .map_err(|e| TypeInferenceError::UnificationFailed(fun_ty, expected_ty, e))?;

        // return the return type of the function
        Ok(ret)
    }
}
