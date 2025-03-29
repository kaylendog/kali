use chumsky::{input::ValueInput, prelude::*};
use kali_ast::{Export, FuncDecl, FuncDeclParam, Import, ImportKind, Module, Stmt};

use crate::{common::identifier, expr::expr, ty_expr::ty_expr, Span, Token};

fn func_decl<'src, I>(
) -> impl Parser<'src, I, FuncDecl<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    let params = choice((
        identifier().map_with(|name, e| FuncDeclParam {
            meta: e.span(),
            name,
            ty: None,
        }),
        identifier()
            .then_ignore(just(Token::SymColon))
            .then(ty_expr())
            .map_with(|(name, ty), e| FuncDeclParam {
                meta: e.span(),
                name,
                ty: Some(ty),
            }),
    ))
    .labelled("parameter")
    .repeated()
    .collect::<Vec<_>>()
    .labelled("parameters");

    just(Token::KeywordFn)
        .ignore_then(identifier())
        .then(params.or_not())
        .then_ignore(just(Token::OpAssign))
        .then(expr())
        .map_with(|((name, params), body), e| FuncDecl {
            meta: e.span(),
            name,
            params: params.unwrap_or_default(),
            ret_ty: None,
            body: body.boxed(),
        })
        .labelled("function declaration")
}

pub fn import<'src, I>(
) -> impl Parser<'src, I, Import<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    just(Token::KeywordImport)
        .ignore_then(
            identifier()
                .separated_by(just(Token::SymComma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
        )
        .then_ignore(just(Token::KeywordFrom))
        .then(select! { Token::LitString(s) => s.to_owned() }.labelled("path"))
        .map_with(|(symbols, path), e| Import {
            meta: e.span(),
            kind: ImportKind::Named { symbols, path },
        })
}

pub fn export<'src, I>(
) -> impl Parser<'src, I, Export<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    just(Token::KeywordExport)
        .ignore_then(
            identifier()
                .separated_by(just(Token::SymComma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)),
        )
        .map_with(|symbols, e| Export {
            meta: e.span(),
            symbols,
        })
}

pub fn stmt<'src, I>(
) -> impl Parser<'src, I, Stmt<Span>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    let func_decl = func_decl().map(Stmt::FuncDecl);
    let import = import().map(Stmt::Import);
    let export = export().map(Stmt::Export);

    choice((func_decl, import, export)).labelled("statement")
}

pub fn module<'src, I>(
) -> impl Parser<'src, I, Module<Span>, extra::Err<Rich<'src, Token<'src>, Span>>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    stmt().repeated().collect::<Vec<_>>().map(|stmts| {
        let mut imports = vec![];
        let mut exports = vec![];
        let mut remaining = vec![];

        stmts.into_iter().for_each(|stmt| match stmt {
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
