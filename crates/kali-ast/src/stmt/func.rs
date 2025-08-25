use crate::{Expr, Identifier, TypeExpr};

/// A named function declaration.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FuncDecl<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The name of the function.
    pub name: Identifier<Meta>,
    /// The parameters of the function.
    pub params: Vec<FuncDeclParam<Meta>>,
    /// The return type of the function.
    pub ret_ty: Option<TypeExpr<Meta>>,
    /// The body of the function.
    pub body: Box<Expr<Meta>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FuncDeclParam<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The name of the parameter.
    pub name: Identifier<Meta>,
    /// The type of the parameter.
    pub ty: Option<TypeExpr<Meta>>,
}
