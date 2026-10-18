use crate::common::{assert_eq, parses, pattern_sexp, rejects};

fn pattern(src: &str) {
    parses(&format!("let x = match v {{ | {src} -> 0 }}"));
}

fn rejects_pattern(src: &str) {
    rejects(&format!("let x = match v {{ | {src} -> 0 }}"));
}

#[test]
fn atoms() {
    pattern("_");
    pattern("x");
    pattern("0");
    pattern("true");
    pattern("\"s\"");
    pattern("()");
    pattern("[]");
}

#[test]
fn tuples() {
    pattern("(a, b)");
    pattern("(a, (b, _), c)");
    pattern("(a, b,)");
    pattern("(a)");
    rejects_pattern("(,)");
}

#[test]
fn records() {
    pattern("{ x: 0, y: y }");
    pattern("{ x: 0, }");
    pattern("{ }");
    pattern("{ p: (a, b) }");
    pattern("{ x }");
}

#[test]
fn cons_is_right_associative() {
    assert_eq!(pattern_sexp("x :: rest"), "(:: x rest)");
    assert_eq!(pattern_sexp("a :: b :: rest"), "(:: a (:: b rest))");
    assert_eq!(pattern_sexp("(a, _) :: rest"), "(:: (tuple a _) rest)");
    rejects_pattern(":: a");
}

#[test]
fn cons_binds_tighter_than_or() {
    assert_eq!(pattern_sexp("1 | 2 | 3"), "(| (| 1 2) 3)");
    assert_eq!(pattern_sexp("a :: b | c"), "(| (:: a b) c)");
    assert_eq!(pattern_sexp("a | b :: c"), "(| a (:: b c))");
    assert_eq!(pattern_sexp("a :: (b | c)"), "(:: a (| b c))");
    rejects_pattern("a |");
}

#[test]
fn constructors() {
    assert_eq!(pattern_sexp("Point"), "Point");
    assert_eq!(pattern_sexp("Circle(r)"), "(Circle r)");
    assert_eq!(pattern_sexp("Rectangle(w, h)"), "(Rectangle (tuple w h))");
    assert_eq!(pattern_sexp("Rectangle(w, h,)"), "(Rectangle (tuple w h))");
    assert_eq!(pattern_sexp("Some(Circle(r))"), "(Some (Circle r))");
    assert_eq!(pattern_sexp("Pair((a, b), c)"), "(Pair (tuple (tuple a b) c))");
    assert_eq!(pattern_sexp("Some(x) | None"), "(| (Some x) None)");
    rejects_pattern("Circle()");
    rejects_pattern("Circle r");
}

#[test]
fn negative_literals() {
    assert_eq!(pattern_sexp("-1"), "-1");
    assert_eq!(pattern_sexp("-1.5"), "-1.5");
    assert_eq!(pattern_sexp("-1 | 0"), "(| -1 0)");
    rejects_pattern("-true");
    rejects_pattern("- -1");
    rejects_pattern("-x");
}

#[test]
fn rest_is_not_parsed() {
    rejects_pattern("[a, ...]");
    rejects_pattern("(a, ...)");
}

#[test]
fn let_bindings() {
    parses("let x = 1");
    parses("let (a, b) = p");
    parses("let (a, (b, c)) = p");
    parses("let { x, y } = p");
    parses("let { x: a, y: (b, c) } = p");
    parses("let { x, } = p");
    parses("let ((a)) = p");
    parses("let 1 = x");
    parses("let _ = x");
    parses("let [] = x");
    parses("let Some(a) = x");
    parses("let a :: b = x");
}
