//! A lexer for the Kali programming language.

use chumsky::prelude::*;

/// Lex a source code string into a sequence of tokens.
pub fn lex<'src>(input: &'src str) -> Result<Vec<Token<'src>>, Vec<Rich<'src, char>>> {
    block().parse(input).into_result()
}

/// An enumeration of possbile tokens that can be lexed from source code.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    /// A symbol.
    Symbol(Symbol),
    /// An operator.
    Operator(Operator),
    /// A numeric literal.
    Numeric(Numeric),
    /// A keyword.
    Keyword(Keyword),
    /// An identifier.
    Identifier(&'src str),
    /// A string literal.
    String { contents: &'src str },
    /// The start of a block.
    BlockStart,
    /// The end of a block.
    BlockEnd,
}

/// Parser for tokens.
fn token<'src>() -> impl Parser<'src, &'src str, Token<'src>, extra::Err<Rich<'src, char>>> {
    choice((
        symbol().map(Token::Symbol),
        operator().map(Token::Operator),
        numeric().map(Token::Numeric),
        keyword().map(Token::Keyword),
        text::unicode::ident().map(Token::Identifier),
    ))
}

/// A parser for a sequence of tokens, ignoring whitespace characters (except for newlines).
fn block<'src>() -> impl Parser<'src, &'src str, Vec<Token<'src>>, extra::Err<Rich<'src, char>>> {
    let block = recursive(|block| {
        let indent = choice((just(' '), just('\t')))
            .repeated()
            .configure(|cfg, depth| cfg.exactly(*depth));
    });

    block.with_ctx(0usize)
}

// A numeric literal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Numeric {
    /// A natural number.
    Natural(u64),
    /// An integer number.
    Integer(i64),
    /// A floating-point number.
    Real(f64),
}

/// Parse a numeric literal.
fn numeric<'src>() -> impl Parser<'src, &'src str, Numeric, extra::Err<Rich<'src, char>>> {
    choice((
        natural().map(Numeric::Natural),
        integer().map(Numeric::Integer),
    ))
}

/// A natural literal.
fn natural<'src>() -> impl Parser<'src, &'src str, u64, extra::Err<Rich<'src, char>>> {
    choice((
        // decimal
        just("0d").ignore_then(raw_natural(10)),
        // hex
        just("0x").ignore_then(raw_natural(16)),
        // octal
        just("0o").ignore_then(raw_natural(8)),
        // binary
        just("0b").ignore_then(raw_natural(2)),
        // default to decimal
        raw_natural(10),
    ))
}

/// An integer literal.
fn integer<'src>() -> impl Parser<'src, &'src str, i64, extra::Err<Rich<'src, char>>> {
    just("-")
        .or_not()
        .map(|s| s.map(|_| -1i64).unwrap_or(1i64))
        .then(natural())
        .try_map(|(sign, value), span| {
            value
                .try_into()
                .map(|value: i64| sign * value)
                .map_err(|_| Rich::custom(span, "integer overflow"))
        })
}

/// A raw natural literal.
fn raw_natural<'src>(base: u32) -> impl Parser<'src, &'src str, u64, extra::Err<Rich<'src, char>>> {
    text::digits(base)
        .collect::<String>()
        .separated_by(just('_').repeated())
        .allow_trailing()
        .collect::<Vec<_>>()
        .try_map(move |segments, span| {
            u64::from_str_radix(&segments.join(""), base)
                .map_err(|_| Rich::custom(span, "failed to parse"))
        })
}

/// A source code symbol. These are distinct from operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Bang,
    Star,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Pipe,
    Arrow,
    Colon,
}

fn symbol<'src>() -> impl Parser<'src, &'src str, Symbol, extra::Err<Rich<'src, char>>> {
    choice((
        just('!').to(Symbol::Bang),
        just('*').to(Symbol::Star),
        just('(').to(Symbol::LeftParen),
        just(')').to(Symbol::RightParen),
        just('[').to(Symbol::LeftBracket),
        just(']').to(Symbol::RightBracket),
        just('{').to(Symbol::LeftBrace),
        just('}').to(Symbol::RightBrace),
        just('|').to(Symbol::Pipe),
        just("->").to(Symbol::Arrow),
        just(':').to(Symbol::Colon),
    ))
}

