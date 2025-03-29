//! Implements the type inferrence engine.

use std::cell::OnceCell;

use kali_ast::{
    BinaryExpr, Call, Conditional, Decl, Export, FuncDecl, Identifier, Import, Lambda, Literal,
    Match, Module, Rewriter, TypeExpr, UnaryExpr,
};
use kali_parse::Span;

use crate::{Type, TypeInferenceError};

/// The type inferrence engine.
pub struct TypeInferenceEngine;

impl TypeInferenceEngine {
    /// Infer the types of a module.
    pub fn infer(module: Module<Span>) -> Result<Module<Meta>, TypeInferenceError> {
        let mut context = TypeInferenceContext {};
        Self::rewrite(&mut context, module)
    }
}

/// Meta information for a node.
#[derive(Debug, Clone)]
pub struct Meta {
    /// The span of the node.
    pub span: Span,
    /// The type of the node.
    pub ty: OnceCell<Type>,
}

impl Meta {
    /// Create a new meta.
    pub fn new(span: Span) -> Self {
        Self {
            span,
            ty: OnceCell::default(),
        }
    }

    /// Add a type to this meta.
    pub fn with_ty(self, ty: OnceCell<Type>) -> Self {
        Self { ty, ..self }
    }
}

impl From<Span> for Meta {
    fn from(span: Span) -> Self {
        Self {
            span,
            ty: OnceCell::default(),
        }
    }
}

/// The type inferrence context.
pub struct TypeInferenceContext {}

impl Rewriter<BinaryExpr<Span>, BinaryExpr<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: BinaryExpr<Span>,
    ) -> Result<BinaryExpr<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<UnaryExpr<Span>, UnaryExpr<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: UnaryExpr<Span>,
    ) -> Result<UnaryExpr<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Literal<Span>, Literal<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Literal<Span>,
    ) -> Result<Literal<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Identifier<Span>, Identifier<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Identifier<Span>,
    ) -> Result<Identifier<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Conditional<Span>, Conditional<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Conditional<Span>,
    ) -> Result<Conditional<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Lambda<Span>, Lambda<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Lambda<Span>,
    ) -> Result<Lambda<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Match<Span>, Match<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Match<Span>,
    ) -> Result<Match<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Call<Span>, Call<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Call<Span>,
    ) -> Result<Call<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Import<Span>, Import<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Import<Span>,
    ) -> Result<Import<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Export<Span>, Export<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Export<Span>,
    ) -> Result<Export<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<TypeExpr<Span>, TypeExpr<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: TypeExpr<Span>,
    ) -> Result<TypeExpr<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<Decl<Span>, Decl<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: Decl<Span>,
    ) -> Result<Decl<Meta>, TypeInferenceError> {
        todo!()
    }
}

impl Rewriter<FuncDecl<Span>, FuncDecl<Meta>, TypeInferenceContext, TypeInferenceError>
    for TypeInferenceEngine
{
    fn rewrite(
        ctx: &mut TypeInferenceContext,
        node: FuncDecl<Span>,
    ) -> Result<FuncDecl<Meta>, TypeInferenceError> {
        todo!()
    }
}
