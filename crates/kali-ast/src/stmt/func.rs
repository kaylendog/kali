use crate::{Expr, Node, TypeExpr};

/// A named function declaration.
#[derive(Debug, Clone)]
pub struct FuncDecl {
    /// The name of the function.
    pub name: String,
    /// The parameters of the function.
    pub params: Vec<FuncDeclParam>,
    /// The return type of the function.
    pub ret_ty: Option<Node<TypeExpr>>,
    /// The body of the function.
    pub body: Node<Expr>,
}

impl FuncDecl {
    /// Creates a new function declaration.
    pub fn new(
        name: String,
        params: Vec<FuncDeclParam>,
        ret_ty: Option<Node<TypeExpr>>,
        body: Node<Expr>,
    ) -> Self {
        Self {
            name,
            params,
            ret_ty,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncDeclParam {
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    pub ty: Option<TypeExpr>,
}

impl FuncDeclParam {
    /// Creates a new function declaration parameter.
    pub fn new(name: String, ty: Option<TypeExpr>) -> Self {
        Self { name, ty }
    }
}
