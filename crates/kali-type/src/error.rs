//! Type inference errors and their origins.

use kali_ast::{Pattern, Span};

use crate::types::{Constraint, Type, TypeVar};

/// A type inference error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// The kind of error that occurred.
    pub kind: ErrorKind,
    /// The origin of the error, providing context about where the error
    /// occurred.
    pub origin: Origin,
}

/// An enumeration of the kinds of type inference errors that can occur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// A mismatch between the expected and actual types occurred during type
    /// inference.
    Mismatch { expected: Type, actual: Type },
    /// Found a type variable that was not declared in the current scope.
    UnknownName(lasso::Spur),
    /// A constraint violation occurred during type inference.
    ConstraintViolation {
        /// The type variable that violated the constraint.
        var: TypeVar,
        /// The constraint that was violated.
        constraint: Constraint,
        /// The type that violated the constraint.
        actual: Type,
    },
    /// An infinite type was encountered during type inference.
    InfiniteType,
    /// Or-pattern bindings were found during type inference.
    OrPatternBindings,
    /// A pattern type mismatch occurred during type inference.
    PatternTypeMismatch {
        /// The pattern that caused the mismatch.
        pattern: Pattern,
        /// The expected type for the pattern.
        expected: Type,
    },
}

/// An enumeration of the origins of type inference errors, providing slightly
/// more context than just a `Span` alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The error relates the two branches of a conditional.
    Conditional {
        condition: Span,
        then: Span,
        otherwise: Span,
    },
    /// The error relates one match arm with another.
    MatchArms {
        first: Span,
        arm: Span,
    },
    /// The error relates a function definition and its argument.
    CallArgument {
        function: Span,
        argument: Span,
    },
    LambdaAnnotation {
        annotation: Span,
        value: Span,
    },
    /// The error relates an operator and its operands.
    BinaryExpr {
        op: Span,
        lhs: Span,
        rhs: Span,
    },
    /// The error relates an operator and its operand.
    UnaryExpr {
        op: Span,
        expr: Span,
    },
    /// The error relates a definition and its value.
    Definition {
        name: Span,
        value: Span,
    },
    /// The error relates a name and its value.
    Name(Span),
    /// An error that occurred in a pattern, providing the span of the pattern.
    Pattern(Span),
    /// Empty origin for debugging purposes.
    Empty,
}
