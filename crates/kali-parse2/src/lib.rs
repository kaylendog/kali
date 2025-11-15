use chumsky::{
    IterParser, Parser,
    prelude::{choice, just, recursive},
};
use kali_ast2::{
    ConstItem, Expr, ExprKind, FnItem, FnParam, Ident, ImportItem, ImportTree, Item, ItemKind,
    PrimitiveTypeKind, Span, Type, TypeAliasItem, TypeKind, Visibility,
};

/// Holds parser state, including the current unique identifier and a string interner for identifiers.
pub struct State {
    /// The next unique identifier to assign.
    current_id: u32,
    /// Interns and stores all encountered identifiers.
    identifiers: lasso::Rodeo,
}

impl State {
    /// Returns the next unique identifier and increments the internal counter.
    pub fn next_id(&mut self) -> u32 {
        let id = self.current_id;
        self.current_id += 1;
        id
    }
}

pub fn ident<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    Ident,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    chumsky::text::ident().map_with(|ident, e| {
        let state: &mut chumsky::extra::SimpleState<State> = e.state();
        Ident {
            id: state.current_id,
            index: state.identifiers.get_or_intern(ident),
            span: e.span(),
        }
    })
}

pub fn ty<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    Type,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    recursive(|ty| {
        let primitive = choice((
            just::<_, _, chumsky::extra::State<chumsky::extra::SimpleState<State>>>("int")
                .to(PrimitiveTypeKind::Int),
            just::<_, _, chumsky::extra::State<chumsky::extra::SimpleState<State>>>("float")
                .to(PrimitiveTypeKind::Float),
            just::<_, _, chumsky::extra::State<chumsky::extra::SimpleState<State>>>("bool")
                .to(PrimitiveTypeKind::Bool),
        ))
        .map(TypeKind::Primitive);

        let tuple = ty
            .clone()
            .separated_by(just(","))
            .at_least(2)
            .allow_trailing()
            .collect::<Vec<_>>()
            .map(TypeKind::Tuple);

        let atom = choice((tuple, primitive)).map_with(|kind, e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();
            Type {
                id: state.next_id(),
                kind,
                span: e.span(),
            }
        });

        let list = choice((atom.clone(), ty.delimited_by(just("("), just(")")))).foldl_with(
            just("[]").repeated().at_least(1),
            |child, _, e| {
                let state: &mut chumsky::extra::SimpleState<State> = e.state();
                Type {
                    id: state.next_id(),
                    kind: TypeKind::List(Box::new(child)),
                    span: e.span(),
                }
            },
        );

        choice((list, atom))
    })
}

pub fn fn_item<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    FnItem,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    just("fn")
        .ignore_then(ident())
        .then(fn_param().repeated().collect())
        .map_with(|(name, parameters), e| FnItem {
            id: 0,
            name,
            parameters,
            span: e.span(),
        })
}

pub fn fn_param<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    FnParam,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    ident().map_with(|name, e| FnParam {
        id: 0,
        name,
        ty: None,
        span: e.span(),
    })
}

pub fn expr<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    Expr,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    recursive(|expr| {
        just("0").map_with(|_, e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();

            Expr {
                id: state.next_id(),
                kind: ExprKind::Literal {
                    kind: kali_ast2::LiteralKind::Integer,
                },
                span: e.span(),
            }
        })
    })
}

pub fn const_item<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    ConstItem,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    just("const")
        .ignore_then(ident())
        .then_ignore(just("="))
        .then(expr())
        .map_with(|(name, expr), e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();
            ConstItem {
                content: expr,
                id: state.next_id(),
                name,
                span: e.span(),
                ty: None,
            }
        })
}

pub fn type_alias_item<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    TypeAliasItem,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    just("type")
        .ignore_then(ident())
        .then_ignore(just("="))
        .then(ty())
        .map_with(|(name, ty), e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();
            TypeAliasItem {
                id: state.next_id(),
                name,
                ty,
                span: e.span(),
            }
        })
}

pub fn import_item<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    ImportItem,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    just("import")
        .ignore_then(recursive(|it| {
            let simple = ident().map(ImportTree::Simple).boxed();
            let aliased = ident()
                .then_ignore(just("as"))
                .then(ident())
                .map(|(original, alias)| ImportTree::Aliased(original, alias))
                .boxed();

            let atom = choice((simple, aliased));

            let group = atom
                .clone()
                .or(it)
                .separated_by(just(","))
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(just("{"), just("}"))
                .map_with(|children, e| {
                    let state: &mut chumsky::extra::SimpleState<State> = e.state();
                    ImportTree::List {
                        id: state.next_id(),
                        span: e.span(),
                        children,
                    }
                })
                .boxed();

            group.or(atom)
        }))
        .map_with(|tree, e| {
            let state: &mut chumsky::extra::SimpleState<State> = e.state();

            ImportItem {
                id: state.next_id(),
                kind: tree,
                span: e.span(),
            }
        })
}

pub fn item<'src, F>() -> impl Parser<
    'src,
    chumsky::input::MappedSpan<Span, &'src str, F>,
    Item,
    chumsky::extra::State<chumsky::extra::SimpleState<State>>,
>
where
    F: Fn(chumsky::span::SimpleSpan) -> Span + 'src,
{
    let exportable = choice((
        just("export").ignored().or_not().then(choice((
            fn_item().map(ItemKind::Fn),
            const_item().map(ItemKind::Const),
            type_alias_item().map(ItemKind::TypeAlias),
        ))),
        import_item().map(|import| (None, ItemKind::Import(import))),
    ))
    .map_with(|(exported, kind), e| {
        let state: &mut chumsky::extra::SimpleState<State> = e.state();
        Item {
            id: state.next_id(),
            span: e.span(),
            visibility: match exported {
                Some(_) => Visibility::Exported,
                None => Visibility::Inherited,
            },
            kind,
        }
    });

    exportable
}
