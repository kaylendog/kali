use kali_print::print_module;

fn print(src: &str) -> String {
    let (module, errors) = kali_parse::parse_str(src);
    assert!(errors.is_empty(), "{src:?} failed to parse: {errors:?}");
    let mut out = Vec::new();
    print_module(&module.unwrap(), src, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// Printing `src` gives `expected`, and printing `expected` gives it back unchanged.
fn assert_round_trip(src: &str, expected: &str) {
    assert_eq!(print(src), expected, "first print of {src:?}");
    assert_eq!(print(expected), expected, "reprint of {expected:?}");
}

#[test]
fn lambda_body_spans_binary_expr() {
    assert_round_trip("let add = a b -> a + b", "let add = a b -> a + b\n");
}

#[test]
fn lambda_param_annotation() {
    assert_round_trip(
        "let add = (a: int) b -> a + b",
        "let add = (a: int) b -> a + b\n",
    );
}

#[test]
fn parentheses_follow_precedence() {
    assert_round_trip("let x = a + (b * c)", "let x = a + b * c\n");
    assert_round_trip(
        "let x = (a + b) * c - (d - e)",
        "let x = (a + b) * c - (d - e)\n",
    );
    assert_round_trip("let x = a ** (b ** c)", "let x = a ** b ** c\n");
    assert_round_trip("let x = (a ** b) ** c", "let x = (a ** b) ** c\n");
    assert_round_trip("let x = -(f y)", "let x = -(f y)\n");
    assert_round_trip("let x = (a -> a) 1", "let x = (a -> a) 1\n");
}

#[test]
fn call_arguments_are_atoms() {
    assert_round_trip("let x = f (n - 1), [1, 2]", "let x = f (n - 1), [1, 2]\n");
    assert_round_trip("let x = f ()", "let x = f ()\n");
}

#[test]
fn fib() {
    let src = "let fib = n -> match n { | 0 -> 0 | 1 -> 1 | n -> fib (n - 1) + fib (n - 2) }";
    let expected = "\
let fib = n -> match n {
\t| 0 -> 0
\t| 1 -> 1
\t| n -> fib (n - 1) + fib (n - 2)
}
";
    assert_round_trip(src, expected);
}

#[test]
fn nested_match_is_closed_by_braces() {
    let expected = "\
let m = a b -> match a {
\t| 0 -> b
\t| x -> match b {
\t\t| 0 -> a
\t\t| y -> x + y
\t}
}
";
    assert_round_trip(expected, expected);
}

#[test]
fn conditional() {
    let expected = "\
let x = if a < b {
\ta
} else {
\tb
}
";
    assert_round_trip("let x = if a < b { a } else { b }", expected);
}

#[test]
fn patterns() {
    assert_round_trip(
        "let x = match l { | [] -> 0 | (a, _) :: rest -> 1 | 1 | 2 -> 3 | { x: 1 } -> 4 }",
        "let x = match l {\n\t| [] -> 0\n\t| (a, _) :: rest -> 1\n\t| 1 | 2 -> 3\n\t| { x: 1 } -> 4\n}\n",
    );
}

#[test]
fn items() {
    assert_round_trip(
        "import std :: { io as x, * }\ntype P = { x: int, y: int }\ntype L = [int]",
        "import std :: { io as x, * }\ntype P = { x: int, y: int }\ntype L = [int]\n",
    );
}

#[test]
fn sum_types() {
    let expected = "\
type Shape =
\t| Circle(int)
\t| Rectangle(int, int)
\t| Point
let circle = Circle 3
let rect = Rectangle (2, 4)
let area = shape -> match shape {
\t| Circle(r) -> 3 * r * r
\t| Rectangle(w, h) -> w * h
\t| Point -> 0
}
";
    assert_round_trip(expected, expected);
    // One line, no leading pipe, and a bare constructor is a one-variant sum type.
    assert_round_trip(
        "type Shape = Circle(int) | Point\ntype Foo = Bar",
        "type Shape =\n\t| Circle(int)\n\t| Point\ntype Foo =\n\t| Bar\n",
    );
}

#[test]
fn comments() {
    assert_round_trip(
        "// leading\n/// doc\nlet a = 1 // after a\nlet b = 2\n// end",
        "// leading\n/// doc\nlet a = 1 // after a\nlet b = 2\n// end\n",
    );
}

#[test]
fn blank_lines_collapse_to_one() {
    assert_round_trip(
        "\n\n// awoof\n\n\nlet add = a b -> a + b\n\n\n\nlet one = 1\n\n// end\n",
        "// awoof\n\nlet add = a b -> a + b\n\nlet one = 1\n\n// end\n",
    );
}

#[test]
fn explicit_semicolons_are_dropped() {
    assert_round_trip("let a = 1; let b = 2;", "let a = 1\nlet b = 2\n");
}

#[test]
fn line_ending_in_operator_continues() {
    assert_round_trip(
        "let x = fib (n - 1) +\n\tfib (n - 2)\nlet y = [1,\n\t2]",
        "let x = fib (n - 1) + fib (n - 2)\nlet y = [1, 2]\n",
    );
}
