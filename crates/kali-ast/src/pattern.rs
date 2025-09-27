//! Pattern matching.

use std::hash::Hash;

use crate::Identifier;

/// A pattern.
#[derive(Debug, Clone)]
pub struct Pattern<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The kind of pattern.
    pub kind: PatternKind<Meta>,
}

impl<Meta> PartialEq for Pattern<Meta> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<Meta> Eq for Pattern<Meta> {}

impl<Meta> Hash for Pattern<Meta> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

/// Represents a pattern in the AST.
#[derive(Debug, Clone)]
pub enum PatternKind<Meta> {
    /// A wildcard pattern.
    Wildcard,
    /// An identifier pattern.
    Ident(Identifier<Meta>),
    /// A tuple pattern.
    Tuple(Vec<PatternKind<Meta>>),
    /// A cons pattern.
    Cons(Box<PatternKind<Meta>>, Box<PatternKind<Meta>>),
    /// The empty list pattern.
    EmptyList,
    /// A literal pattern.
    Literal(PatternLiteralKind),
}

impl<Meta> PartialEq for PatternKind<Meta> {
    fn eq(&self, other: &Self) -> bool {
        use PatternKind as PK;
        match (self, other) {
            (PK::Cons(lhs_head, lhs_tail), PK::Cons(rhs_head, rhs_tail)) => {
                lhs_head == rhs_head && lhs_tail == rhs_tail
            }
            (PK::EmptyList, PK::EmptyList) => true,
            (PK::Ident(lhs), PK::Ident(rhs)) => lhs == rhs,
            (PK::Literal(lhs), PK::Literal(rhs)) => lhs == rhs,
            (PK::Tuple(lhs), PK::Tuple(rhs)) => lhs == rhs,
            (PK::Wildcard, PK::Wildcard) => true,
            (_, _) => false,
        }
    }
}

impl<Meta> Eq for PatternKind<Meta> {}

impl<Meta> Hash for PatternKind<Meta> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use PatternKind as PK;
        match self {
            PK::Cons(head, tail) => {
                head.hash(state);
                tail.hash(state);
            }
            PK::Ident(ident) => ident.hash(state),
            PK::Literal(lit) => lit.hash(state),
            PK::Tuple(tuple) => tuple.hash(state),
            PK::EmptyList => state.write_u8(0),
            PK::Wildcard => state.write_u8(1),
        }
    }
}

/// Represents a literal pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternLiteralKind {
    /// A literal string pattern.
    String(String),
    /// A literal integer pattern.
    Integer(i64),
    /// A literal natural pattern.
    Natural(u64),
    /// A range pattern.
    Range(i64, i64),
}
