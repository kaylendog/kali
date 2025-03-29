//! Type expressions.

use crate::Identifier;

/// A type expression in the Kali language.
#[derive(Debug, Clone)]
pub struct TypeExpr<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The kind of type expression.
    pub kind: TypeExprKind<Meta>,
}

/// An enumeration of type expression kinds.
#[derive(Debug, Clone)]
pub enum TypeExprKind<Meta> {
    Constant(ConstantType),
    /// A type variable.
    Variable(String),
    /// A function type.
    Function(Vec<TypeExpr<Meta>>, Box<TypeExpr<Meta>>),
    /// A tuple type.
    Tuple(Vec<TypeExpr<Meta>>),
    /// An array type.
    Array(Box<TypeExpr<Meta>>),
    /// A record type.
    Record(Vec<(Identifier<Meta>, TypeExpr<Meta>)>),
}

/// An enumeration of literal constant types.
#[derive(Debug, Clone)]
pub enum ConstantType {
    Int,
    Float,
    Bool,
    String,
    Unit,
}
