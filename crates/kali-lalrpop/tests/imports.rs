use assert_matches::assert_matches;
use kali_lalrpop::ast::{ImportTree, ImportTreeKind, Span};

macro_rules! parse {
    ($input:expr) => {{
        let parser = kali_lalrpop::grammar::ImportTreeParser::new();
        let mut cache = lasso::Rodeo::new();
        let tokens = kali_lalrpop::lexer::Lexer::new($input);
        parser.parse(&mut cache, tokens).unwrap()
    }};
}

#[test]
fn test_parse_import_item() {
    assert_matches!(
        parse!("module"),
        ImportTree {
            kind: ImportTreeKind::Item { alias: None, .. },
            span: Span { start: 0, end: 6 }
        }
    );
    assert_matches!(
        parse!("module as alias"),
        ImportTree {
            kind: ImportTreeKind::Item { alias: Some(_), .. },
            span: Span { start: 0, end: 15 }
        }
    );
}

#[test]
fn test_parse_import_segment() {
    assert_matches!(
        parse!("module::submodule"),
        ImportTree {
            kind: ImportTreeKind::Segment { .. },
            span: Span { start: 0, end: 17 }
        }
    );
    assert_matches!(
        parse!("module::submodule as alias"),
        ImportTree {
            kind: ImportTreeKind::Segment { child, .. },
            span: Span { start: 0, end: 26 }
        } => {
            assert_matches!(
                *child,
                ImportTree {
                    kind: ImportTreeKind::Item { alias: Some(_), .. },
                    span: Span { start: 8, end: 26 }
                }
            );
        }
    );
}

#[test]
fn test_parse_import_glob() {
    assert_matches!(
        parse!("module::*"),
        ImportTree {
            kind: ImportTreeKind::Segment { child, .. },
            span: Span { start: 0, end: 9 }
        } => {
            assert_matches!(
                *child,
                ImportTree {
                    kind: ImportTreeKind::Glob,
                    span: Span { start: 8, end: 9 }
                }
            );
        }
    );
}
