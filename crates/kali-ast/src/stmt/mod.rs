//! Statements in the AST.

use kali_type::{Type, Typed};

use crate::{Literal, TypeExpr};

mod decl;
mod func;
mod module;

pub use decl::*;
pub use func::*;
pub use module::*;

/// A statement in the AST.
#[derive(Debug, Clone)]
pub enum Stmt<Meta = ()> {
    /// An import statement.
    Import(Import),
    /// An export statement.
    Export(Export),
    /// A constant declaration.
    Const(String, Literal<Meta>),
    /// A type declaration.
    Type(String, TypeExpr),
    /// A declaration.
    Decl(Decl<Meta>),
    /// A function declaration.
    FuncDecl(FuncDecl<Meta>),
}

impl Typed for Stmt {
    fn ty(
        &self,
        context: &mut kali_type::Context,
    ) -> Result<kali_type::Type, kali_type::TypeInferenceError> {
        Ok(match self {
            Stmt::Import(_) => Type::Never,
            Stmt::Export(_) => Type::Never,
            Stmt::Const(_, _) => Type::Never,
            Stmt::Type(_, _) => {
                todo!()
            }
            Stmt::Decl(decl) => {
                // TODO: this seems hacky - we need to run the type checker on the declaration,
                // but since statements don't have types, we must return never, rather than any
                // type from further down the AST.
                decl.ty(context)?;
                Type::Never
            }
            Stmt::FuncDecl(FuncDecl {
                name,
                params,
                ret_ty,
                body,
            }) => {
                // declare return type
                let ret_ty = ret_ty
                    .as_ref()
                    .map(|ty| ty.inner.as_ty())
                    .unwrap_or_else(|| context.declare_inferred());

                // declare parameters
                let params: Vec<_> = params
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

                context.push().declare_known_iter(params.iter().cloned());

                // declare function itself - done before body to allow for recursion
                context.declare_known(
                    name.clone(),
                    Type::Lambda(
                        params.iter().map(|(_, ty)| ty).cloned().collect(),
                        Box::new(ret_ty.clone()),
                    ),
                );

                // infer the type of the body
                let body_ty = body.ty(context)?;
                ret_ty.unify(&body_ty, context).map_err(|e| {
                    kali_type::TypeInferenceError::UnificationFailed(body_ty, ret_ty, e)
                })?;

                context.pop();

                Type::Never
            }
        })
    }
}
