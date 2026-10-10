use crate::common::{assert_fixed, assert_round_trip};

#[test]
fn imports() {
    // Rust style: no spaces around `::` or inside braces. Spaced input is
    // accepted.
    assert_fixed("import a\nimport a as b\nimport a::*\nimport a::b::c\n");
    assert_fixed("import a::{b, c as d, e::*}\n");
    assert_round_trip("import a :: { b, }", "import a::{b}\n");
    assert_round_trip("import std :: { io as x, * }", "import std::{io as x, *}\n");
}

#[test]
fn definitions_destructure() {
    assert_fixed("let (a, b) = (1, 2)\nlet { x, y: z } = p\nlet ((a, b), c) = t\n");
    assert_round_trip("let { x: (a, b), } = p", "let { x: (a, b) } = p\n");
}

#[test]
fn type_alias_and_decl() {
    assert_fixed("type P = { x: int, y: int }\ntype L = [int]\n");
    // A bare uppercase name is a one-variant sum type, not an alias.
    assert_fixed("type A = int\ntype B = Foo\n");
}

#[test]
fn sum_types() {
    // Short declarations stay on one line; see `width` for the vertical form.
    assert_round_trip(
        "type Shape =\n\t| Circle(int)\n\t| Rectangle(int, int)\n\t| Point\nlet circle = Circle 3\nlet rect = Rectangle (2, 4)\n",
        "type Shape = Circle(int) | Rectangle(int, int) | Point\nlet circle = Circle 3\nlet rect = Rectangle (2, 4)\n",
    );
    assert_round_trip(
        "type T = | A([int]) | B({ x: int }, bool)",
        "type T = A([int]) | B({ x: int }, bool)\n",
    );
}
