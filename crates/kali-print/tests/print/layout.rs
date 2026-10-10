use crate::common::{assert_eq, assert_round_trip, print};

#[test]
fn empty_module() {
    assert_eq!(print(""), "");
    assert_eq!(print("\n\n"), "");
}

#[test]
fn blank_lines_collapse_to_one() {
    assert_round_trip(
        "\n\n// awoof\n\n\nlet add = a b -> a + b\n\n\n\nlet one = 1\n\n// end\n",
        "// awoof\n\nlet add = a b -> a + b\n\nlet one = 1\n\n// end\n",
    );
    assert_round_trip("let a = 1\n\n\n", "let a = 1\n");
}

#[test]
fn explicit_semicolons_are_dropped() {
    assert_round_trip("let a = 1; let b = 2;", "let a = 1\nlet b = 2\n");
    assert_round_trip("let a = 1;\nlet b = 2", "let a = 1\nlet b = 2\n");
}

#[test]
fn line_ending_in_operator_continues() {
    assert_round_trip(
        "let x = fib (n - 1) +\n\tfib (n - 2)\nlet y = [1,\n\t2]",
        "let x = fib (n - 1) + fib (n - 2)\nlet y = [1, 2]\n",
    );
}

#[test]
fn line_ending_in_open_bracket_continues() {
    assert_round_trip(
        "let x = (\n\t1, 2)\nlet y = f (\n\ta)",
        "let x = (1, 2)\nlet y = f a\n",
    );
}

#[test]
fn line_before_closing_bracket_continues() {
    assert_round_trip(
        "let x = [1, 2\n]\nlet y = (a, b\n)",
        "let x = [1, 2]\nlet y = (a, b)\n",
    );
}

#[test]
fn variants_may_span_lines() {
    assert_round_trip("type S =\n\t| A\n\t| B", "type S = A | B\n");
    assert_round_trip("type S = A\n\t| B", "type S = A | B\n");
}

#[test]
fn match_arms_may_be_spaced_out() {
    assert_round_trip(
        "let x = match a {\n\t| 0 -> 1\n\n\t| _ -> 2\n}",
        "let x = match a {\n\t| 0 -> 1\n\t| _ -> 2\n}\n",
    );
}

#[test]
fn indentation_uses_tabs() {
    assert_round_trip(
        "let x = if a {\n    b\n} else {\n    c\n}",
        "let x = if a {\n\tb\n} else {\n\tc\n}\n",
    );
}
