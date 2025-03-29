//! Defines a parser for [`kali_ast::Pattern`].

use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{Pattern, PatternKind, PatternLiteralKind};

use crate::{common::identifier, lexer::Token, span::Span};

/// A parser for [`kali_ast::Pattern`].
pub fn pattern<'src, I>(
) -> impl Parser<'src, I, Pattern<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    recursive(|pattern| {
        // []
        let empty_list = just(Token::SymArray).to(PatternKind::EmptyList);

        let literal = select! {
            Token::LitNatural(nat) => PatternKind::Literal(PatternLiteralKind::Natural(nat)),
            Token::LitInteger(int) => PatternKind::Literal(PatternLiteralKind::Integer(int))
        };

        // <ident>
        let ident = identifier().map(|s| PatternKind::Ident(s));

        // ( (<pattern> ,) * <pattern> )
        let tuple = pattern
            .separated_by(just(Token::SymComma))
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(|p| PatternKind::Tuple(p));

        let atom = choice((ident, empty_list, literal, tuple));

        // <atom> :: <cons> | <atom>
        let cons = atom
            .clone()
            .then_ignore(just(Token::OpCons))
            .repeated()
            .foldr(atom.clone(), |a, b| {
                PatternKind::Cons(Box::new(a), Box::new(b))
            });

        // <cons> | <atom>
        choice((cons, atom))
    })
    .map_with(|kind, e| Pattern {
        meta: e.span(),
        kind,
    })
    // boxed to improve performance
    .boxed()
    .labelled("pattern")
}
