//! Statements in the AST.

use crate::{Identifier, Literal, TypeExpr};

mod decl;
mod func;
mod module;

pub use decl::*;
pub use func::*;
pub use module::*;

/// A statement in the AST.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stmt<Meta> {
    /// An import statement.
    Import(Import<Meta>),
    /// An export statement.
    Export(Export<Meta>),
    /// A constant declaration.
    Const(Identifier<Meta>, Literal<Meta>),
    /// A type declaration.
    Type(Identifier<Meta>, TypeExpr<Meta>),
    /// A declaration.
    Decl(Decl<Meta>),
    /// A function declaration.
    FuncDecl(FuncDecl<Meta>),
}
