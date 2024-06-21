use crate::{Expr, TypeExpr};

pub struct Lambda {
    pub params: Vec<String>,
    pub body: Box<Expr>,
}

pub struct Parameter {
    pub name: String,
    pub ty: Option<TypeExpr>,
}
