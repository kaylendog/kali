use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{FuncDecl, FuncDeclParam};

use crate::{
    ident::ident,
    lexer::{Keyword, Symbol, Token},
    ty::ty,
    whitespace::space,
};

/// A parser for function definitions.
// pub fn func<'src, I>() -> impl Parser<'src, I, FuncDecl, extra::Err<Rich<'src, Token<'src>>>>
// where
//     I: ValueInput<'src, Token = crate::lexer::Token<'src>, Span = SimpleSpan>,
// {
//     just(Token::Keyword(Keyword::Fn))
//         .ignore_then(ident())
//         .then(param().separated_by(space()))
//         .map(|(name, params)| FuncDecl {
//             name: name.to_owned(),
//             params: vec![],
//             body:
//         })
// }

/// A parser for function parameters.
fn param<'src, I>() -> impl Parser<'src, I, FuncDeclParam, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = crate::lexer::Token<'src>, Span = SimpleSpan>,
{
    choice((
        // param with type
        ident()
            .then(just(Token::Symbol(Symbol::Colon)).ignore_then(ty()))
            .map(|(name, ty)| FuncDeclParam {
                name: name.to_owned(),
                ty: Some(ty),
            }),
        // param without type
        ident().map(|s| FuncDeclParam {
            name: s.to_owned(),
            ty: None,
        }),
    ))
}
