//! Statements in the AST.

use crate::{Literal, TypeExpr};

mod decl;
mod module;

pub use decl::*;
use kali_type::{Type, Typed};
pub use module::*;

/// A statement in the AST.
pub enum Stmt {
    /// An import statement.
    Import(Import),
    /// An export statement.
    Export(Export),
    /// A constant declaration.
    Const(String, Literal),
    /// A type declaration.
    Type(String, TypeExpr),
    /// A declaration.
    Decl(Decl),
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
        })
    }
}
