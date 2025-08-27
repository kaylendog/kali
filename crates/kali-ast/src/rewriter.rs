//! AST meta rewriting.

use crate::{
    BinaryExpr, Call, Conditional, Decl, Export, Expr, FuncDecl, FuncDeclParam, Identifier, Import,
    ImportKind, Lambda, Literal, LiteralKind, Match, Module, Parameter, Pattern, PatternKind, Stmt,
    TypeExpr, TypeExprKind, UnaryExpr,
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

/// A rewriter that deletes all meta information from AST nodes.
pub struct Eraser;

impl<T> Rewriter<BinaryExpr<T>, BinaryExpr<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: BinaryExpr<T>) -> Result<BinaryExpr<()>, ()> {
        let lhs = Eraser::rewrite(&mut (), node.lhs)?;
        let rhs = Eraser::rewrite(&mut (), node.rhs)?;
        Ok(BinaryExpr {
            meta: (),
            lhs,
            rhs,
            operator: node.operator,
        })
    }
}

impl<T> Rewriter<UnaryExpr<T>, UnaryExpr<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: UnaryExpr<T>) -> Result<UnaryExpr<()>, ()> {
        let inner = Eraser::rewrite(&mut (), node.inner)?;
        Ok(UnaryExpr {
            meta: (),
            inner,
            operator: node.operator,
        })
    }
}

impl<T> Rewriter<Literal<T>, Literal<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Literal<T>) -> Result<Literal<()>, ()> {
        Ok(match node.kind {
            LiteralKind::Natural(x) => Literal {
                meta: (),
                kind: LiteralKind::Natural(x),
            },
            LiteralKind::Integer(x) => Literal {
                meta: (),
                kind: LiteralKind::Integer(x),
            },
            LiteralKind::Float(x) => Literal {
                meta: (),
                kind: LiteralKind::Float(x),
            },
            LiteralKind::Bool(x) => Literal {
                meta: (),
                kind: LiteralKind::Bool(x),
            },
            LiteralKind::String(x) => Literal {
                meta: (),
                kind: LiteralKind::String(x),
            },
            LiteralKind::Unit => Literal {
                meta: (),
                kind: LiteralKind::Unit,
            },
            LiteralKind::Array(exprs) => {
                let exprs: Vec<_> = exprs
                    .into_iter()
                    .map(|expr| Eraser::rewrite(&mut (), expr))
                    .collect::<Result<_, _>>()?;
                Literal {
                    meta: (),
                    kind: LiteralKind::Array(exprs),
                }
            }
            LiteralKind::Tuple(exprs) => {
                let exprs: Vec<_> = exprs
                    .into_iter()
                    .map(|expr| Eraser::rewrite(&mut (), expr))
                    .collect::<Result<_, _>>()?;
                Literal {
                    meta: (),
                    kind: LiteralKind::Tuple(exprs),
                }
            }
            LiteralKind::Struct(btree_map) => {
                let btree_map: std::collections::BTreeMap<_, _> = btree_map
                    .into_iter()
                    .map(|(k, v)| Ok((k, Eraser::rewrite(&mut (), v)?)))
                    .collect::<Result<_, _>>()?;
                Literal {
                    meta: (),
                    kind: LiteralKind::Struct(btree_map),
                }
            }
        })
    }
}

impl<T> Rewriter<Identifier<T>, Identifier<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Identifier<T>) -> Result<Identifier<()>, ()> {
        Ok(Identifier {
            meta: (),
            value: node.value,
        })
    }
}

impl<T> Rewriter<Conditional<T>, Conditional<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Conditional<T>) -> Result<Conditional<()>, ()> {
        let condition = Eraser::rewrite(&mut (), node.condition)?;
        let body = Eraser::rewrite(&mut (), node.body)?;
        let otherwise = Eraser::rewrite(&mut (), node.otherwise)?;
        Ok(Conditional {
            meta: (),
            condition,
            body,
            otherwise,
        })
    }
}

impl<T> Rewriter<Lambda<T>, Lambda<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Lambda<T>) -> Result<Lambda<()>, ()> {
        Ok(Lambda {
            meta: (),
            params: node
                .params
                .into_iter()
                .map(|param| Eraser::rewrite(&mut (), param))
                .collect::<Result<_, _>>()?,
            body: Eraser::rewrite(&mut (), node.body)?,
        })
    }
}

impl<T> Rewriter<Parameter<T>, Parameter<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Parameter<T>) -> Result<Parameter<()>, ()> {
        Ok(Parameter {
            meta: (),
            name: node.name,
            ty: node.ty.map(|ty| Eraser::rewrite(&mut (), ty)).transpose()?,
        })
    }
}

impl<T> Rewriter<Match<T>, Match<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Match<T>) -> Result<Match<()>, ()> {
        let expr = Eraser::rewrite(&mut (), node.expr)?;
        let branches: std::collections::HashMap<_, _> = node
            .branches
            .into_iter()
            .map(|(pattern, expr)| {
                let pattern = Eraser::rewrite(&mut (), pattern)?;
                let expr = Eraser::rewrite(&mut (), expr)?;
                Ok((pattern, expr))
            })
            .collect::<Result<_, _>>()?;
        Ok(Match {
            meta: (),
            expr,
            branches,
        })
    }
}

