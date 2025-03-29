//! Literal values.

use std::collections::BTreeMap;

use super::Expr;

/// A literal value.
#[derive(Debug, Clone)]
pub struct Literal<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The kind of literal.
    pub kind: LiteralKind<Meta>,
}

/// The kind of literal value.
#[derive(Debug, Clone)]
pub enum LiteralKind<Meta> {
    /// A natural number literal.
    Natural(u64),
    /// An integer literal.
    Integer(i64),
    /// A floating-point literal.
    Float(f64),
    /// A boolean literal.
    Bool(bool),
    /// A string literal.
    String(String),
    /// A unit literal.
    Unit,
    /// An array literal.
    Array(Vec<Expr<Meta>>),
    /// A tuple literal.
    Tuple(Vec<Expr<Meta>>),
    /// A struct literal.
    Struct(BTreeMap<String, Expr<Meta>>),
}
