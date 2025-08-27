//! Implements a simple LL parser for the Kali language.

/// Re-export of [`chumsky`], the parsing library used internally.
pub use chumsky;
use chumsky::{
    error::Rich,
    input::{Input, Stream},
    Parser,
};

use kali_ast::Module;

pub mod common;
pub mod expr;
pub mod lexer;
pub mod pattern;
pub mod span;
pub mod stmt;
pub mod ty_expr;

pub use lexer::{IndentLexer, Token};
pub use span::Span;

/// Parse a string into a Kali module.
pub fn parse_str<'src>(
    input: &'src str,
) -> Result<Module<Span>, Vec<Rich<'src, Token<'src>, Span>>> {
    let tokens = lexer::unwrap_to_vec(input);
    let input = Stream::from_iter(tokens).spanned(Span::eoi(input));
    stmt::module().parse(input).into_result()
}

/// Parse a string into a Kali expression.
pub fn parse_expr_str<'src>(
    input: &'src str,
) -> Result<kali_ast::Expr<Span>, Vec<Rich<'src, Token<'src>, Span>>> {
    let tokens = lexer::unwrap_to_vec(input);
    let input = Stream::from_iter(tokens).spanned(Span::eoi(input));
    expr::expr().parse(input).into_result()
}

/// Macro to parse inline Kali code at compile time.
///
/// Usage:
/// ```
/// kali_inline! {
///     x + 1
//  }
/// ```
#[macro_export]
macro_rules! kali {
    ($($tt:tt)*) => {
        {
            // Parse the Kali code at compile time using the parser.
            // This requires the `parse_str` function to be accessible.
            // Note: This macro only works for string literals or code blocks.
            $crate::parse_str(stringify!($($tt)*))
                .expect("Failed to parse Kali code")
        }
    };
}

/// Macro to parse inline Kali expressions at compile time.
///
/// Usage:
/// ```
/// kali_expr! {
///     x + 1
/// }
/// ```
#[macro_export]
macro_rules! kali_expr {
    ($($tt:tt)*) => {
        {
            use $crate::chumsky::{Parser, input::Input};
            let input = stringify!($($tt)*);
            let tokens = $crate::lexer::unwrap_to_vec(input);
            let stream = $crate::chumsky::input::Stream::from_iter(tokens)
                .spanned($crate::span::Span::eoi(input));
            $crate::expr::expr()
                .parse(stream)
                .into_result()
                .expect("Failed to parse Kali expression")
        }
    };
}

#[cfg(test)]
mod tests {
    use kali_ast::{BinaryExpr, BinaryOp, Expr, Identifier, Literal, LiteralKind};

    use super::*;

    #[test]
    fn test_kali_expr_macro() {
        let parsed = kali_expr! { x + 1 };
        assert_eq!(
            Expr::BinaryExpr(BinaryExpr {
                meta: Span::new(0, 5),
                lhs: Box::new(Expr::Ident(Identifier {
                    value: "x".to_owned(),
                    meta: Span::new(0, 1),
                })),
                rhs: Box::new(Expr::Literal(Literal {
                    meta: Span::new(4, 5),
                    kind: LiteralKind::Natural(1)
                })),
                operator: BinaryOp::Add
            }),
            parsed
        );
    }
}
