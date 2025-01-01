//! Defines a parser for [`kali_ast::Pattern`].

use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{Pattern, PatternLiteral, Span};

use crate::{common::ident, lexer::Token};

/// A parser for [`kali_ast::Pattern`].
pub fn pattern<'src, I>(
) -> impl Parser<'src, I, Pattern, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    recursive(|pattern| {
        // []
        let empty_list = just(Token::SymArray).to(Pattern::EmptyList);

        let literal = select! {
            Token::LitNatural(nat) => Pattern::Literal(PatternLiteral::Natural(nat)),
            Token::LitInteger(int) => Pattern::Literal(PatternLiteral::Integer(int))
        };

        // <ident>
        let ident = ident().map(|s| Pattern::Ident(s));

        // ( (<pattern> ,) * <pattern> )
        let tuple = pattern
            .separated_by(just(Token::SymComma))
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(|p| Pattern::Tuple(p));

        let atom = choice((ident, empty_list, literal, tuple));

        // <atom> :: <cons> | <atom>
        let cons = atom
            .clone()
            .then_ignore(just(Token::OpCons))
            .repeated()
            .foldr(atom.clone(), |a, b| Pattern::Cons(Box::new(a), Box::new(b)));

        // <cons> | <atom>
        choice((cons, atom))
    })
    // boxed to improve performance
    .boxed()
    .labelled("pattern")
}

#[cfg(test)]
mod tests {
    use chumsky::{
        input::{Input, Stream},
        Parser,
    };
    use kali_ast::{Pattern, Span};

    use crate::{
        lexer::{self},
        pattern::pattern,
    };

    #[test]
    fn tuple() {
        // (a, b, c)
        let input =
            Stream::from_iter(lexer::unwrap_to_vec("(a, b, c)")).spanned(Span::eoi("(a, b, c)"));

        assert_eq!(
            pattern().parse(input).unwrap(),
            Pattern::Tuple(vec![
                Pattern::Ident("a".to_owned()),
                Pattern::Ident("b".to_owned()),
                Pattern::Ident("c".to_owned())
            ])
        );
    }

    #[test]
    fn cons() {
        // a :: b :: c :: []
        let input = "a :: b :: c :: []";
        let input = Stream::from_iter(lexer::unwrap_to_vec(input)).spanned(Span::eoi(input));

        assert_eq!(
            pattern().parse(input).unwrap(),
            Pattern::Cons(
                Box::new(Pattern::Ident("a".to_owned())),
                Box::new(Pattern::Cons(
                    Box::new(Pattern::Ident("b".to_owned())),
                    Box::new(Pattern::Cons(
                        Box::new(Pattern::Ident("c".to_owned())),
                        Box::new(Pattern::EmptyList)
                    ))
                ))
            )
        );
    }
}
