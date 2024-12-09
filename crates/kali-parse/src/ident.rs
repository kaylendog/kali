use chumsky::{input::ValueInput, prelude::*};

use crate::lexer::Token;

/// A parser for identifiers.
pub fn ident<'src, I>() -> impl Parser<'src, I, &'src str, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = crate::lexer::Token<'src>, Span = SimpleSpan>,
{
    select! { Token::Identifier(name) => name }
}
