use chumsky::{input::ValueInput, prelude::*};
use kali_ast::TypeExpr;

use crate::{ident::ident, lexer::Token};

/// A parser for type expressions.
pub fn ty<'src, I>() -> impl Parser<'src, I, TypeExpr, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = crate::lexer::Token<'src>, Span = SimpleSpan>,
{
    choice((ident().map(|s| TypeExpr::Variable(s.to_owned())),))
}
