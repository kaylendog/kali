use crate::{Expr, Node, TypeExpr};

/// A named function declaration.
#[derive(Debug, Clone)]
pub struct FuncDecl<Meta = ()> {
    /// The name of the function.
    pub name: String,
    /// The parameters of the function.
    pub params: Vec<Node<FuncDeclParam, Meta>>,
    /// The return type of the function.
    pub ret_ty: Option<Node<TypeExpr, Meta>>,
    /// The body of the function.
    pub body: Node<Expr, Meta>,
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
