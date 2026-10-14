use kali_ast::DestructorKind;
use pretty_assertions::assert_eq;

/// Utility function to infer the type of an expression from a string
/// representation, returning the string representation of the inferred type.
/// Panics on any parse or type error, so a test cannot pass by accident on a
/// poisoned `Type::Error`.
fn infer(src: &str) -> String {
    let (module, errors) = kali_parse::parse_str(src);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let module = module.unwrap();
    let infer = kali_type::check(&module);
    assert!(infer.errors.is_empty(), "type errors: {:?}", infer.errors);

    // Lookup the identifier of the last definition in the module
    let last_definition =
        module.items.iter().rev().find_map(|item| item.kind.as_definition()).unwrap();
    let ident = match last_definition.name.kind {
        DestructorKind::Var(ident) => ident,
        _ => panic!("Expected a variable definition, found {:?}", last_definition.name.kind),
    };

    // Return a string representation of the inferred type of the identifier.
    infer.scope.lookup_value(ident.key).unwrap().display(&module.cache).to_string()
}

#[test]
fn test_infer_simple() {
    assert_eq!("int", infer("let x = 42;"));
    assert_eq!("float", infer("let y = 3.14;"));
    assert_eq!("bool", infer("let z = true;"));
    assert_eq!("string", infer("let s = \"hello\";"));
}

#[test]
fn test_infer_binary_expr() {
    assert_eq!("int", infer("let x = 1 + 2;"));
    assert_eq!("float", infer("let y = 3.0 * 4.0;"));
    assert_eq!("bool", infer("let z = true && false;"));
}

#[test]
fn test_infer_lambda_call() {
    assert_eq!("num 'a => 'a -> 'a -> 'a", infer("let add = a b -> a + b;"));
    assert_eq!("int", infer("let add = a b -> a + b; let result = add 1 2;"));
    assert_eq!("float", infer("let add = a b -> a + b; let result = add 1.0 2.0;"));

    // currying
    assert_eq!("int -> int", infer("let add = a b -> a + b; let add_one = add 1;"));
}

#[test]
#[ignore = "needs SCC ordering of top-level lets"]
fn test_infer_mutual_recursion() {
    assert_eq!(
        "int -> bool",
        infer("let is_odd = n -> n == 1 || is_even (n - 1); let is_even = n -> n == 0 || is_odd (n - 1);")
    );
}
