use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{ConstantType, TypeExpr, TypeExprKind};

use crate::{Span, Token};

pub fn ty_expr<'src, I>(
) -> impl Parser<'src, I, TypeExpr<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    any()
        .map_with(|_, e| TypeExpr {
            meta: e.span(),
            kind: TypeExprKind::Constant(ConstantType::Unit),
        })
        .labelled("type expression")
}
