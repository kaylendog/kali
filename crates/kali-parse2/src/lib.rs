use chumsky::{
    IterParser, Parser,
    prelude::{choice, just, recursive},
};
use kali_ast2::{
    ConstItem, ExportItem, FnItem, FnParam, Ident, ImportItem, Item, PrimitiveTypeKind, Type,
    TypeAliasItem, TypeKind,
};

pub fn ident<'src>() -> impl Parser<'src, &'src str, Ident> {
    chumsky::text::ident().map_with(|ident, e| Ident {
        id: 0,
        index: 0,
        span: e.span().into_range(),
    })
}

pub fn ty<'src>() -> impl Parser<'src, &'src str, Type> {
    recursive(|ty| {
        let tuple = ty
            .separated_by(just(","))
            .collect()
            .to(TypeKind::Tuple)
            .delimited_by(just("("), just(")"));

        let array = ty.delimited_by(just("["), just("]")).to(TypeKind::List);

        let ty_primitive = ty_primitive().to(TypeKind::Primitive);

        // tuple
        choice((tuple, array, ty_primitive)).map_with(|kind, e| Type {
            id: 0,
            span: e.span().into(),
            kind,
        })
    })
}

pub fn ty_primitive<'src>() -> impl Parser<'src, &'src str, PrimitiveTypeKind> {
    choice((
        just("int").to(PrimitiveTypeKind::Int),
        just("float").to(PrimitiveTypeKind::Float),
        just("bool").to(PrimitiveTypeKind::Bool),
    ))
}

pub fn fn_item<'src>() -> impl Parser<'src, &'src str, FnItem> {
    just("fn")
        .ignore_then(ident())
        .then(fn_param().repeated().collect())
        .map_with(|(name, parameters), e| FnItem {
            id: 0,
            name,
            parameters,
            span: e.span().into_range(),
        })
}

pub fn fn_param<'src>() -> impl Parser<'src, &'src str, FnParam> {
    ident().map_with(|name, e| FnParam {
        id: 0,
        name,
        ty: None,
        span: e.span().into_range(),
    })
}

pub fn const_item<'src>() -> impl Parser<'src, &'src str, ConstItem> {}

pub fn type_alias_item<'src>() -> impl Parser<'src, &'src str, TypeAliasItem> {}

pub fn import_item<'src>() -> impl Parser<'src, &'src str, ImportItem> {}

pub fn export_item<'src>() -> impl Parser<'src, &'src str, ExportItem> {}

pub fn item<'src>() -> impl Parser<'src, &'src str, Item> {
    choice((
        fn_item(),
        const_item(),
        type_alias_item(),
        import_item(),
        export_item(),
    ))
}
