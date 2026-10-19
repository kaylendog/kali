//! A simple span type for representing ranges in source code, with a start and
//! end offset and context information.

use std::{
    fmt,
    ops::{self, Deref, Range},
};

/// A simple span representing a range in the source code, with a start and end
/// offset and an optional context.
///
/// Currently a line-for-line copy of `chumsky::span::SimpleSpan`, but pulled
/// out in case I want to add more functionality to it later without needing to
/// rewrite the parser.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// The start offset of the span.
    pub start: usize,
    /// The end (exclusive) offset of the span.
    pub end: usize,
    /// The context of the span.
    pub context: Context,
}

/// A context for a span, which can be used to store additional information
/// about the span, such as the source file or line number.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context {
    /// The source file of the span.
    pub src: SourceId,
}

impl Default for Context {
    fn default() -> Self {
        Self { src: SourceId(0) }
    }
}

/// A unique identifier for a source file, used to identify the source of a
/// span.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(pub usize);

impl Span {
    /// Convert this span into a [`std::ops::Range`].
    pub fn into_range(self) -> Range<usize> {
        self.start..self.end
    }

    /// Set the context of this span.
    pub fn with_context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span { start: range.start, end: range.end, context: Context { src: SourceId(0) } }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        Range { start: span.start, end: span.end }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl IntoIterator for Span {
    type IntoIter = Range<usize>;
    type Item = usize;

    fn into_iter(self) -> Self::IntoIter {
        self.start..self.end
    }
}

impl chumsky::span::Span for Span {
    type Context = Context;
    type Offset = usize;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Self { start: range.start, end: range.end, context }
    }
    fn context(&self) -> Self::Context {
        self.context.clone()
    }
    fn start(&self) -> Self::Offset {
        self.start.clone()
    }
    fn end(&self) -> Self::Offset {
        self.end.clone()
    }
}

impl ariadne::Span for Span {
    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        &self.context.src
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl ariadne::Span for &Span {
    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        &self.context.src
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

/// A spanned value, which contains a value and a span.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spanned<T> {
    /// The value of the spanned value.
    pub value: T,
    /// The span of the spanned value.
    pub span: Span,
}

impl<T> ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// A trait for types that can be spanned, i.e. literally anything.
pub trait WithSpan {
    /// Wrap this value in a [`Spanned`] with the given span.
    fn with_span(self, span: Span) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned { value: self, span }
    }
}

impl<T> WithSpan for T {}

impl<T> fmt::Display for Spanned<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
