use crate::common::infer;

#[test]
fn test_conditional_expr() {
    assert_eq!("int", infer("let x = if true { 1 } else { 2 };"));
    assert_eq!("float", infer("let x = if false { 1.0 } else { 2.0 };"));
}

#[test]
fn test_match_monomorphic() {
    assert_eq!("int", infer("let x = match 1 { 1 -> 42 | _ -> 0 };"));
    assert_eq!("float", infer("let x = match 3.14 { 3.14 -> 2.71 | _ -> 0.0 };"));
}

#[test]
fn test_match_polymorphic() {
    assert_eq!("('a, 'a) -> 'a", infer("let pick = x -> match x { (a, _) -> a | (_, a) -> a };"));
    assert_eq!(
        "['a] -> 'a",
        infer("let last = lst -> match lst { x :: [] -> x | x :: xs -> last xs };")
    );
}
