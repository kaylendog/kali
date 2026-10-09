use crate::common::{assert_eq, parses, rejects, sexp};

#[test]
fn literals() {
    assert_eq!(sexp("42"), "42");
    assert_eq!(sexp("0xff"), "255");
    assert_eq!(sexp("10.5"), "10.5");
    assert_eq!(sexp("true"), "true");
    assert_eq!(sexp("\"hi\""), "\"hi\"");
    assert_eq!(sexp("()"), "()");
}

#[test]
fn negative_literal_is_negation() {
    assert_eq!(sexp("-5"), "(- 5)");
    assert_eq!(sexp("n-1"), "(- n 1)");
    assert_eq!(sexp("n -1"), "(- n 1)");
}

#[test]
fn arithmetic_precedence() {
    assert_eq!(sexp("a + b * c"), "(+ a (* b c))");
    assert_eq!(sexp("a * b + c"), "(+ (* a b) c)");
    assert_eq!(sexp("a - b - c"), "(- (- a b) c)");
    assert_eq!(sexp("a / b % c"), "(% (/ a b) c)");
    assert_eq!(sexp("a ** b ** c"), "(** a (** b c))");
    assert_eq!(sexp("a * b ** c"), "(* a (** b c))");
    assert_eq!(sexp("-a ** b"), "(** (- a) b)");
}

#[test]
fn comparison_and_logic_precedence() {
    assert_eq!(sexp("a + b < c"), "(< (+ a b) c)");
    assert_eq!(sexp("a < b == c"), "(== (< a b) c)");
    assert_eq!(sexp("a == b && c"), "(&& (== a b) c)");
    assert_eq!(sexp("a && b || c"), "(|| (&& a b) c)");
    assert_eq!(sexp("a || b && c"), "(|| a (&& b c))");
    assert_eq!(sexp("!a && b"), "(&& (! a) b)");
    assert_eq!(sexp("a <= b >= c"), "(>= (<= a b) c)");
    assert_eq!(sexp("a != b"), "(!= a b)");
}

#[test]
fn cons_sits_between_arithmetic_and_comparison() {
    assert_eq!(sexp("a :: b :: c"), "(:: a (:: b c))");
    assert_eq!(sexp("a + b :: c"), "(:: (+ a b) c)");
    assert_eq!(sexp("x :: rest == l"), "(== (:: x rest) l)");
    assert_eq!(sexp("a :: b < c || d"), "(|| (< (:: a b) c) d)");
}

#[test]
fn unary_operators() {
    assert_eq!(sexp("~a"), "(~ a)");
    assert_eq!(sexp("+a"), "(+ a)");
    assert_eq!(sexp("- -a"), "(- (- a))");
    assert_eq!(sexp("!f a"), "(! (call f [a]))");
    assert_eq!(sexp("-f x + 1"), "(+ (- (call f [x])) 1)");
    assert_eq!(sexp("(-f) x"), "(call (- f) [x])");
}

#[test]
fn unsupported_operators_are_rejected() {
    rejects("let x = a & b");
    rejects("let x = a | b");
    rejects("let x = a ^ b");
    rejects("let x = a << b");
    rejects("let x = a @ b");
}

#[test]
fn application_is_curried() {
    assert_eq!(sexp("f a"), "(call f [a])");
    assert_eq!(sexp("f a b"), "(call (call f [a]) [b])");
    assert_eq!(sexp("f (g a)"), "(call f [(call g [a])])");
    assert_eq!(sexp("f ()"), "(call f [])");
    assert_eq!(sexp("f a + g b"), "(+ (call f [a]) (call g [b]))");
    assert_eq!(sexp("(a -> a) 1"), "(call (lambda 1 a) [1])");
}

#[test]
fn uncurried_call_with_commas() {
    assert_eq!(sexp("f a, b"), "(call f [a b])");
    assert_eq!(sexp("f (n - 1), [1, 2]"), "(call f [(- n 1) (list 1 2)])");
}

#[test]
fn tuples_and_lists() {
    assert_eq!(sexp("(1, 2)"), "(tuple 1 2)");
    assert_eq!(sexp("((1, 2), 3)"), "(tuple (tuple 1 2) 3)");
    assert_eq!(sexp("(1)"), "1");
    assert_eq!(sexp("[1, [2], []]"), "(list 1 (list 2) (list ))");
    assert_eq!(sexp("[]"), "(list )");
    assert_eq!(sexp("(1, 2,)"), "(tuple 1 2)");
    assert_eq!(sexp("[1,]"), "(list 1)");
    rejects("let x = (,)");
    rejects("let x = (1,)");
}

#[test]
fn lambdas() {
    assert_eq!(sexp("a b -> a + b"), "(lambda 2 (+ a b))");
    assert_eq!(sexp("a -> b -> a"), "(lambda 1 (lambda 1 a))");
    assert_eq!(sexp("(a: int) b -> a"), "(lambda 2 a)");
    assert_eq!(sexp("(a, b) -> a"), "(lambda 1 a)");
    assert_eq!(sexp("{ x, y: z } -> x"), "(lambda 1 x)");
    rejects("let f = -> a");
    rejects("let f = a: int -> a");
}

#[test]
fn conditionals() {
    parses("let x = if a { b } else { c }");
    parses("let x = if a { b }");
    parses("let x = if a { b } else { if c { d } else { e } }");
    parses("let x = if a { b } else if c { d } else { e }");
    parses("let x = if a { b } else if c { d }");
    rejects("let x = if a { b } else c");
    rejects("let x = if a then b else c");
}

#[test]
fn matches() {
    parses("let x = match a { | 0 -> 1 | _ -> 2 }");
    rejects("let x = match a { }");
    rejects("let x = match a { 0 -> 1 }");
    rejects("let x = match a with | 0 -> 1");
}

#[test]
fn records_and_field_access_are_not_parsed() {
    rejects("let x = { x: 10, y: 20 }");
    rejects("let x = p.x");
}
