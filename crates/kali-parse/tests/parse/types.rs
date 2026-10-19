use crate::common::{parses, rejects};

fn ty(src: &str) {
    parses(&format!("type T = {src}"));
}

#[test]
fn primitives() {
    for name in ["int", "float", "bool", "string", "()"] {
        ty(name);
    }
}

#[test]
fn named() {
    ty("point");
    ty("μέσος");
}

#[test]
fn tuples() {
    ty("(int, string)");
    ty("(int, (bool, string), [int])");
    ty("(int)");
    rejects("type T = ()()");
    rejects("type T = (int,)");
}

#[test]
fn lists() {
    ty("[int]");
    ty("[[int]]");
    ty("[(int, bool)]");
    rejects("type T = []");
    rejects("type T = [int, bool]");
}

#[test]
fn records() {
    ty("{ x: int, y: int }");
    ty("{ x: int, y: int, }");
    ty("{ }");
    ty("{ p: { x: int }, l: [bool] }");
    rejects("type T = { x int }");
    rejects("type T = { x: }");
}

#[test]
fn unsupported_type_syntax_is_rejected() {
    rejects("type T = int -> int");
    rejects("type T = (int) -> int");
    rejects("type T = list(int)");
    rejects("type T = int?");
}

#[test]
fn annotations_in_lambda_params() {
    parses("let f = (a: [int]) (b: { x: int }) (c: (int, bool)) -> a");
    rejects("let f = (a: int -> int) -> a");
}

#[test]
fn type_parameters() {
    parses("data Result<T, E> = Ok(T) | Err(E)");
    parses("type Pair<T> = (T, T)");
    parses("type ProcessResult = Result<(), Error>");
    rejects("type Pair<> = int");
    rejects("data D<T = Ok(T)");
}
