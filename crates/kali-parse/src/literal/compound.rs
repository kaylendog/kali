//! Parsers for compound literals, such as arrays, tuples, and structs.

use chumsky::prelude::*;
use kali_ast::{Expr, Literal};

/// A parser for a tuple literal. Requires a parser for the tuple elements.
pub fn tuple<'a, P: Parser<char, Expr, Error = Simple<char>> + 'a>(
    expr: P,
) -> BoxedParser<'a, char, Literal, Simple<char>> {
    expr.separated_by(just(','))
        .delimited_by(just('('), just(')'))
        .map(|expr| Literal::Tuple(expr))
        .labelled("array")
        .boxed()
}

/// A parser for an array literal. Requires a parser for the array elements.
pub fn array<'a, P: Parser<char, Expr, Error = Simple<char>> + 'a>(
    expr: P,
) -> BoxedParser<'a, char, Literal, Simple<char>> {
    expr.separated_by(just(','))
        .delimited_by(just('['), just(']'))
        .map(|expr| Literal::Array(expr))
        .labelled("tuple")
        .boxed()
}

/// A parser for a compound literal. Requires a parser for the compound elements.
///
/// This parser will attempt to parse a [Literal::Tuple] or [Literal::Array], in that order.
pub fn compound<'a, P: Parser<char, Expr, Error = Simple<char>> + Clone + 'a>(
    expr: P,
) -> BoxedParser<'a, char, Literal, Simple<char>> {
    choice([tuple(expr.clone()), array(expr)]).boxed()
}

#[cfg(test)]
mod tests {
    use chumsky::prelude::*;
    use kali_ast::{Expr, Literal};

    use crate::literal::{
        compound::{array, tuple},
        int,
    };

    #[test]
    fn parse_tuple() {
        assert_eq!(
            tuple(int().map(Expr::Literal)).parse("(1,2,3)").unwrap(),
            Literal::Tuple(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3)),
            ])
        );
    }

    #[test]
    fn parse_array() {
        assert_eq!(
            array(int().map(Expr::Literal)).parse("[1,2,3]").unwrap(),
            Literal::Array(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3)),
            ])
        );
    }

    #[test]
    fn parse_nested_array() {
        assert_eq!(
            recursive(|r| array(r).or(int()).map(Expr::Literal))
                .parse("[[1,2],[3,4]]")
                .unwrap(),
            Expr::Literal(Literal::Array(vec![
                Expr::Literal(Literal::Array(vec![
                    Expr::Literal(Literal::Int(1)),
                    Expr::Literal(Literal::Int(2)),
                ])),
                Expr::Literal(Literal::Array(vec![
                    Expr::Literal(Literal::Int(3)),
                    Expr::Literal(Literal::Int(4)),
                ])),
            ]))
        );
    }
}
