use std::{
    cell::OnceCell,
    hash::{Hash, Hasher},
    ops::Range,
    path::PathBuf,
    rc::Rc,
};

/// Context associated with a span.
#[derive(Debug, Clone)]
pub struct Context {
    /// The file this span is contained within.
    pub file: Rc<PathBuf>,
}

/// A span with a start and end index.
#[derive(Debug, Clone)]
pub struct Span {
    /// Context associated with this span.
    pub context: OnceCell<Context>,
    /// The start index of this span.
    pub start: usize,
    /// The end index of this span.
    pub end: usize,
}

impl Hash for Span {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.start);
        state.write_usize(self.end);
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Span {}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            context: OnceCell::default(),
        }
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
            context: self.context.clone(),
        }
    }

    /// Create an EOI span.
    pub fn eoi(str: &str) -> Self {
        Self {
            start: str.len(),
            end: str.len(),
            context: OnceCell::new(),
        }
    }
}

impl chumsky::span::Span for Span {
    type Context = OnceCell<Context>;

    type Offset = usize;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Self {
            context,
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {
        self.context.clone()
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}
