use kali_type::Typed;

use crate::Node;

use super::Expr;

#[derive(Clone, Debug)]
pub struct Call {
    pub fun: Box<Node<Expr>>,
    pub args: Vec<Node<Expr>>,
}

impl Typed for Call {
    fn ty(
        &self,
        context: &mut kali_type::Context,
    ) -> Result<kali_type::Type, kali_type::TypeInferenceError> {
        todo!()
    }
}
