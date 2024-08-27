//! Statements in the AST.

use crate::{Literal, TypeExpr};

mod decl;
mod module;

pub use decl::*;
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
