use crate::common::{assert_fixed, assert_round_trip};

#[test]
fn primitives() {
    assert_fixed("type A = int\ntype B = float\ntype C = bool\ntype D = string\ntype E = ()\n");
}

#[test]
fn named() {
    // Lowercase only: an uppercase name on the right is a one-variant sum type,
    // see items.
    assert_fixed("type P = point\n");
}

#[test]
fn composites() {
    assert_fixed("type T = (int, string)\ntype L = [[int]]\ntype R = { x: int, y: [bool] }\n");
    assert_round_trip("type R = { x: int, y: bool, }", "type R = { x: int, y: bool }\n");
    assert_fixed("type N = [(int, { a: point })]\n");
}

#[test]
fn annotations_in_lambda_params() {
    assert_fixed("let f = (a: [int]) (b: { x: int }) (c: (int, bool)) -> a\n");
}

#[test]
fn data() {
    assert_fixed(
        "data Shape = Circle(int) | Rectangle(int, int) | Point\nlet circle = Circle 3\nlet rect = Rectangle (2, 4)\n",
    );
}
