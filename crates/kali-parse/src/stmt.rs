use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{Export, FuncDecl, FuncDeclParam, Import, Module, Node, Span, Stmt};

use crate::{
    common::{ident, ParserExt},
    expr::expr,
    ty_expr::ty_expr,
    Token,
};

fn func_decl<'src, I>(
) -> impl Parser<'src, I, FuncDecl, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    let params = choice((
        ident().map(|s| FuncDeclParam { name: s, ty: None }),
        ident()
            .then_ignore(just(Token::SymColon))
            .then(ty_expr())
            .map(|(name, ty)| FuncDeclParam { name, ty: Some(ty) }),
    ))
    .labelled("parameter")
    .repeated()
    .collect::<Vec<_>>()
    .labelled("parameters");

    ident()
        .then(params.or_not())
        .then_ignore(just(Token::OpEq))
        .then(expr())
        .map(|((name, params), body)| FuncDecl {
            name,
            params: params.unwrap_or_default(),
            ret_ty: None,
            body,
        })
        .labelled("function declaration")
}

pub fn import<'src, I>(
) -> impl Parser<'src, I, Import, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    just(Token::KeywordImport)
        .ignore_then(
            ident()
                .separated_by(just(Token::SymComma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
        )
        .then_ignore(just(Token::KeywordFrom))
        .then(select! { Token::LitString(s) => s.to_owned() }.labelled("path"))
        .map(|(symbols, path)| Import::Named { symbols, path })
}

pub fn export<'src, I>(
) -> impl Parser<'src, I, Export, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    just(Token::KeywordExport)
        .ignore_then(
            ident()
                .separated_by(just(Token::SymComma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
        )
        .map(|symbols| Export { symbols })
}

pub fn stmt<'src, I>(
) -> impl Parser<'src, I, Node<Stmt>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    let func_decl = func_decl().map(Stmt::FuncDecl);
    let import = import().map(Stmt::Import);
    let export = export().map(Stmt::Export);

    choice((func_decl, import, export))
        .node()
        .labelled("statement")
}

pub fn module<'src, I>() -> impl Parser<'src, I, Module, extra::Err<Rich<'src, Token<'src>, Span>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    stmt().repeated().collect::<Vec<_>>().map(|stmts| {
        let mut imports = vec![];
        let mut exports = vec![];
        let mut remaining = vec![];

        stmts.into_iter().for_each(|stmt| match stmt.inner {
            Stmt::Import(import) => imports.push(import),
            Stmt::Export(export) => exports.push(export),
            stmt => remaining.push(stmt),
        });

        Module {
            imports,
            exports,
            stmts: remaining,
        }
    })
}
