//! Pattern matching.

use std::hash::Hash;

/// Represents a pattern in the AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// A wildcard pattern.
    Wildcard,
    /// An identifier pattern.
    Ident(String),
    /// A tuple pattern.
    Tuple(Vec<Pattern>),
    /// A cons pattern.
    Cons(Box<Pattern>, Box<Pattern>),
    /// The empty list pattern.
    EmptyList,
    /// A literal pattern.
    Literal(PatternLiteral),
}

/// Represents a literal pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternLiteral {
    /// A literal string pattern.
    String(String),
    /// A literal integer pattern.
    Integer(i64),
    /// A literal natural pattern.
    Natural(u64),
    /// A range pattern.
    Range(i64, i64),
}
