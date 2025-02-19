use chumsky::{input::ValueInput, prelude::*};
use extra::ParserExtra;
use kali_ast::{
    BinaryExpr, BinaryOp, Call, Conditional, Expr, Literal, Match, Node, Span, UnaryExpr, UnaryOp,
};

use crate::{
    common::{ident, ParserExt},
    pattern::pattern,
    Token,
};

trait NodeExprParserExt<'src, I, E>: Parser<'src, I, Node<Expr>, E>
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
{
    fn binopl<A>(self, op: A) -> impl Parser<'src, I, Node<Expr>, E> + Clone
    where
        Self: Sized + Clone,
        I: Input<'src, Token = Token<'src>>,
        A: Parser<'src, I, BinaryOp, E> + Clone,
    {
        self.operationl(op, |lhs, (op, rhs)| {
            let span = lhs.span.extend(&rhs.span);
            Node::new(
                Expr::BinaryExpr(BinaryExpr {
                    operator: op,
                    lhs: lhs.boxed(),
                    rhs: rhs.boxed(),
                }),
                span,
            )
        })
    }

    fn binopr<A>(self, op: A) -> impl Parser<'src, I, Node<Expr>, E> + Clone
    where
        Self: Sized + Clone,
        I: Input<'src, Token = Token<'src>>,
        A: Parser<'src, I, BinaryOp, E> + Clone,
    {
        self.operationr(op, |(lhs, op), rhs| {
            let span = lhs.span.extend(&rhs.span);
            Node::new(
                Expr::BinaryExpr(BinaryExpr {
                    operator: op,
                    lhs: lhs.boxed(),
                    rhs: rhs.boxed(),
                }),
                span,
            )
        })
    }
}

impl<'src, I, E, P> NodeExprParserExt<'src, I, E> for P
where
    I: Input<'src, Span = Span>,
    E: ParserExtra<'src, I>,
    P: Parser<'src, I, Node<Expr>, E>,
{
}

pub fn literal<'src, I>(
) -> impl Parser<'src, I, Literal<()>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::LitBool(value) => Literal::Bool(value),
        Token::LitInteger(value) => Literal::Integer(value),
        Token::LitNatural(value) => Literal::Natural(value),
        Token::LitString(value) => Literal::String(value.to_string()),
        Token::LitUnit => Literal::Unit,
        Token::SymArray => Literal::Array(vec![])
    }
}

pub fn expr<'src, I>(
) -> impl Parser<'src, I, Node<Expr>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
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
            literal().map(Expr::Literal).node(),
            ident().map(Expr::Ident).node(),
        ))
        .boxed()
        .labelled("atom");

        // <unary> -> <op> <unary> | <atom>
        let unary = select! {
            Token::OpSub => UnaryOp::Negate,
            Token::OpBitNot => UnaryOp::BitwiseNot,
        }
        .node()
        .repeated()
        .foldr(atom.clone(), |op, inner| {
            let span = op.span.extend(&inner.span);
            Node::new(
                Expr::UnaryExpr(UnaryExpr {
                    operator: op.inner,
                    inner: inner.boxed(),
                }),
                span,
            )
        })
        .boxed();

        let callable = choice((
            expr.clone()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
            ident().map(Expr::Ident).node(),
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
                        .map(|v| Some(v)),
                    just(Token::SymLParen)
                        .ignore_then(just(Token::SymRParen))
                        .ignored()
                        .map(|_| None),
                )))
                .map(|(fun, args)| {
                    Expr::Call(Call {
                        fun: Box::new(fun),
                        args: args.unwrap_or_default(),
                    })
                })
                .node(),
            callable
                .then_ignore(just(Token::SymLParen))
                .then_ignore(just(Token::SymRParen))
                .map(|atom| {
                    Expr::Call(Call {
                        fun: Box::new(atom),
                        args: vec![],
                    })
                })
                .node(),
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
            .map(|((condition, body), otherwise)| {
                Expr::Conditional(Conditional {
                    condition: condition.boxed(),
                    body: body.boxed(),
                    otherwise: otherwise.boxed(),
                })
            })
            .node()
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
            .map(|(expr, branches)| {
                Expr::Match(Match {
                    expr: expr.boxed(),
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
            .node()
            .boxed();

        choice((cons, conditional, match_expr))
    })
    // box to improve performance
    .boxed()
    .labelled("expression")
}