impl<T> Rewriter<Call<T>, Call<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Call<T>) -> Result<Call<()>, ()> {
        let callee = Eraser::rewrite(&mut (), node.fun)?;
        let args = node
            .args
            .into_iter()
            .map(|arg| Eraser::rewrite(&mut (), arg))
            .collect::<Result<_, _>>()?;
        Ok(Call {
            meta: (),
            fun: callee,
            args,
        })
    }
}

impl<T> Rewriter<Import<T>, Import<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Import<T>) -> Result<Import<()>, ()> {
        Ok(Import {
            meta: (),
            kind: Eraser::rewrite(&mut (), node.kind)?,
        })
    }
}

impl<T> Rewriter<ImportKind<T>, ImportKind<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: ImportKind<T>) -> Result<ImportKind<()>, ()> {
        Ok(match node {
            ImportKind::Named { symbols, path } => {
                let symbols = symbols
                    .into_iter()
                    .map(|identifier| Eraser::rewrite(&mut (), identifier))
                    .collect::<Result<_, _>>()?;
                ImportKind::Named { symbols, path }
            }
            ImportKind::Wildcard { path } => ImportKind::Wildcard { path },
            ImportKind::NamedWildcard { alias, path } => ImportKind::NamedWildcard {
                alias: Eraser::rewrite(&mut (), alias)?,
                path,
            },
        })
    }
}

impl<T> Rewriter<Export<T>, Export<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Export<T>) -> Result<Export<()>, ()> {
        Ok(Export {
            meta: (),
            symbols: node
                .symbols
                .into_iter()
                .map(|symbol| Eraser::rewrite(&mut (), symbol))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<T> Rewriter<TypeExpr<T>, TypeExpr<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), _node: TypeExpr<T>) -> Result<TypeExpr<()>, ()> {
        todo!()
    }
}

impl<T> Rewriter<TypeExprKind<T>, TypeExprKind<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: TypeExprKind<T>) -> Result<TypeExprKind<()>, ()> {
        Ok(match node {
            TypeExprKind::Constant(constant_type) => TypeExprKind::Constant(constant_type),
            TypeExprKind::Variable(var) => TypeExprKind::Variable(var),
            TypeExprKind::Function(type_exprs, type_expr) => TypeExprKind::Function(
                type_exprs
                    .into_iter()
                    .map(|ty| Eraser::rewrite(&mut (), ty))
                    .collect::<Result<_, _>>()?,
                Eraser::rewrite(&mut (), type_expr)?,
            ),
            TypeExprKind::Tuple(type_exprs) => TypeExprKind::Tuple(
                type_exprs
                    .into_iter()
                    .map(|ty| Eraser::rewrite(&mut (), ty))
                    .collect::<Result<_, _>>()?,
            ),
            TypeExprKind::Array(type_expr) => {
                TypeExprKind::Array(Eraser::rewrite(&mut (), type_expr)?)
            }
            TypeExprKind::Record(_) => todo!(),
        })
    }
}

impl<T> Rewriter<Decl<T>, Decl<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Decl<T>) -> Result<Decl<()>, ()> {
        Ok(Decl {
            name: node.name,
            value: Eraser::rewrite(&mut (), node.value)?,
        })
    }
}

impl<T> Rewriter<FuncDecl<T>, FuncDecl<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: FuncDecl<T>) -> Result<FuncDecl<()>, ()> {
        let params: Vec<_> = node
            .params
            .into_iter()
            .map(|param| Eraser::rewrite(&mut (), param))
            .collect::<Result<_, _>>()?;
        let body = Eraser::rewrite(&mut (), node.body)?;
        Ok(FuncDecl {
            meta: (),
            name: Eraser::rewrite(&mut (), node.name)?,
            params,
            body,
            ret_ty: node
                .ret_ty
                .map(|ty| Eraser::rewrite(&mut (), ty))
                .transpose()?,
        })
    }
}

impl<T> Rewriter<FuncDeclParam<T>, FuncDeclParam<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: FuncDeclParam<T>) -> Result<FuncDeclParam<()>, ()> {
        let type_expr = node.ty.map(|ty| Eraser::rewrite(&mut (), ty)).transpose()?;
        Ok(FuncDeclParam {
            meta: (),
            name: Eraser::rewrite(&mut (), node.name)?,
            ty: type_expr,
        })
    }
}

impl<T> Rewriter<Pattern<T>, Pattern<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: Pattern<T>) -> Result<Pattern<()>, ()> {
        Ok(Pattern {
            meta: (),
            kind: Eraser::rewrite(&mut (), node.kind)?,
        })
    }
}

impl<T> Rewriter<PatternKind<T>, PatternKind<()>, (), ()> for Eraser {
    fn rewrite(_ctx: &mut (), node: PatternKind<T>) -> Result<PatternKind<()>, ()> {
        Ok(match node {
            PatternKind::Wildcard => PatternKind::Wildcard,
            PatternKind::Literal(literal) => PatternKind::Literal(literal),
            PatternKind::Tuple(patterns) => PatternKind::Tuple(
                patterns
                    .into_iter()
                    .map(|pattern| Eraser::rewrite(&mut (), pattern))
                    .collect::<Result<_, _>>()?,
            ),
            PatternKind::EmptyList => PatternKind::EmptyList,
            PatternKind::Cons(head, tail) => PatternKind::Cons(
                Eraser::rewrite(&mut (), head)?,
                Eraser::rewrite(&mut (), tail)?,
            ),
            PatternKind::Ident(ident) => PatternKind::Ident(Eraser::rewrite(&mut (), ident)?),
        })
    }
}
