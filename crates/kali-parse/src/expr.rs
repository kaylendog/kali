use chumsky::{input::ValueInput, prelude::*};
use extra::ParserExtra;
use kali_ast::{
    BinaryExpr, BinaryOp, Call, Conditional, Expr, Literal, LiteralKind, Match, UnaryExpr, UnaryOp,
};

use crate::{
    common::{identifier, ParserExt},
    pattern::pattern,
    Span, Token,
};

trait ExprParserExt<'src, I, E>: Parser<'src, I, Expr<Span>, E>
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
{
    fn binopl<A>(self, op: A) -> impl Parser<'src, I, Expr<Span>, E> + Clone
    where
        Self: Sized + Clone,
        I: Input<'src, Token = Token<'src>>,
        A: Parser<'src, I, BinaryOp, E> + Clone,
    {
        self.operationl(op, |lhs, (op, rhs)| {
            let span = lhs.meta().extend(rhs.meta());
            Expr::BinaryExpr(BinaryExpr {
                meta: span,
                operator: op,
                lhs: lhs.boxed(),
                rhs: rhs.boxed(),
            })
        })
    }

    fn binopr<A>(self, op: A) -> impl Parser<'src, I, Expr<Span>, E> + Clone
    where
        Self: Sized + Clone,
        I: Input<'src, Token = Token<'src>>,
        A: Parser<'src, I, BinaryOp, E> + Clone,
    {
        self.operationr(op, |(lhs, op), rhs| {
            let span = lhs.meta().extend(rhs.meta());
            Expr::BinaryExpr(BinaryExpr {
                meta: span,
                operator: op,
                lhs: lhs.boxed(),
                rhs: rhs.boxed(),
            })
        })
    }
}

impl<'src, I, E, P> ExprParserExt<'src, I, E> for P
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
    P: Parser<'src, I, Expr<Span>, E>,
{
}

pub fn literal<'src, I>(
) -> impl Parser<'src, I, Literal<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::LitBool(value) => LiteralKind::Bool(value),
        Token::LitInteger(value) => LiteralKind::Integer(value),
        Token::LitNatural(value) => LiteralKind::Natural(value),
        Token::LitString(value) => LiteralKind::String(value.to_string()),
        Token::LitUnit => LiteralKind::Unit,
        Token::SymArray => LiteralKind::Array(vec![])
    }
    .map_with(|kind, tok| Literal {
        meta: tok.span(),
        kind,
    })
}

