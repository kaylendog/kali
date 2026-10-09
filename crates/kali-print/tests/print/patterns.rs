use crate::common::{assert_fixed, assert_round_trip};

fn arms(arms: &str) -> String {
    format!("let x = match l {{\n{arms}}}\n")
}

#[test]
fn patterns() {
    assert_round_trip(
        "let x = match l { | [] -> 0 | (a, _) :: rest -> 1 | 1 | 2 -> 3 | { x: 1 } -> 4 }",
        &arms("\t| [] -> 0\n\t| (a, _) :: rest -> 1\n\t| 1 | 2 -> 3\n\t| { x: 1 } -> 4\n"),
    );
}

#[test]
fn literal_patterns() {
    assert_fixed(&arms("\t| true -> 1\n\t| \"s\" -> 2\n\t| () -> 3\n"));
}

#[test]
fn constructor_patterns() {
    assert_fixed(&arms(
        "\t| Circle(r) -> r\n\t| Rectangle(w, h) -> w\n\t| Point -> 0\n\t| Some(Circle(r)) -> r\n\t| Pair((a, b), c) -> a\n",
    ));
}

#[test]
fn nested_patterns() {
    assert_fixed(&arms(
        "\t| (a, (b, _)) :: rest -> 0\n\t| { p: (a, b), q: [] } -> 1\n",
    ));
}

#[test]
fn trailing_commas_are_dropped() {
    assert_round_trip(
        "let x = match l { | (a, b,) -> 0 | { x: a, } -> 1 | Circle(r,) -> 2 }",
        &arms("\t| (a, b) -> 0\n\t| { x: a } -> 1\n\t| Circle(r) -> 2\n"),
    );
}

#[test]
fn redundant_parentheses_are_dropped() {
    assert_round_trip(
        "let x = match l { | (a) -> 0 | ((a, b)) -> 1 }",
        &arms("\t| a -> 0\n\t| (a, b) -> 1\n"),
    );
}

#[test]
fn or_binds_tighter_than_cons() {
    assert_round_trip(
        "let x = match l { | (a | b) :: c -> 0 | a :: (b | c) -> 1 | (a :: b) | c -> 2 }",
        &arms("\t| a | b :: c -> 0\n\t| a :: b | c -> 1\n\t| (a :: b) | c -> 2\n"),
    );
}
