//! Parsers for integer literals.

use chumsky::{prelude::*, text::digits};
use kali_ast::Literal;

/// A parser for an integer with an optional base prefix.
pub fn prefixed_uint_with_base(
    chars: &'static str,
    base: u32,
) -> impl Parser<char, String, Error = Simple<char>> {
    just('0')
        .then(one_of(chars))
        .labelled("prefix")
        .ignore_then(digits(base))
}

/// A parser for a signed integer, with support for signed binary, octal, decimal, and hexadecimal integers.
pub fn int() -> BoxedParser<'static, char, Literal, Simple<char>> {
    just("-")
        .or_not()
        .then(
            choice([
                prefixed_uint_with_base("bB", 2).labelled("binary int"),
                prefixed_uint_with_base("oO", 8).labelled("octal int"),
                prefixed_uint_with_base("dD", 10).labelled("decimal int"),
                prefixed_uint_with_base("xX", 16).labelled("hexadecimal int"),
            ])
            .or(digits(10)),
        )
        .map(|(sign, s)| {
            let i = i64::from_str_radix(&s, 10).unwrap();
            Literal::Int(if sign.is_some() { -i } else { i })
        })
        .labelled("int")
        .boxed()
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;
    use kali_ast::Literal;

    use crate::literal::int;

    #[test]
    fn parse_int() {
        assert_eq!(int().parse("0").unwrap(), Literal::Int(0));
        assert_eq!(int().parse("1").unwrap(), Literal::Int(1));
        assert_eq!(int().parse("10").unwrap(), Literal::Int(10));
    }

    #[test]
    fn parse_negative_int() {
        assert_eq!(int().parse("-0").unwrap(), Literal::Int(0));
        assert_eq!(int().parse("-1").unwrap(), Literal::Int(-1));
        assert_eq!(int().parse("-10").unwrap(), Literal::Int(-10));
    }

    #[test]
    fn parse_binary_int() {
        assert_eq!(int().parse("0b0").unwrap(), Literal::Int(0));
        assert_eq!(int().parse("0b1").unwrap(), Literal::Int(1));
        assert_eq!(int().parse("0b10").unwrap(), Literal::Int(2));
    }

    #[test]
    fn parse_octal_int() {
        assert_eq!(int().parse("0o0").unwrap(), Literal::Int(0));
        assert_eq!(int().parse("0o1").unwrap(), Literal::Int(1));
        assert_eq!(int().parse("0o10").unwrap(), Literal::Int(8));
    }

    #[test]
    fn parse_hexadecimal_int() {
        assert_eq!(int().parse("0x0").unwrap(), Literal::Int(0));
        assert_eq!(int().parse("0x1").unwrap(), Literal::Int(1));
        assert_eq!(int().parse("0x10").unwrap(), Literal::Int(16));
    }
}
