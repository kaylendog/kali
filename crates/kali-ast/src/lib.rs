//! # kali-ast
//!
//! This crate provides the abstract syntax tree (AST) for the Kali language.

use std::{fmt::Debug, ops::Range};

mod attr;
mod expr;
mod meta;
mod pattern;
mod stmt;
mod ty;

pub use attr::*;
pub use expr::*;
pub use meta::*;
pub use pattern::*;
pub use stmt::*;
pub use ty::*;

/// A node in the AST, with an associated span and metadata.
#[derive(Debug, Clone)]
pub struct Node<T> {
    /// The inner node.
    pub inner: T,
    /// Metadata associated with the node.
    pub meta: Meta,
    /// The span of the node in the source code.
    pub span: Range<usize>,
}

impl<T> Node<T> {
    /// Create a new node with no metadata.
    pub fn new(inner: T, span: Range<usize>) -> Self {
        Self {
            inner,
            meta: Meta::default(),
            span,
        }
    }

    /// Return this node's span as a slice of the given source.
    pub fn as_str<'a>(&'a self, source: &'a str) -> &'a str {
        &source[self.span.clone()]
    }
}

impl<T> PartialEq for Node<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
