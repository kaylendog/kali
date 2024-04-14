use chumsky::prelude::*;
use kali_ast::Expr;

use crate::literal;

/// A recursive parser for an expression.
pub fn expr() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| choice([literal(expr).map(Expr::Literal)])).labelled("expression")
}
