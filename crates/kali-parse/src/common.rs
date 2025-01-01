//! Defines a parser for identifiers.

use chumsky::{input::ValueInput, prelude::*};
use extra::ParserExtra;
use kali_ast::{Node, Span};

use crate::lexer::Token;

/// Parses an identifier.
pub fn ident<'src, I>(
) -> impl Parser<'src, I, String, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    select! { Token::Ident(s) => s.to_owned() }.labelled("identifier")
}

/// A utility trait for extending parsers.
pub trait ParserExt<'src, I, O, E>: Parser<'src, I, O, E>
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
{
    fn node(self) -> impl Parser<'src, I, Node<O>, E> + Clone
    where
        Self: Sized + Clone,
    {
        self.map_with(|s, e| Node::new(s, e.span()))
    }

    /// Produces a parser capable of parsing a left-associative binary operation.
    fn operationl<A, AO, F>(self, op: A, f: F) -> impl Parser<'src, I, O, E> + Clone
    where
        Self: Sized + Clone,
        I: Input<'src, Token = Token<'src>>,
        A: Parser<'src, I, AO, E> + Clone,
        F: Fn(O, (AO, O)) -> O + Clone,
    {
        choice((
            // A -> A op B | B
            self.clone().foldl(op.then(self.clone()).repeated(), f),
            self.clone(),
        ))
    }

    /// Produces a parser capable of parsing a right-associative binary operation.
    fn operationr<A, AO, F>(self, op: A, f: F) -> impl Parser<'src, I, O, E> + Clone
    where
        Self: Sized + Clone,
        I: Input<'src, Token = Token<'src>>,
        A: Parser<'src, I, AO, E> + Clone,
        F: Fn((O, AO), O) -> O + Clone,
    {
        choice((
            // A -> B op A | A
            self.clone().then(op).repeated().foldr(self.clone(), f),
            self.clone(),
        ))
    }
}

impl<'src, I, O, E, P> ParserExt<'src, I, O, E> for P
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
    P: Parser<'src, I, O, E>,
{
}
