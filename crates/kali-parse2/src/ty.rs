//! Parsers for type items.

use chumsky::prelude::*;
use kali_ast2::{PrimitiveTypeKind, Type, TypeKind};

use crate::{State, define_parser};

define_parser!(ty_primitive: Type, {}, {
    choice((
        just("int").to(PrimitiveTypeKind::Int),
        just("float").to(PrimitiveTypeKind::Float),
        just("bool").to(PrimitiveTypeKind::Bool),
    ))
    .map_with(|contents, e| {
        let state: &mut chumsky::extra::SimpleState<State> = e.state();
        Type {
            id: state.next_id(),
            kind: TypeKind::Primitive(contents),
            span: e.span(),
        }
    })
});

define_parser!(ty_tuple: Type, {ty: Type}, {
    ty
        .separated_by(just(","))
        .at_least(2)
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just("("), just(")"))
        .map_with(|contents, e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();
            Type {
                id: state.next_id(),
                kind: TypeKind::Tuple(contents),
                span: e.span(),
            }
        })
});

define_parser!(ty_list: Type, {ty: Type, atom: Type}, {
    choice((atom, ty.delimited_by(just("("), just(")")))).foldl_with(
        just("[]").repeated().at_least(1),
        |child, _, e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();
            Type {
                id: state.next_id(),
                kind: TypeKind::List(Box::new(child)),
                span: e.span(),
            }
        },
    )
});

define_parser!(ty: Type, {}, {
    recursive(|ty| {
        let tuple = ty_tuple(ty.clone());
        let atom = choice((tuple.clone(), ty_primitive()));
        let list = ty_list(
            ty.clone(),
            atom.clone(),
        );
        choice((
            list, atom
        ))
    })
});

#[cfg(test)]
mod tests {
    use chumsky::Parser;
    use kali_ast2::{PrimitiveTypeKind, Type, TypeKind};

    use crate::{
        assert_parse_ok, span_of,
        ty::{ty_list, ty_primitive, ty_tuple},
    };

    #[test]
    fn parse_ty_primitive() {
        assert_parse_ok!(
            ty_primitive(),
            "bool",
            Type {
                id: 0,
                span: span_of!("bool"),
                kind: TypeKind::Primitive(PrimitiveTypeKind::Bool)
            }
        );
    }

    #[test]
    fn parse_ty_tuple() {
        assert_parse_ok!(
            ty_tuple(ty_primitive()),
            "(int,bool,float)",
            Type {
                id: 3,
                span: span_of!("(int,bool,float)"),
                kind: TypeKind::Tuple(vec![
                    Type {
                        id: 0,
                        span: span_of!("int", offset = 1),
                        kind: TypeKind::Primitive(PrimitiveTypeKind::Int)
                    },
                    Type {
                        id: 1,
                        span: span_of!("bool", offset = 5),
                        kind: TypeKind::Primitive(PrimitiveTypeKind::Bool)
                    },
                    Type {
                        id: 2,
                        span: span_of!("float", offset = 10),
                        kind: TypeKind::Primitive(PrimitiveTypeKind::Float)
                    }
                ])
            }
        );
    }

    #[test]
    fn parse_nested_ty_tuple() {
        assert_parse_ok!(
            ty_tuple(ty_tuple(ty_primitive()).or(ty_primitive())),
            "((int,bool),float)",
            Type {
                id: 4,
                span: span_of!("((int,bool),float)"),
                kind: TypeKind::Tuple(vec![
                    Type {
                        id: 2,
                        span: span_of!("(int,bool)", offset = 1),
                        kind: TypeKind::Tuple(vec![
                            Type {
                                id: 0,
                                span: span_of!("int", offset = 2),
                                kind: TypeKind::Primitive(PrimitiveTypeKind::Int)
                            },
                            Type {
                                id: 1,
                                span: span_of!("bool", offset = 6),
                                kind: TypeKind::Primitive(PrimitiveTypeKind::Bool)
                            }
                        ])
                    },
                    Type {
                        id: 3,
                        span: span_of!("float", offset = 12),
                        kind: TypeKind::Primitive(PrimitiveTypeKind::Float)
                    }
                ])
            }
        );
    }

    #[test]
    fn parse_ty_list() {
        assert_parse_ok!(
            ty_list(ty_primitive(), ty_primitive()),
            "int[][]",
            Type {
                id: 2,
                span: span_of!("int[][]"),
                kind: TypeKind::List(Box::new(Type {
                    id: 1,
                    span: span_of!("int[]"),
                    kind: TypeKind::List(Box::new(Type {
                        id: 0,
                        span: span_of!("int"),
                        kind: TypeKind::Primitive(PrimitiveTypeKind::Int)
                    }))
                }))
            }
        );

        assert_parse_ok!(
            ty_list(ty_primitive(), ty_tuple(ty_primitive())),
            "(bool,float)[]",
            Type {
                id: 3,
                span: span_of!("(bool,float)[]"),
                kind: TypeKind::List(Box::new(Type {
                    id: 2,
                    span: span_of!("(bool,float)", offset = 0),
                    kind: TypeKind::Tuple(vec![
                        Type {
                            id: 0,
                            span: span_of!("bool", offset = 1),
                            kind: TypeKind::Primitive(PrimitiveTypeKind::Bool)
                        },
                        Type {
                            id: 1,
                            span: span_of!("float", offset = 6),
                            kind: TypeKind::Primitive(PrimitiveTypeKind::Float)
                        }
                    ])
                }))
            }
        );
    }
}
