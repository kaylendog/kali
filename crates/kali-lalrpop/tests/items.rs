use assert_matches::assert_matches;
use kali_lalrpop::ast::{
    Ident, Item, ItemKind, PrimitiveTypeKind, Span, Type, TypeAlias, TypeKind, Visibility,
};

macro_rules! parse {
    ($($tt:tt)*) => {{
        let parser = kali_lalrpop::grammar::ItemParser::new();
        let mut cache = lasso::Rodeo::new();
        let tokens = kali_lalrpop::lexer::Lexer::new(stringify!($($tt)*));
        parser.parse(&mut cache, tokens).unwrap()
    }};
}

#[test]
fn test_parse_type_alias() {
    assert_matches!(
        parse! { type x = int },
        Item {
            kind: ItemKind::TypeAlias(TypeAlias {
                name: Ident {
                    span: Span { start: 5, end: 6 },
                    ..
                },
                ty: Type {
                    kind: TypeKind::Primitive(PrimitiveTypeKind::Integer),
                    span: Span { start: 9, end: 12 }
                },
                span: Span { start: 0, end: 12 }
            }),
            visibility: Visibility::Inherited,
            span: Span { start: 0, end: 12 }
        }
    );
}
