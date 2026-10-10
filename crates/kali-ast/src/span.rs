use std::{fmt, ops::Range};

/// A simple span representing a range in the source code, with a start and end
/// offset and an optional context.
///
/// Currently a line-for-line copy of `chumsky::span::SimpleSpan`, but pulled
/// out in case I want to add more functionality to it later without needing to
/// rewrite the parser.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span<T = usize, C = ()> {
    /// The start offset of the span.
    pub start: T,
    /// The end (exclusive) offset of the span.
    pub end: T,
    /// The context of the span.
    pub context: C,
}

impl<T, C> Span<T, C> {
    /// Convert this span into a [`std::ops::Range`].
    pub fn into_range(self) -> Range<T> {
        self.start..self.end
    }
}

impl<T> From<Range<T>> for Span<T> {
    fn from(range: Range<T>) -> Self {
        Span { start: range.start, end: range.end, context: () }
    }
}

impl<T> From<Span<T, ()>> for Range<T> {
    fn from(span: Span<T>) -> Self {
        Range { start: span.start, end: span.end }
    }
}

impl<T, C> fmt::Debug for Span<T, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl<T, C> fmt::Display for Span<T, C>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl<T, C> IntoIterator for Span<T, C>
where
    Range<T>: Iterator<Item = T>,
{
    type IntoIter = Range<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.start..self.end
    }
}

impl<T: Clone, C: Clone> chumsky::span::Span for Span<T, C> {
    type Context = C;
    type Offset = T;

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
