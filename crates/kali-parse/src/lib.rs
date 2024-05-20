use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

#[cfg(test)]
mod tests {
    use kali_ast::{BinaryExpr, BinaryOperator, Expr, Literal};

    #[test]
    fn test_parse_binary() {
        use crate::grammar::ExprParser;

        let input = "1 + 2";
        let expr = ExprParser::new().parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryExpr(BinaryExpr {
                lhs: Box::new(Expr::Literal(Literal::Int(1))),
                operator: BinaryOperator::Add,
                rhs: Box::new(Expr::Literal(Literal::Int(2))),
            })
        );
    }
}