/// An enumeration of operator tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
}

fn operator<'src>() -> impl Parser<'src, &'src str, Operator, extra::Err<Rich<'src, char>>> {
    choice((
        choice((
            just('+').to(Operator::Add),
            just('-').to(Operator::Sub),
            just('*').to(Operator::Mul),
            just('/').to(Operator::Div),
            just('%').to(Operator::Mod),
            just('^').to(Operator::Pow),
            just('=').to(Operator::Eq),
            just("!=").to(Operator::Ne),
            just('<').to(Operator::Lt),
            just("<=").to(Operator::Le),
            just('>').to(Operator::Gt),
            just(">=").to(Operator::Ge),
            just("&&").to(Operator::And),
            just("||").to(Operator::Or),
            just('!').to(Operator::Not),
            just("land").to(Operator::BitAnd),
            just("lor").to(Operator::BitOr),
            just("lxor").to(Operator::BitXor),
            just("lnot").to(Operator::BitNot),
            just("<<").to(Operator::Shl),
            just(">>").to(Operator::Shr),
            just(":=").to(Operator::Assign),
            just("+=").to(Operator::AddAssign),
            just("-=").to(Operator::SubAssign),
            just("*=").to(Operator::MulAssign),
            just("/=").to(Operator::DivAssign),
        )),
        choice((
            just("%=").to(Operator::ModAssign),
            just("^=").to(Operator::PowAssign),
            just("&=").to(Operator::AndAssign),
            just("|=").to(Operator::OrAssign),
            just("^=").to(Operator::XorAssign),
            just("<<=").to(Operator::ShlAssign),
            just(">>=").to(Operator::ShrAssign),
        )),
    ))
}

/// A keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    If,
    Else,
    Match,
    Let,
    Fn,
    With,
}

fn keyword<'src>() -> impl Parser<'src, &'src str, Keyword, extra::Err<Rich<'src, char>>> {
    choice((
        just("if").to(Keyword::If),
        just("else").to(Keyword::Else),
        just("match").to(Keyword::Match),
        just("let").to(Keyword::Let),
        just("fn").to(Keyword::Fn),
        just("with").to(Keyword::With),
    ))
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    #[test]
    fn raw_natural() {
        assert_eq!(123, super::raw_natural(10).parse("123").unwrap());
        assert_eq!(123, super::raw_natural(10).parse("1_2_3").unwrap());
        assert_eq!(123, super::raw_natural(10).parse("1_2_3_").unwrap());
        assert_eq!(123, super::raw_natural(10).parse("1_2_3__").unwrap());

        assert!(super::raw_natural(10).parse("_1234").has_errors());
        assert!(super::raw_natural(10).parse("_").has_errors());
    }

    #[test]
    fn natural() {
        // decimal
        assert_eq!(123, super::natural().parse("123").unwrap());
        assert_eq!(123, super::natural().parse("0d123").unwrap());

        // hex
        assert_eq!(0x123, super::natural().parse("0x123").unwrap());
        assert_eq!(0x123, super::natural().parse("0x1_2_3").unwrap());

        // octal
        assert_eq!(0o123, super::natural().parse("0o123").unwrap());
        assert_eq!(0o123, super::natural().parse("0o1_2_3").unwrap());

        // binary
        assert_eq!(0b101, super::natural().parse("0b101").unwrap());

        // errors
        assert!(super::natural().parse("0x_1234").has_errors());
    }

    #[test]
    fn keyword() {
        assert_eq!(super::Keyword::If, super::keyword().parse("if").unwrap());
        assert_eq!(
            super::Keyword::Else,
            super::keyword().parse("else").unwrap()
        );
        assert_eq!(
            super::Keyword::Match,
            super::keyword().parse("match").unwrap()
        );
        assert_eq!(super::Keyword::Let, super::keyword().parse("let").unwrap());
        assert_eq!(super::Keyword::Fn, super::keyword().parse("fn").unwrap());
    }

    #[test]
    fn example_add() {
        let input = include_str!("../../../examples/fib.flx");
        let tokens = super::lex(input).unwrap();
        println!("{:#?}", tokens);
    }
}
