//! # kali-ast
//!
//! This crate provides the abstract syntax tree (AST) for the Kali language.

mod attr;
mod expr;
mod pattern;
mod rewriter;
mod stmt;
mod ty;

pub use attr::*;
pub use expr::*;
pub use pattern::*;
pub use rewriter::*;
pub use stmt::*;
pub use ty::*;

/// A module, representing a single file translation unit.
#[derive(Debug, Clone)]
pub struct Module<Meta> {
    /// Top-level statements in this module.
    pub stmts: Vec<Stmt<Meta>>,
    /// Imports in this module.
    pub imports: Vec<Import<Meta>>,
    /// Exports in this module.
    pub exports: Vec<Export<Meta>>,
}
