//! Implements a simple LL parser for the Kali language.

use chumsky::{
    error::Rich,
    input::{Input, Stream},
    Parser,
};

use kali_ast::Module;

pub mod lexer;

mod common;
mod expr;
mod pattern;
mod span;
mod stmt;
mod ty_expr;

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
