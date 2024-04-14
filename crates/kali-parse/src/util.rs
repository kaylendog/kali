use chumsky::{prelude::*, primitive::OrderedContainer, Error};

/// Optionally parses a value.
pub fn maybe<I: Clone + PartialEq, C: OrderedContainer<I> + Clone, E: Error<I>, P>(
    input: I,
    parser: P,
) -> impl Parser<I, C, Error = E>
where
    P: Parser<I, C, Error = E> + Clone,
{
    just(input).ignore_then(parser.clone()).or(parser)
}
