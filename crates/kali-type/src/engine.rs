//! Implements the type inferrence engine.

use std::collections::HashMap;

use kali_ast::{
    BinaryExpr, BinaryOp, Call, Conditional, Decl, Export, FuncDecl, FuncDeclParam, Identifier,
    Import, ImportKind, Lambda, Literal, LiteralKind, Match, Module, Pattern, PatternKind,
    Rewriter, TypeExpr, UnaryExpr,
};
use kali_parse::Span;
use tracing::trace;

use crate::{iter::TypeRefIterator, Constant, Context, Type, TypeInferenceError};

/// The type inferrence engine.
pub struct TypeInferenceEngine {
    limit: usize,
}

impl TypeInferenceEngine {
    /// Infer the types of a module.
    pub fn infer(module: Module<Span>) -> Result<Module<Meta>, TypeInferenceError> {
        let mut context = Context::new();
        Self::rewrite(&mut context, module)
    }
}

/// Meta information for a node.
#[derive(Debug, Clone)]
pub struct Meta {
    /// The span of the node.
    pub span: Span,
    /// The type of the node.
    pub ty: Type,
}

impl Meta {
    /// Create a new meta.
    pub fn new(span: Span) -> Self {
        Self {
            span,
            ty: Type::Never,
        }
    }

    /// Add a type to this meta.
    pub fn with_ty(self, ty: Type) -> Self {
        Self { ty, ..self }
    }
}

impl Rewriter<BinaryExpr<Span>, BinaryExpr<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: BinaryExpr<Span>,
    ) -> Result<BinaryExpr<Meta>, TypeInferenceError> {
        trace!("Rewriting BinaryExpr");

        let lhs = Self::rewrite(ctx, node.lhs)?;
        let rhs = Self::rewrite(ctx, node.rhs)?;

        // split on comparison operators
        let ty = if matches!(
            node.operator,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::LessThan
                | BinaryOp::LessThanOrEqual
                | BinaryOp::GreaterThan
                | BinaryOp::GreaterThanOrEqual
        ) {
            Type::Constant(Constant::Bool)
        } else {
            lhs.meta().ty.unify(&rhs.meta().ty, ctx).map_err(|err| {
                TypeInferenceError::UnificationFailed(
                    lhs.meta().ty.clone(),
                    rhs.meta().ty.clone(),
                    err,
                )
            })?
        };

        Ok(BinaryExpr {
            meta: Meta::new(node.meta).with_ty(ty),
            lhs,
            rhs,
            operator: node.operator,
        })
    }
}

impl Rewriter<UnaryExpr<Span>, UnaryExpr<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: UnaryExpr<Span>,
    ) -> Result<UnaryExpr<Meta>, TypeInferenceError> {
        trace!("Rewriting UnaryExpr");
        let inner = Self::rewrite(ctx, node.inner)?;
        Ok(UnaryExpr {
            meta: Meta::new(node.meta).with_ty(inner.meta().ty.clone()),
            inner,
            operator: node.operator,
        })
    }
}

macro_rules! rewrite_literal_variant {
    ($node:tt, $x:tt, $variant:ident) => {
        Literal {
            meta: Meta::new($node.meta).with_ty(Type::Constant(Constant::$variant)),
            kind: LiteralKind::$variant($x),
        }
    };
}

impl Rewriter<Literal<Span>, Literal<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: Literal<Span>,
    ) -> Result<Literal<Meta>, TypeInferenceError> {
        trace!("Rewriting Literal");
        Ok(match node.kind {
            LiteralKind::Natural(x) => rewrite_literal_variant!(node, x, Natural),
            LiteralKind::Integer(x) => rewrite_literal_variant!(node, x, Integer),
            LiteralKind::Float(x) => rewrite_literal_variant!(node, x, Float),
            LiteralKind::Bool(x) => rewrite_literal_variant!(node, x, Bool),
            LiteralKind::String(x) => rewrite_literal_variant!(node, x, String),
            LiteralKind::Unit => Literal {
                meta: Meta::new(node.meta).with_ty(Type::Constant(Constant::Unit)),
                kind: LiteralKind::Unit,
            },
            LiteralKind::Array(exprs) => {
                let exprs: Vec<_> = exprs
                    .into_iter()
                    .map(|expr| Self::rewrite(ctx, expr))
                    .collect::<Result<_, _>>()?;

                let ty = exprs.iter().map(|expr| &expr.meta().ty).fold_unify(ctx)?;

                Literal {
                    meta: Meta::new(node.meta).with_ty(Type::Array(Box::new(ty))),
                    kind: LiteralKind::Array(exprs),
                }
            }
            LiteralKind::Tuple(exprs) => {
                let exprs: Vec<_> = exprs
                    .into_iter()
                    .map(|expr| Self::rewrite(ctx, expr))
                    .collect::<Result<_, _>>()?;

                let ty = Type::Tuple(exprs.iter().map(|expr| expr.meta().ty.clone()).collect());

                Literal {
                    meta: Meta::new(node.meta).with_ty(ty),
                    kind: LiteralKind::Tuple(exprs),
                }
            }
            LiteralKind::Struct(btree_map) => {
                let btree_map: std::collections::BTreeMap<_, _> = btree_map
                    .into_iter()
                    .map(|(k, v)| Ok((k, Self::rewrite(ctx, v)?)))
                    .collect::<Result<_, _>>()?;

                let ty = Type::Record(
                    btree_map
                        .iter()
                        .map(|(k, v)| (k.clone(), v.meta().ty.clone()))
                        .collect(),
                );

                Literal {
                    meta: Meta::new(node.meta).with_ty(ty),
                    kind: LiteralKind::Struct(btree_map),
                }
            }
        })
    }
}

