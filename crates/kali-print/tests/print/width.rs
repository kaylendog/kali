//! Layout at a narrow width. Everything else in this suite fits on one line at
//! the default width, so only these tests exercise line breaking.

use kali_print::Settings;

use crate::common::{assert_eq, print_with};

const NARROW: Settings = Settings { width: 30, indent: 4, tabs: true };

/// Printing `src` at [`NARROW`] gives `expected`, and printing that again gives
/// it back.
fn assert_wraps(src: &str, expected: &str) {
    assert_eq!(print_with(src, NARROW), expected, "first print of {src:?}");
    assert_eq!(print_with(expected, NARROW), expected, "reprint of {expected:?}");
}

#[test]
fn short_lines_stay_flat() {
    assert_wraps("let x = [1, 2, 3]\n", "let x = [1, 2, 3]\n");
}

#[test]
fn lists_break_one_per_line_with_trailing_comma() {
    assert_wraps(
        "let xs = [first, second, third, fourth]\n",
        "let xs = [\n\tfirst,\n\tsecond,\n\tthird,\n\tfourth,\n]\n",
    );
    assert_wraps(
        "type P = { alpha: int, beta: string }\n",
        "type P = {\n\talpha: int,\n\tbeta: string,\n}\n",
    );
}

#[test]
fn definitions_break_after_the_equals() {
    assert_wraps("let total = alpha + beta + gamma\n", "let total =\n\talpha + beta + gamma\n");
}

#[test]
fn operators_stay_at_line_ends() {
    // `+` at the end of a line continues the item; an operand at the end would
    // terminate it.
    assert_wraps(
        "let t = aaaaaaaa + bbbbbbbb + cccccccc + dddddddd\n",
        "let t =\n\taaaaaaaa + bbbbbbbb +\n\t\tcccccccc +\n\t\tdddddddd\n",
    );
}

#[test]
fn calls_never_break() {
    assert_wraps(
        "let r = function argument_one argument_two\n",
        "let r =\n\tfunction argument_one argument_two\n",
    );
}

#[test]
fn sum_types_break_one_variant_per_line() {
    assert_wraps("data Unit = Only\n", "data Unit = Only\n");
    assert_wraps(
        "data Shape = Circle(float) | Rectangle(float, float) | Dot\n",
        "data Shape =\n\t| Circle(float)\n\t| Rectangle(float, float)\n\t| Dot\n",
    );
}

#[test]
fn block_bodies_hug_the_arrow() {
    assert_wraps(
        "let f = x -> match x {\n\t| 0 -> 1\n\t| _ -> 2\n}\n",
        "let f = x -> match x {\n\t| 0 -> 1\n\t| _ -> 2\n}\n",
    );
}
