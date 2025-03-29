//! AST meta rewriting.

use crate::{
    BinaryExpr, Call, Conditional, Decl, Export, Expr, FuncDecl, Identifier, Import, Lambda,
    Literal, Match, Module, Stmt, TypeExpr, UnaryExpr,
};

/// A trait for all types that implement AST meta rewriting.
pub trait Rewriter<In, Out, Context, Error> {
    /// Rewrite the meta of a node.
    fn rewrite(ctx: &mut Context, node: In) -> Result<Out, Error>;
}

impl<In, Out, Context, Error, R> Rewriter<Box<In>, Box<Out>, Context, Error> for R
where
    R: Rewriter<In, Out, Context, Error>,
{
    fn rewrite(ctx: &mut Context, node: Box<In>) -> Result<Box<Out>, Error> {
        R::rewrite(ctx, *node).map(Box::new)
    }
}

impl<In, Out, Ctx, Error, R> Rewriter<Expr<In>, Expr<Out>, Ctx, Error> for R
where
    R: Rewriter<BinaryExpr<In>, BinaryExpr<Out>, Ctx, Error>
        + Rewriter<UnaryExpr<In>, UnaryExpr<Out>, Ctx, Error>
        + Rewriter<Literal<In>, Literal<Out>, Ctx, Error>
        + Rewriter<Identifier<In>, Identifier<Out>, Ctx, Error>
        + Rewriter<Conditional<In>, Conditional<Out>, Ctx, Error>
        + Rewriter<Lambda<In>, Lambda<Out>, Ctx, Error>
        + Rewriter<Match<In>, Match<Out>, Ctx, Error>
        + Rewriter<Call<In>, Call<Out>, Ctx, Error>,
{
    fn rewrite(ctx: &mut Ctx, node: Expr<In>) -> Result<Expr<Out>, Error> {
        match node {
            Expr::BinaryExpr(binary) => R::rewrite(ctx, binary).map(Expr::BinaryExpr),
            Expr::Literal(literal) => R::rewrite(ctx, literal).map(Expr::Literal),
            Expr::Ident(identifier) => R::rewrite(ctx, identifier).map(Expr::Ident),
            Expr::UnaryExpr(unary_expr) => R::rewrite(ctx, unary_expr).map(Expr::UnaryExpr),
            Expr::Conditional(conditional) => R::rewrite(ctx, conditional).map(Expr::Conditional),
            Expr::Lambda(lambda) => R::rewrite(ctx, lambda).map(Expr::Lambda),
            Expr::Match(node) => R::rewrite(ctx, node).map(Expr::Match),
            Expr::Call(call) => R::rewrite(ctx, call).map(Expr::Call),
        }
    }
}

impl<In, Out, Ctx, Error, R> Rewriter<Stmt<In>, Stmt<Out>, Ctx, Error> for R
where
    R: Rewriter<Import<In>, Import<Out>, Ctx, Error>
        + Rewriter<Export<In>, Export<Out>, Ctx, Error>
        // const needs Identifier and Literal
        + Rewriter<Identifier<In>, Identifier<Out>, Ctx, Error>
        + Rewriter<Literal<In>, Literal<Out>, Ctx, Error>
        // type needs Identifier and TypeExpr
        + Rewriter<TypeExpr<In>, TypeExpr<Out>, Ctx, Error>
        + Rewriter<Decl<In>, Decl<Out>, Ctx, Error>
        + Rewriter<FuncDecl<In>, FuncDecl<Out>, Ctx, Error>,
{
    fn rewrite(ctx: &mut Ctx, node: Stmt<In>) -> Result<Stmt<Out>, Error> {
        match node {
            Stmt::Import(import) => R::rewrite(ctx, import).map(Stmt::Import),
            Stmt::Export(export) => R::rewrite(ctx, export).map(Stmt::Export),
            Stmt::Const(identifier, literal) => {
                R::rewrite(ctx, identifier).and_then(|identifier| {
                    R::rewrite(ctx, literal).map(|literal| Stmt::Const(identifier, literal))
                })
            }
            Stmt::Type(identifier, type_expr) => {
                R::rewrite(ctx, identifier).and_then(|identifier| {
                    R::rewrite(ctx, type_expr).map(|type_expr| Stmt::Type(identifier, type_expr))
                })
            }
            Stmt::Decl(decl) => R::rewrite(ctx, decl).map(Stmt::Decl),
            Stmt::FuncDecl(func_decl) => R::rewrite(ctx, func_decl).map(Stmt::FuncDecl),
        }
    }
}

impl<In, Out, Ctx, Error, R> Rewriter<Module<In>, Module<Out>, Ctx, Error> for R
where
    R: Rewriter<Stmt<In>, Stmt<Out>, Ctx, Error>
        + Rewriter<Import<In>, Import<Out>, Ctx, Error>
        + Rewriter<Export<In>, Export<Out>, Ctx, Error>,
{
    fn rewrite(ctx: &mut Ctx, node: Module<In>) -> Result<Module<Out>, Error> {
        let Module {
            imports,
            exports,
            stmts,
        } = node;
        let imports = imports
            .into_iter()
            .map(|import| R::rewrite(ctx, import))
            .collect::<Result<_, _>>()?;
        let exports = exports
            .into_iter()
            .map(|export| R::rewrite(ctx, export))
            .collect::<Result<_, _>>()?;
        let stmts = stmts
            .into_iter()
            .map(|stmt| R::rewrite(ctx, stmt))
            .collect::<Result<_, _>>()?;
        Ok(Module {
            imports,
            exports,
            stmts,
        })
    }
}
