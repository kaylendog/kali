//! Function-related AST nodes.

use kali_type::{Context, Type, TypeInferenceError, Typed};

use crate::{Expr, Node, TypeExpr};

/// A lambda expression.
#[derive(Debug, Clone)]
pub struct Lambda<Meta = ()> {
    /// The parameters to the function.
    pub params: Vec<Node<Parameter, Meta>>,
    /// The body of the function.
    pub body: Box<Node<Expr, Meta>>,
}

/// A parameter to the lambda.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The parameter name.
    pub name: String,
    /// An optional type annotation.
    pub ty: Option<TypeExpr>,
}

impl Typed for Lambda {
    fn ty(&self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        // push the parameters into the context
        let params: Vec<_> = self
            .params
            .iter()
            .map(|param| {
                (
                    param.inner.name.clone(),
                    param
                        .inner
                        .ty
                        .as_ref()
                        .map(|ty| ty.as_ty())
                        .unwrap_or_else(|| context.declare_inferred()),
                )
            })
            .collect();

        // unnecessary cloning?
        context.push().declare_known_iter(params.clone());

        // infer the type of the body
        let body_ty = self.body.ty(context)?;
        context.pop();

        Ok(Type::Lambda(
            params.into_iter().map(|(_, ty)| ty).collect(),
            Box::new(body_ty),
        ))
    }
}