impl Rewriter<Identifier<Span>, Identifier<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: Identifier<Span>,
    ) -> Result<Identifier<Meta>, TypeInferenceError> {
        trace!("Rewriting Identifier");
        Ok(Identifier {
            meta: Meta::new(node.meta).with_ty(ctx.get_known(&node.value).unwrap().clone()),
            value: node.value,
        })
    }
}

impl Rewriter<Conditional<Span>, Conditional<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: Conditional<Span>,
    ) -> Result<Conditional<Meta>, TypeInferenceError> {
        trace!("Rewriting Conditional");

        let condition = Self::rewrite(ctx, node.condition)?;
        let body = Self::rewrite(ctx, node.body)?;
        let otherwise = Self::rewrite(ctx, node.otherwise)?;

        // enforce boolean condition
        condition
            .meta()
            .ty
            .unify(&Type::Constant(Constant::Bool), ctx)
            .map_err(|err| {
                TypeInferenceError::UnificationFailed(
                    condition.meta().ty.clone(),
                    Type::Constant(Constant::Bool),
                    err,
                )
            })?;

        let ty = body
            .meta()
            .ty
            .unify(&otherwise.meta().ty, ctx)
            .map_err(|err| {
                TypeInferenceError::UnificationFailed(
                    body.meta().ty.clone(),
                    otherwise.meta().ty.clone(),
                    err,
                )
            })?;

        Ok(Conditional {
            meta: Meta::new(node.meta).with_ty(ty),
            condition,
            body,
            otherwise,
        })
    }
}

impl Rewriter<Lambda<Span>, Lambda<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(ctx: &mut Context, node: Lambda<Span>) -> Result<Lambda<Meta>, TypeInferenceError> {
        trace!("Rewriting Lambda");
        todo!("lambda")
    }
}

impl Rewriter<Match<Span>, Match<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(ctx: &mut Context, node: Match<Span>) -> Result<Match<Meta>, TypeInferenceError> {
        trace!("Rewriting Match");
        let expr = Self::rewrite(ctx, node.expr)?;
        let branches: HashMap<_, _> = node
            .branches
            .into_iter()
            .map(|(pattern, expr)| {
                let pattern = Self::rewrite(ctx, pattern)?;
                let expr = Self::rewrite(ctx, expr)?;

                Ok((pattern, expr))
            })
            .collect::<Result<_, _>>()?;

        // unify all branches
        let ty = branches
            .values()
            .map(|expr| &expr.meta().ty)
            .fold_unify(ctx)?;

        Ok(Match {
            meta: Meta::new(node.meta).with_ty(ty),
            expr,
            branches,
        })
    }
}

impl Rewriter<Call<Span>, Call<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(ctx: &mut Context, node: Call<Span>) -> Result<Call<Meta>, TypeInferenceError> {
        trace!("Rewriting Call");
        let callee = Self::rewrite(ctx, node.fun)?;
        let args = node
            .args
            .into_iter()
            .map(|arg| Self::rewrite(ctx, arg))
            .collect::<Result<_, _>>()?;

        Ok(Call {
            meta: Meta::new(node.meta).with_ty(ctx.declare_inferred()),
            fun: callee,
            args,
        })
    }
}

impl Rewriter<Import<Span>, Import<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(ctx: &mut Context, node: Import<Span>) -> Result<Import<Meta>, TypeInferenceError> {
        trace!("Rewriting Import");
        Ok(Import {
            meta: Meta::new(node.meta),
            kind: Self::rewrite(ctx, node.kind)?,
        })
    }
}

