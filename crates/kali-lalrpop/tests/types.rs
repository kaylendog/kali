use assert_matches::assert_matches;
use kali_lalrpop::ast::{PrimitiveTypeKind, Type, TypeKind};

macro_rules! parse {
    ($input:expr) => {{
        let parser = kali_lalrpop::grammar::TypeParser::new();
        let mut cache = lasso::Rodeo::new();
        let tokens = kali_lalrpop::lexer::Lexer::new($input);
        parser.parse(&mut cache, tokens).unwrap()
    }};
}

#[test]
fn test_parse_primitives() {
    let cases = [
        ("int", PrimitiveTypeKind::Integer),
        ("float", PrimitiveTypeKind::Float),
        ("bool", PrimitiveTypeKind::Bool),
        ("string", PrimitiveTypeKind::String),
        ("()", PrimitiveTypeKind::Unit),
    ];

    for (input, expected_kind) in cases {
        assert_matches!(
            parse!(input),
            Type {
                kind: TypeKind::Primitive(kind),
                ..
            } if kind == expected_kind
        );
    }
}

#[test]
fn test_parse_tuple() {
    assert_matches!(
        parse!("(int, float, bool)"),
        Type {
            kind: TypeKind::Tuple(children),
            ..
        } => {
            assert_eq!(children.len(), 3);
        }
    );
}

#[test]
fn test_parse_nested_tuple() {
    assert_matches!(
        parse!("((int, float), bool)"),
        Type {
            kind: TypeKind::Tuple(children),
            ..
        } => {
            assert_eq!(children.len(), 2);
            assert_matches!(
                &children[0],
                Type {
                    kind: TypeKind::Tuple(children),
                    ..
                } => {
                    assert_eq!(children.len(), 2);
                    assert_matches!(
                        children[0],
                        Type {
                            kind: TypeKind::Primitive(PrimitiveTypeKind::Integer),
                            ..
                        }
                    );
                    assert_matches!(
                        children[1],
                        Type {
                            kind: TypeKind::Primitive(PrimitiveTypeKind::Float),
                            ..
                        }
                    );
                }
            );
            assert_matches!(
                children[1],
                Type {
                    kind: TypeKind::Primitive(PrimitiveTypeKind::Bool),
                    ..
                }
            );
        }
    );
}

#[test]
fn test_parse_list() {
    assert_matches!(
        parse!("[int]"),
        Type {
            kind: TypeKind::List(element),
            ..
        } => {
            assert_matches!(
                *element,
                Type {
                    kind: TypeKind::Primitive(PrimitiveTypeKind::Integer),
                    ..
                }
            );
        }
    );
}

#[test]
fn test_parse_record() {
    assert_matches!(
        parse!("{field1: int, field2: float}"),
        Type {
            kind: TypeKind::Record(entries),
            ..
        } => {
            assert_eq!(entries.len(), 2);
        }
    );
}