pub fn expr<'src, I>(
) -> impl Parser<'src, I, Expr<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    recursive(|expr| {
        let atom = choice((
            // <blockstart> <expr> <blockend>
            expr.clone()
                .delimited_by(just(Token::BlockStart), just(Token::BlockEnd)),
            // ( <expr> )
            expr.clone()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
            // <literal>
            literal().map(Expr::Literal),
            identifier().map(Expr::Ident),
        ))
        .boxed()
        .labelled("atom");

        // <unary> -> <op> <unary> | <atom>
        let unary = select! {
            Token::OpSub => UnaryOp::Negate,
            Token::OpBitNot => UnaryOp::BitwiseNot,
        }
        .map_with(|op, e| (op, e.span()))
        .repeated()
        .foldr(atom.clone(), |(op, span): (UnaryOp, Span), inner| {
            let span = span.extend(inner.meta());
            Expr::UnaryExpr(UnaryExpr {
                meta: span,
                operator: op,
                inner: inner.boxed(),
            })
        })
        .boxed();

        let callable = choice((
            expr.clone()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
            identifier().map(Expr::Ident),
        ));

        let call = choice((
            callable
                .clone()
                .then(choice((
                    unary
                        .clone()
                        .repeated()
                        .at_least(1)
                        .collect::<Vec<_>>()
                        .map(Some),
                    just(Token::SymLParen)
                        .ignore_then(just(Token::SymRParen))
                        .ignored()
                        .map(|_| None),
                )))
                .map_with(|(fun, args), e| {
                    Expr::Call(Call {
                        meta: e.span(),
                        fun: Box::new(fun),
                        args: args.unwrap_or_default(),
                    })
                }),
            callable
                .then_ignore(just(Token::SymLParen))
                .then_ignore(just(Token::SymRParen))
                .map_with(|atom, e| {
                    Expr::Call(Call {
                        meta: e.span(),
                        fun: Box::new(atom),
                        args: vec![],
                    })
                }),
            atom.clone(),
        ));

        // <exp> -> <unary> ** <exp> | <unary>
        let exp = call
            .binopl(select! { Token::OpPow => BinaryOp::Exponentiate })
            .boxed();

        // <mul> -> <exp> * <mul> | <exp> / <mul> | <exp> % <mul> | <exp>
        let mul = exp
            .binopl(select! {
                Token::OpMul => BinaryOp::Multiply,
                Token::OpDiv => BinaryOp::Divide,
                Token::OpMod => BinaryOp::Modulo,
            })
            .boxed();

        // add -> <mul> + <add> | <mul> - <add> | <mul>
        let add = mul
            .binopl(select! {
                Token::OpAdd => BinaryOp::Add,
                Token::OpSub => BinaryOp::Subtract,
            })
            .boxed();

        // logical -> <add> && | || <logical> | <add>
        let logical = add
            .binopl(select! {
                Token::OpAnd => BinaryOp::LogicalAnd,
                Token::OpOr => BinaryOp::LogicalOr,
            })
            .boxed();

        // comparison -> <logical> < | <= | > | >= <logical> | <logical>
        let comparison = logical
            .binopl(select! {
                Token::SymLAngle => BinaryOp::LessThan,
                Token::OpLe => BinaryOp::LessThanOrEqual,
                Token::SymRAngle => BinaryOp::GreaterThan,
                Token::OpGe => BinaryOp::GreaterThanOrEqual,
            })
            .boxed();

        // equality -> <comparison> == | != <equality> | <comparison>
        let equality = comparison
            .binopl(select! {
                Token::OpEq => BinaryOp::Equal,
                Token::OpNe => BinaryOp::NotEqual,
            })
            .boxed();

        // <atom> :: <atom> | <atom>
        let cons = equality
            .binopr(select! { Token::OpCons => BinaryOp::Cons })
            .boxed();

        // <conditional> -> if <expr> then <expr> else <expr>
        let conditional = just(Token::KeywordIf)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::KeywordThen))
            .then(expr.clone())
            .then_ignore(just(Token::KeywordElse))
            .then(expr.clone())
            .map_with(|((condition, body), otherwise), e| {
                Expr::Conditional(Conditional {
                    meta: e.span(),
                    condition: condition.boxed(),
                    body: body.boxed(),
                    otherwise: otherwise.boxed(),
                })
            })
            .boxed();

        // <match> -> match <expr> with <branches>
        let match_expr = just(Token::KeywordMatch)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::KeywordWith))
            .then(
                // branches -> (|? <branch> -> <expr>)*
                pattern()
                    .separated_by(just(Token::SymPipe))
                    .collect::<Vec<_>>()
                    .then_ignore(just(Token::SymArrow))
                    .then(expr.clone())
                    .labelled("branch")
                    .separated_by(just(Token::SymPipe))
                    .allow_leading()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::BlockStart), just(Token::BlockEnd)),
            )
            .map_with(|(expr, branches), e| {
                Expr::Match(Match {
                    meta: e.span(),
                    expr: Box::new(expr),
                    branches: branches
                        .into_iter()
                        .flat_map(|(patterns, expr)| {
                            patterns
                                .into_iter()
                                .map(move |pattern| (pattern, expr.clone()))
                        })
                        .collect(),
                })
            })
            .boxed();

        choice((cons, conditional, match_expr))
    })
    // box to improve performance
    .boxed()
    .labelled("expression")
}
