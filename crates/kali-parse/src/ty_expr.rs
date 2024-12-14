use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{Span, TypeExpr};

use crate::Token;

pub fn ty_expr<'src, I>(
) -> impl Parser<'src, I, TypeExpr, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    any()
        .to(TypeExpr::Constant(kali_ast::ConstantType::Unit))
        .labelled("type expression")
}
