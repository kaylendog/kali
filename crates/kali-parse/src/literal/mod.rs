//! Parsers for literals.

use chumsky::{prelude::*, text::keyword};
use kali_ast::{Expr, Literal};

pub mod compound;
mod float;
mod int;

pub use compound::compound;
pub use float::float;
pub use int::int;

/// A parser for a boolean literal.
pub fn bool() -> BoxedParser<'static, char, Literal, Simple<char>> {
    keyword("true")
        .map(|_| Literal::Bool(true))
        .or(keyword("false").map(|_| Literal::Bool(false)))
        .labelled("bool")
        .boxed()
}

/// A parser for a literal. Requires an [Expr] parser for compound literals.
pub fn literal<P: Parser<char, Expr, Error = Simple<char>> + Clone + 'static>(
    expr: P,
) -> impl Parser<char, Literal, Error = Simple<char>> {
    choice([bool(), int(), compound(expr)]).labelled("literal")
}
