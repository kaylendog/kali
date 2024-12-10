use chumsky::{input::ValueInput, prelude::*};
use kali_ast::Pattern;

mod lexer;

use lexer::Token;

fn ident<'src, I>() -> impl Parser<'src, I, String, extra::Err<Rich<'src, Token<'src>>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    select! { Token::Ident(s) => s.to_owned() }
}

fn pattern<'src, I>() -> impl Parser<'src, I, Pattern, extra::Err<Rich<'src, Token<'src>>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|pattern| {
        // []
        let empty_list = just(Token::SymArray).to(Pattern::EmptyList);

        // <ident>
        let ident = ident().map(|s| Pattern::Ident(s));

        // ( (<pattern> ,) * <pattern> )
        let tuple = pattern
            .separated_by(just(Token::SymComma))
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(|p| Pattern::Tuple(p));

        let atom = choice((empty_list, ident, tuple));

        // <atom> :: <cons> | <atom>
        let cons = atom
            .clone()
            .then_ignore(just(Token::OpCons))
            .repeated()
            .foldr(atom.clone(), |a, b| Pattern::Cons(Box::new(a), Box::new(b)));

        // <cons> | <atom>
        choice((cons, atom))
    })
}

#[cfg(test)]
mod tests {

    mod pattern {
        use chumsky::{input::Stream, prelude::*};
        use kali_ast::Pattern;

        use crate::{lexer::Token, pattern};

        #[test]
        fn tuple() {
            // (a, b, c)
            let input = vec![
                Token::SymLParen,
                Token::Ident("a"),
                Token::SymComma,
                Token::Ident("b"),
                Token::SymComma,
                Token::Ident("c"),
                Token::SymRParen,
            ];

            let input = Stream::from_iter(input.into_iter());

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
            let input = vec![
                Token::Ident("a"),
                Token::OpCons,
                Token::Ident("b"),
                Token::OpCons,
                Token::Ident("c"),
                Token::OpCons,
                Token::SymArray,
            ];

            let input = Stream::from_iter(input.into_iter());

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
}
