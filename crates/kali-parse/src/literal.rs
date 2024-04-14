use kali_ast::Literal;

use chumsky::{
    prelude::*,
    text::{digits, keyword},
};

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
pub fn int() -> impl Parser<char, Literal, Error = Simple<char>> {
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
}

/// A parser for a boolean literal.
pub fn bool() -> impl Parser<char, Literal, Error = Simple<char>> {
    keyword("true")
        .map(|_| Literal::Bool(true))
        .or(keyword("false").map(|_| Literal::Bool(false)))
}

pub fn prefixed_float_with_base(
    chars: &'static str,
    base: u32,
) -> impl Parser<char, String, Error = Simple<char>> {
    let exponent = one_of("eE")
        .ignore_then(just('-').or_not())
        .ignore_then(digits(10));

    just('0')
        .then(one_of(chars))
        .labelled("prefix")
        .ignore_then(digits(base))
        .then(just('.').ignore_then(digits(base)))
        .then(exponent.or_not())
}