impl Rewriter<ImportKind<Span>, ImportKind<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: ImportKind<Span>,
    ) -> Result<ImportKind<Meta>, TypeInferenceError> {
        trace!("Rewriting ImportKind");
        Ok(match node {
            ImportKind::Named { symbols, path } => {
                let symbols = symbols
                    .into_iter()
                    .map(|identifier| Self::rewrite(ctx, identifier))
                    .collect::<Result<_, _>>()?;
                ImportKind::Named { symbols, path }
            }
            ImportKind::Wildcard { path } => ImportKind::Wildcard { path },
            ImportKind::NamedWildcard { alias, path } => ImportKind::NamedWildcard {
                alias: Self::rewrite(ctx, alias)?,
                path,
            },
        })
    }
}

impl Rewriter<Export<Span>, Export<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(ctx: &mut Context, node: Export<Span>) -> Result<Export<Meta>, TypeInferenceError> {
        Ok(Export {
            meta: Meta::new(node.meta),
            symbols: node
                .symbols
                .into_iter()
                .map(|symbol| Self::rewrite(ctx, symbol))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Rewriter<TypeExpr<Span>, TypeExpr<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: TypeExpr<Span>,
    ) -> Result<TypeExpr<Meta>, TypeInferenceError> {
        trace!("Rewriting TypeExpr");
        todo!("type expr")
    }
}

impl Rewriter<Decl<Span>, Decl<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(ctx: &mut Context, node: Decl<Span>) -> Result<Decl<Meta>, TypeInferenceError> {
        trace!("Rewriting Decl");
        todo!("decl")
    }
}

impl Rewriter<FuncDecl<Span>, FuncDecl<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: FuncDecl<Span>,
    ) -> Result<FuncDecl<Meta>, TypeInferenceError> {
        trace!("Rewriting FuncDecl");
        let params: Vec<_> = node
            .params
            .into_iter()
            .map(|param| Self::rewrite(ctx, param))
            .collect::<Result<_, _>>()?;

        ctx.push();

        for param in &params {
            let ty = match &param.ty {
                Some(ty) => ty.into(),
                None => ctx.declare_inferred(),
            };
            ctx.declare_known(param.name.value.clone(), ty);
        }

        let body = Self::rewrite(ctx, node.body)?;

        ctx.pop();

        Ok(FuncDecl {
            meta: Meta::new(node.meta).with_ty(Type::Lambda(
                params.iter().map(|param| param.meta.ty.clone()).collect(),
                Box::new(body.meta().ty.clone()),
            )),
            name: Self::rewrite(ctx, node.name)?,
            params,
            body,
            ret_ty: node.ret_ty.map(|ty| Self::rewrite(ctx, ty)).transpose()?,
        })
    }
}

impl Rewriter<FuncDeclParam<Span>, FuncDeclParam<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: FuncDeclParam<Span>,
    ) -> Result<FuncDeclParam<Meta>, TypeInferenceError> {
        trace!("Rewriting FuncDeclParam");

        let type_expr = node.ty.map(|ty| Self::rewrite(ctx, ty)).transpose()?;
        let ty = type_expr.clone();

        Ok(FuncDeclParam {
            meta: Meta::new(node.meta),
            name: Self::rewrite(ctx, node.name)?,
            ty: type_expr,
        })
    }
}

impl Rewriter<Pattern<Span>, Pattern<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: Pattern<Span>,
    ) -> Result<Pattern<Meta>, TypeInferenceError> {
        trace!("Rewriting Pattern");
        Ok(Pattern {
            meta: Meta::new(node.meta),
            kind: Self::rewrite(ctx, node.kind)?,
        })
    }
}

impl Rewriter<PatternKind<Span>, PatternKind<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    #[tracing::instrument(skip_all)]
    fn rewrite(
        ctx: &mut Context,
        node: PatternKind<Span>,
    ) -> Result<PatternKind<Meta>, TypeInferenceError> {
        trace!("Rewriting PatternKind");
        Ok(match node {
            PatternKind::Wildcard => PatternKind::Wildcard,
            PatternKind::Literal(literal) => PatternKind::Literal(literal),
            PatternKind::Tuple(patterns) => PatternKind::Tuple(
                patterns
                    .into_iter()
                    .map(|pattern| Self::rewrite(ctx, pattern))
                    .collect::<Result<_, _>>()?,
            ),
            PatternKind::EmptyList => PatternKind::EmptyList,
            PatternKind::Cons(head, tail) => {
                PatternKind::Cons(Self::rewrite(ctx, head)?, Self::rewrite(ctx, tail)?)
            }
            PatternKind::Ident(ident) => PatternKind::Ident(Self::rewrite(ctx, ident)?),
        })
    }
}
