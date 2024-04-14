//! Parsers for floating point numbers.

use chumsky::{prelude::*, text::digits};

/// A parser for a floating point number with an optional base prefix, exponent, leading integer part, and the
/// various combinations thereof.
pub fn float() -> impl Parser<char, f64, Error = Simple<char>> {
    digits(10)
        .then(just('.').ignore_then(digits(10)).or_not())
        .then(just('e').ignore_then(digits(10)).or_not())
        .map(|((int, frac), exp)| {
            let mut s = int.to_string();
            if let Some(frac) = frac {
                s.push('.');
                s.push_str(&frac);
            }
            if let Some(exp) = exp {
                s.push('e');
                s.push_str(&exp);
            }
            s.parse().unwrap()
        })
        .labelled("float")
}
