//! Pattern matching.

use std::hash::Hash;

/// Represents a pattern in the AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// A wildcard pattern.
    Wildcard,
    /// An identifier pattern.
    Identifier(String),
    /// A literal pattern.
    Tuple(Vec<Pattern>),
    /// A tuple pattern.
    Cons(Box<Pattern>, Box<Pattern>),
    /// A literal pattern.
    Literal(PatternLiteral),
}

/// Represents a literal pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternLiteral {
    /// A literal string pattern.
    String(String),
    /// A literal integer pattern.
    Int(i64),
    /// A range pattern.
    Range(i64, i64),
}
