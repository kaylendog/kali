mod common;
mod expr;
pub mod lexer;
mod pattern;
mod stmt;
mod ty_expr;

pub use lexer::{IndentLexer, Token};

use chumsky::{
    error::Rich,
    input::{Input, Stream},
    Parser,
};

use kali_ast::{Module, Span};
use lexer::unwrap_to_vec;
use stmt::module;

/// Parse a string into a Kali module.
pub fn parse_str<'src>(input: &'src str) -> Result<Module, Vec<Rich<'src, Token<'src>, Span>>> {
    let tokens = unwrap_to_vec(input);
    let input = Stream::from_iter(tokens).spanned(Span::eoi(input));
    module().parse(input).into_result()
}
