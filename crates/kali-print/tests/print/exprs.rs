use crate::common::{assert_fixed, assert_round_trip};

#[test]
fn literals() {
    assert_fixed("let a = true\nlet b = false\nlet c = \"hi\"\nlet d = ()\nlet e = 42\n");
    // Radix prefixes and separators are not preserved.
    assert_round_trip("let a = 0xff\nlet b = 1_000", "let a = 255\nlet b = 1000\n");
    // A negative literal is negation applied to a literal.
    assert_fixed("let a = -5\n");
    assert_fixed("let a = -1.5\n");
}

#[test]
fn tuples_and_lists() {
    assert_fixed("let t = (1, 2, 3)\nlet n = ((1, 2), 3)\nlet l = [1, [2, 3], []]\n");
    assert_round_trip(
        "let t = (1, 2,)\nlet l = [1,]",
        "let t = (1, 2)\nlet l = [1]\n",
    );
}

#[test]
fn parentheses_follow_precedence() {
    assert_round_trip("let x = a + (b * c)", "let x = a + b * c\n");
    assert_fixed("let x = (a + b) * c - (d - e)\n");
    assert_round_trip("let x = a ** (b ** c)", "let x = a ** b ** c\n");
    assert_fixed("let x = (a ** b) ** c\n");
    assert_round_trip("let x = (a - b) - c", "let x = a - b - c\n");
    assert_fixed("let x = a / (b * c)\n");
}

#[test]
fn comparison_and_logic_precedence() {
    assert_round_trip("let x = (a == b) && (c != d)", "let x = a == b && c != d\n");
    assert_round_trip("let x = (a < b) == (c >= d)", "let x = a < b == c >= d\n");
    assert_round_trip("let x = a || (b && c)", "let x = a || b && c\n");
    assert_fixed("let x = (a || b) && c\n");
    assert_fixed("let x = a == (b == c)\n");
}

#[test]
fn cons_is_right_associative() {
    assert_round_trip("let x = a :: (b :: c)", "let x = a :: b :: c\n");
    assert_fixed("let x = (a :: b) :: c\n");
    assert_round_trip("let x = (a + b) :: c == l", "let x = a + b :: c == l\n");
    assert_fixed("let x = a :: (b == c)\nlet y = (a :: b) + c\n");
}

#[test]
fn unary() {
    assert_fixed("let x = !a && b\n");
    assert_fixed("let x = !(a && b)\n");
    assert_fixed("let x = -a + b\n");
    assert_fixed("let x = -(a + b)\n");
    assert_fixed("let x = a * -b\n");
    assert_fixed("let x = ~a\nlet y = +a\n");
    // Application binds tighter than negation, but a compound operand is always
    // parenthesised so it cannot be misread as `(-f) y`.
    assert_round_trip("let x = -f y", "let x = -(f y)\n");
    assert_round_trip("let x = !f a", "let x = !(f a)\n");
    assert_round_trip("let x = - -a", "let x = -(-a)\n");
    assert_fixed("let x = (-f) y\n");
}

#[test]
fn calls() {
    assert_fixed("let x = f a b\n");
    assert_round_trip("let x = (f a) b", "let x = f a b\n");
    assert_fixed("let x = f (g a)\n");
    assert_fixed("let x = f ()\n");
    assert_fixed("let x = (a -> a) 1\n");
    assert_fixed("let x = f (n - 1), [1, 2]\n");
    assert_fixed("let x = f a + g b\n");
}

#[test]
fn lambdas() {
    assert_fixed("let add = a b -> a + b\n");
    assert_fixed("let add = (a: int) b -> a + b\n");
    assert_fixed("let k = a -> b -> a\n");
    assert_fixed("let f = (a, b) -> a\n");
    assert_fixed("let f = { x, y: z } -> x + z\n");
    assert_fixed("let f = (p: (int, int)) -> p\n");
}

#[test]
fn conditional() {
    assert_round_trip(
        "let x = if a < b { a } else { b }",
        "let x = if a < b {\n\ta\n} else {\n\tb\n}\n",
    );
    assert_round_trip("let x = if a { b }", "let x = if a {\n\tb\n}\n");
    assert_round_trip(
        "let x = if a { b } else { if c { d } else { e } }",
        "let x = if a {\n\tb\n} else {\n\tif c {\n\t\td\n\t} else {\n\t\te\n\t}\n}\n",
    );
}

#[test]
fn fib() {
    assert_round_trip(
        "let fib = n -> match n { | 0 -> 0 | 1 -> 1 | n -> fib (n - 1) + fib (n - 2) }",
        "let fib = n -> match n {\n\t| 0 -> 0\n\t| 1 -> 1\n\t| n -> fib (n - 1) + fib (n - 2)\n}\n",
    );
}

#[test]
fn nested_match_is_closed_by_braces() {
    assert_fixed(
        "let m = a b -> match a {\n\t| 0 -> b\n\t| x -> match b {\n\t\t| 0 -> a\n\t\t| y -> x + y\n\t}\n}\n",
    );
}

#[test]
fn match_as_operand_needs_no_parentheses() {
    // The braces delimit the match, so `(match ..) + 1` and `match .. + 1` parse the same.
    assert_round_trip(
        "let x = (match a {\n\t| 0 -> 1\n}) + 1\n",
        "let x = match a {\n\t| 0 -> 1\n} + 1\n",
    );
    assert_round_trip(
        "let x = f (match a {\n\t| 0 -> 1\n})\n",
        "let x = f match a {\n\t| 0 -> 1\n}\n",
    );
}
