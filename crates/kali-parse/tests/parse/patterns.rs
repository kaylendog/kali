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
    // Shorthand `{ x }` is a destructor, not a pattern.
    rejects_pattern("{ x }");
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
    assert_eq!(
        pattern_sexp("Pair((a, b), c)"),
        "(Pair (tuple (tuple a b) c))"
    );
    assert_eq!(pattern_sexp("Some(x) | None"), "(| (Some x) None)");
    rejects_pattern("Circle()");
    rejects_pattern("Circle r");
}

#[test]
fn negative_literals_are_not_patterns_yet() {
    // `-1` is a negation expression, and patterns have no negation.
    rejects_pattern("-1");
}

#[test]
fn rest_is_not_parsed() {
    rejects_pattern("[a, ...]");
    rejects_pattern("(a, ...)");
}

#[test]
fn destructors() {
    parses("let x = 1");
    parses("let (a, b) = p");
    parses("let (a, (b, c)) = p");
    parses("let { x, y } = p");
    parses("let { x: a, y: (b, c) } = p");
    parses("let { x, } = p");
    parses("let ((a)) = p");
    // Destructors bind names only: no literals, wildcards or constructors.
    rejects("let 1 = x");
    rejects("let _ = x");
    rejects("let [] = x");
    rejects("let Some(a) = x");
    rejects("let a :: b = x");
}
