use kali_type::{InferenceContext, Type, TypeInferenceError, Typed};

use crate::{Expr, TypeExpr};

/// An anonymous function definition.
pub struct Lambda {
    /// The function's parameters.
    pub params: Vec<String>,
    /// The body of the function.
    pub body: Box<Expr>,
}

/// A parameter to a function, with an optional type annotation.
pub struct Parameter {
    /// The parameter's name.
    pub name: String,
    /// An optional type annotation for the parameter.
    pub ty: Option<TypeExpr>,
}

impl Typed for Lambda {
    fn ty(&self, context: &mut InferenceContext) -> Result<Type, TypeInferenceError> {
        // push a new frame for the function's parameters
        context.push_frame();
        for param in &self.params {
            context.insert(param.clone(), Type::Infer(param.clone()));
        }
        // infer the type of the function's body
        let body_ty = self.body.ty(context)?;
        // pop the frame
        context.pop_frame();
        Ok(Type::Lambda(
            self.params.iter().map(|p| Type::Infer(p.clone())).collect(),
            Box::new(body_ty),
        ))
    }
}
