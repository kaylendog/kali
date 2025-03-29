//! Implements the type inferrence engine.

use kali_ast::{
    BinaryExpr, BinaryOp, Call, Conditional, Decl, Export, FuncDecl, Identifier, Import, Lambda,
    Literal, LiteralKind, Match, Module, Rewriter, TypeExpr, UnaryExpr,
};
use kali_parse::Span;

use crate::{iter::TypeRefIterator, Constant, Context, Type, TypeInferenceError};

/// The type inferrence engine.
pub struct TypeInferenceEngine;

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
    fn rewrite(
        ctx: &mut Context,
        node: BinaryExpr<Span>,
    ) -> Result<BinaryExpr<Meta>, TypeInferenceError> {
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
    fn rewrite(
        ctx: &mut Context,
        node: UnaryExpr<Span>,
    ) -> Result<UnaryExpr<Meta>, TypeInferenceError> {
        let inner = Self::rewrite(ctx, node.inner)?;
        Ok(UnaryExpr {
            meta: Meta::new(node.meta).with_ty(inner.meta().ty.clone()),
            inner,
            operator: node.operator,
        })
    }
}

impl Rewriter<Literal<Span>, Literal<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(
        ctx: &mut Context,
        node: Literal<Span>,
    ) -> Result<Literal<Meta>, TypeInferenceError> {
        Ok(match node.kind {
            LiteralKind::Natural(x) => Literal {
                meta: Meta::new(node.meta).with_ty(Type::Constant(Constant::Natural)),
                kind: LiteralKind::Natural(x),
            },
            LiteralKind::Integer(x) => Literal {
                meta: Meta::new(node.meta).with_ty(Type::Constant(Constant::Integer)),
                kind: LiteralKind::Integer(x),
            },
            LiteralKind::Float(x) => Literal {
                meta: Meta::new(node.meta).with_ty(Type::Constant(Constant::Float)),
                kind: LiteralKind::Float(x),
            },
            LiteralKind::Bool(x) => Literal {
                meta: Meta::new(node.meta).with_ty(Type::Constant(Constant::Bool)),
                kind: LiteralKind::Bool(x),
            },
            LiteralKind::String(x) => Literal {
                meta: Meta::new(node.meta).with_ty(Type::Constant(Constant::String)),
                kind: LiteralKind::String(x),
            },
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
            LiteralKind::Tuple(exprs) => todo!(),
            LiteralKind::Struct(btree_map) => todo!(),
        })
    }
}

impl Rewriter<Identifier<Span>, Identifier<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut Context,
        node: Identifier<Span>,
    ) -> Result<Identifier<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Conditional<Span>, Conditional<Meta>, Context, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut Context,
        node: Conditional<Span>,
    ) -> Result<Conditional<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Lambda<Span>, Lambda<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(ctx: &mut Context, node: Lambda<Span>) -> Result<Lambda<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Match<Span>, Match<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(ctx: &mut Context, node: Match<Span>) -> Result<Match<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Call<Span>, Call<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(ctx: &mut Context, node: Call<Span>) -> Result<Call<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Import<Span>, Import<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(ctx: &mut Context, node: Import<Span>) -> Result<Import<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Export<Span>, Export<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(ctx: &mut Context, node: Export<Span>) -> Result<Export<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<TypeExpr<Span>, TypeExpr<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(
        ctx: &mut Context,
        node: TypeExpr<Span>,
    ) -> Result<TypeExpr<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Decl<Span>, Decl<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(ctx: &mut Context, node: Decl<Span>) -> Result<Decl<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<FuncDecl<Span>, FuncDecl<Meta>, Context, TypeInferenceError> for TypeInferenceEngine {
    fn rewrite(
        ctx: &mut Context,
        node: FuncDecl<Span>,
    ) -> Result<FuncDecl<Meta>, TypeInferenceError> {
        todo!()
    }
}
