//! # kali-ast
//!
//! This crate provides the abstract syntax tree (AST) for the Kali language.

use std::fmt::{Debug, Display};

mod attr;
mod binary;
mod conditional;
mod expr;
mod func;
mod literal;
mod meta;
mod pattern;
mod ty;
mod unary;

pub use attr::*;
pub use binary::*;
pub use conditional::*;
pub use expr::*;
pub use func::*;
pub use literal::*;
pub use pattern::*;
pub use ty::*;
pub use unary::*;

/// A node in the AST, with an associated span and metadata.
#[derive(Debug, Clone)]
pub struct Node<'src, T, M> {
    /// The inner node.
    pub inner: T,
    /// Metadata associated with the node.
    pub meta: M,
    /// The span of the node in the source code.
    pub span: &'src str,
}

/// A span in the source code.
#[derive(Debug, Clone, Copy)]
pub struct Span<'src> {
    /// The start index of the span.
    pub start: usize,
    /// The end index of the span.
    pub end: usize,
    /// The source code.
    pub src: &'src str,
}

impl<'src> Span<'src> {
    /// Creates a new span from a start and end index.
    pub fn new(start: usize, end: usize, src: &'src str) -> Self {
        Self { start, end, src }
    }

    /// Returns the span as a string slice.
    pub fn as_str(&self) -> &'src str {
        &self.src[self.start..self.end]
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl Display for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
