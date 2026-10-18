use kali_type::Error;

/// Utility function to infer the type of an expression from a string
/// representation, returning the string representation of the inferred type.
/// Panics on any parse or type error, so a test cannot pass by accident on a
/// poisoned `Type::Error`.
pub fn infer(src: &str) -> String {
    let (module, errors) = kali_parse::parse_str(src);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let module = module.unwrap();
    let infer = kali_type::check(&module);

    assert!(infer.errors.is_empty(), "type errors: {:?}", infer.errors);

    // Lookup the identifier of the last definition in the module
    let last_definition =
        module.items.iter().rev().find_map(|item| item.kind.as_definition()).unwrap();
    let ident = match last_definition.name.kind {
        kali_ast::PatternKind::Var(ident) => ident,
        _ => panic!("Expected a variable definition, found {:?}", last_definition.name.kind),
    };

    // Return a string representation of the inferred type of the identifier.
    infer.scope.lookup_value(ident.key).unwrap().display(&module.cache).to_string()
}

/// Like [`infer`], but expects a failure.
pub fn fails(src: &str) -> Vec<Error> {
    let (module, errors) = kali_parse::parse_str(src);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let module = module.unwrap();
    let infer = kali_type::check(&module);
    assert!(infer.errors.len() > 0, "no type errors occured when error expected");
    kali_type::check(&module).errors
}
