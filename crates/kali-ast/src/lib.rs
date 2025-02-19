//! # kali-ast
//!
//! This crate provides the abstract syntax tree (AST) for the Kali language.

use std::{cmp::Ordering, fmt::Debug, ops::Range};

mod attr;
mod expr;
mod meta;
mod pattern;
mod stmt;
mod ty;

pub use attr::*;
pub use expr::*;
pub use pattern::*;
pub use stmt::*;
pub use ty::*;

/// A span with a start and end index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (
            self.start.partial_cmp(&other.start),
            self.end.partial_cmp(&other.end),
        ) {
            (Some(Ordering::Less), Some(Ordering::Greater)) => Some(Ordering::Greater),
            (Some(Ordering::Greater), Some(Ordering::Less)) => Some(Ordering::Less),
            _ => None,
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        Self { start, end }
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a new span from a start and end index.
    pub fn into_range(self) -> Range<usize> {
        self.into()
    }

    /// Extend this span to include another span.
    pub fn extend(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Create an EOI span.
    pub fn eoi(str: &str) -> Self {
        Self {
            start: str.len(),
            end: str.len(),
        }
    }
}

impl chumsky::span::Span for Span {
    type Context = ();

    type Offset = usize;

    fn new(_: Self::Context, range: Range<Self::Offset>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {
        ()
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

/// A node in the AST, which wraps an inner value with metadata.
#[derive(Debug, Clone)]
pub struct Node<T, Meta = ()> {
    /// The inner node.
    pub inner: T,
    /// Metadata associated with the node.
    pub meta: Meta,
    /// The span of the node in the source code.
    pub span: Span,
}

impl<T> Node<T> {
    /// Create a new node with no metadata.
    pub fn new<S: Into<Span>>(inner: T, span: S) -> Self {
        Self {
            inner,
            meta: (),
            span: span.into(),
        }
    }

    pub fn with_meta<M>(self, meta: M) -> Node<T, M> {
        Node::<T, M> {
            meta,
            inner: self.inner,
            span: self.span,
        }
    }
}

impl<T, M> Node<T, M> {
    /// Return this node's span as a slice of the given source.
    pub fn as_str<'a>(&'a self, source: &'a str) -> &'a str {
        &source[self.span.into_range()]
    }

    /// Box this node.
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
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

#[derive(Debug, Clone)]
pub struct Module<Meta = ()> {
    pub stmts: Vec<Stmt<Meta>>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
}
