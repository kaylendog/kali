use assert_matches::assert_matches;
use kali_lalrpop::ast::{
    BinaryExprKind, BinaryOp, Expr, ExprKind, Ident, LiteralKind, MatchArm, Span,
};

macro_rules! parse {
    ($($input:tt)*) => {{
        let parser = kali_lalrpop::grammar::ExprParser::new();
        let mut cache = lasso::Rodeo::new();
        let tokens = kali_lalrpop::lexer::Lexer::new(stringify!($($input)*));
        parser.parse(&mut cache, tokens).unwrap()
    }};
}

#[test]
fn test_parse_binary_expr_simple() {
    assert_matches!(
        parse! { 0 + 0 },
        Expr {
            kind: ExprKind::BinaryExpr {
                op: BinaryOp {
                    kind: BinaryExprKind::Add,
                    span: Span { start: 2, end: 3 }
                },
                lhs,
                rhs
            },
            span: Span { start: 0, end: 5 }
        } => {
            assert_matches!(*lhs, Expr {
                kind: ExprKind::Literal { kind: LiteralKind::Natural },
                span: Span { start: 0, end: 1 }
            });
            assert_matches!(*rhs, Expr {
                kind: ExprKind::Literal { kind: LiteralKind::Natural },
                span: Span { start: 4, end: 5 }
            });
        }
    );
}

#[test]
fn test_parse_precedence_binary_expr() {
    assert_matches!(
        parse! { 0 + 0 * 0 },
        Expr {
            kind: ExprKind::BinaryExpr {
                op: BinaryOp {
                    kind: BinaryExprKind::Add,
                    span: Span { start: 2, end: 3 }
                },
                lhs,
                rhs
            },
            span: Span { start: 0, end: 9 }
        } => {
            assert_matches!(*lhs, Expr {
                kind: ExprKind::Literal { kind: LiteralKind::Natural },
                span: Span { start: 0, end: 1 }
            });
            assert_matches!(*rhs, Expr {
                kind: ExprKind::BinaryExpr {
                    op: BinaryOp {
                        kind: BinaryExprKind::Multiply,
                        span: Span { start: 6, end: 7 }
                    },
                    lhs,
                    rhs
                },
                span: Span { start: 4, end: 9 }
            } => {
                assert_matches!(*lhs, Expr {
                    kind: ExprKind::Literal { kind: LiteralKind::Natural },
                    span: Span { start: 4, end: 5 }
                });
                assert_matches!(*rhs, Expr {
                    kind: ExprKind::Literal { kind: LiteralKind::Natural },
                    span: Span { start: 8, end: 9 }
                });
            });
        }
    );
}

#[test]
fn test_parse_match_expr() {
    assert_matches!(
        parse! {
            match 0 {
                0 -> 0,
                x -> 0,
                (0, 0) -> 0,
                0 | 0 -> 0,
                { a: 0 } -> 0
            }
        },
        Expr {
            kind: ExprKind::Match {
                value,
                arms
            },
            span: Span { start: 0, end: 57 }
        } => {
            assert_matches!(*value, Expr {
                kind: ExprKind::Var(ident),
                span: Span { start: 6, end: 7 }
            } => {
                assert_matches!(ident, Ident {
                    span: Span { start, end },
                    ..
                })
            });
        }
    );
}
