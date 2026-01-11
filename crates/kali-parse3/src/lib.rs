use chumsky::{
    extra::SimpleState,
    input::{Input, MapExtra, Stream, ValueInput},
    pratt::{Associativity, infix, left, postfix, prefix, right},
    prelude::*,
};
use logos::Logos;

use crate::{
    ast::{
        BinaryOp, BinaryOpKind, Destructor, DestructorKind, Expr, ExprKind, Ident, ImportTree,
        ImportTreeKind, Item, ItemKind, LambdaParam, LiteralKind, MatchArm, Module, Pattern,
        PatternKind, PrimitiveTypeKind, Type, TypeKind, UnaryOp, UnaryOpKind, Visibility,
    },
    lexer::Token,
};

mod ast;
mod lexer;

/// Represents the state used during parsing, including a string interner for efficient string handling.
#[derive(Default)]
struct State {
    /// A `Rodeo` instance from the `lasso` crate, used for interning strings to reduce memory usage and improve performance.
    rodeo: lasso::Rodeo,
}

/// Concrete type for the parser extras.
type Extras<'src> = extra::Full<Rich<'src, Token<'src>>, SimpleState<State>, ()>;

/// Shorthand function to construct a [`BinaryOp`].
fn binary_op<'src, I>(
    op: impl Parser<'src, I, BinaryOpKind, Extras<'src>> + 'src,
) -> impl Parser<'src, I, BinaryOp, Extras<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    op.map_with(|kind, e| BinaryOp {
        kind,
        span: e.span(),
    })
    .boxed()
}

/// Shorthand function to construct a [`BinaryExpr`].
fn binary_expr<'src, I>(
    lhs: Expr,
    op: BinaryOp,
    rhs: Expr,
    e: &mut MapExtra<'src, '_, I, Extras<'src>>,
) -> Expr
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    Expr {
        kind: ExprKind::BinaryExpr {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        span: e.span(),
    }
}

pub(crate) fn parser<'src, I>() -> impl Parser<'src, I, Vec<Item>, Extras<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = SimpleSpan>,
{
    // ident ::= Ident
    let ident = select! { Token::Ident(ident) => ident }.map_with(|ident, e| {
        let state: &mut SimpleState<State> = e.state();
        Ident {
            key: state.rodeo.get_or_intern(ident),
            span: e.span(),
        }
    });

    // literal_kind ::= LitBool | LitInteger | LitNatural | LitUnit | LitString
    let literal_kind = choice((
        select! {
            Token::LitBool(value) => LiteralKind::Bool(value),
            Token::LitInteger(value) => LiteralKind::Integer(value),
            Token::LitNatural(value) => LiteralKind::Natural(value),
            // TODO: Floats
            // Token::LitFloat(value) => LiteralKind::Float(value),
            Token::LitUnit => LiteralKind::Unit,
        },
        select! {
            Token::LitString(value) => value
        }
        .map_with(|value, e| {
            let state: &mut SimpleState<State> = e.state();
            LiteralKind::String(state.rodeo.get_or_intern(value))
        }),
    ));

    // ty ::= primitive | named | tuple | list | record | (ty)
    let ty = recursive(|ty| {
        // primitive ::= TypeBool | TypeFloat | TypeInteger | TypeNatural | TypeString | LitUnit
        let primitive = select! {
            Token::TypeBool => PrimitiveTypeKind::Bool,
            Token::TypeFloat => PrimitiveTypeKind::Float,
            Token::TypeInteger => PrimitiveTypeKind::Integer,
            Token::TypeNatural => PrimitiveTypeKind::Natural,
            Token::TypeString => PrimitiveTypeKind::String,
            Token::LitUnit => PrimitiveTypeKind::Unit
        }
        .map(TypeKind::Primitive);

        // named ::= ident
        let named = ident.clone().map(TypeKind::Named);

        // tuple ::= (ty (, ty)*)
        let tuple = ty
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(TypeKind::Tuple);

        // list ::= [ty]
        let list = ty
            .clone()
            .delimited_by(just(Token::SymLBracket), just(Token::SymRBracket))
            .map(|ty| TypeKind::List(Box::new(ty)));

        // record ::= { ident : ty (, ident : ty)* }
        let record = ident
            .clone()
            .then_ignore(just(Token::SymColon))
            .then(ty.clone())
            .separated_by(just(Token::SymComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
            .map(|entries| TypeKind::Record(indexmap::IndexMap::from_iter(entries)));

        let atom = choice((primitive, named, tuple, list, record))
            .map_with(|kind, e| Type {
                kind,
                span: e.span(),
            })
            .or(ty
                .clone()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)));

        // ty ::= ty & ty | ty | ty
        atom.pratt((
            infix(right(1), just(Token::OpBitwiseAnd), |lhs, _, rhs, e| Type {
                kind: TypeKind::Intersection {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span: e.span(),
            }),
            infix(right(2), just(Token::OpBitwiseOr), |lhs, _, rhs, e| Type {
                kind: TypeKind::Union {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span: e.span(),
            }),
        ))
    });

    // pattern ::= literal | variable | wildcard | tuple | record | empty_list | (pattern)
    let pattern = recursive(|pattern| {
        // literal ::= literal_kind
        let atom_literal = literal_kind.clone().map(PatternKind::Literal);

        // variable ::= ident
        let atom_variable = ident.clone().map(PatternKind::Var);

        // wildcard ::= _
        let atom_wildcard = just(Token::SymWildcard).to(PatternKind::Wildcard);

        // tuple ::= (pattern (, pattern)*)
        let atom_tuple = pattern
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(2)
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(PatternKind::Tuple);

        // record ::= { ident : pattern (, ident : pattern)* }
        let atom_record = ident
            .clone()
            .then_ignore(just(Token::SymColon))
            .then(pattern.clone())
            .separated_by(just(Token::SymComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
            .map(|entries| PatternKind::Record(indexmap::IndexMap::from_iter(entries)));

        // empty_list ::= []
        let atom_empty_list = just(Token::SymArray).to(PatternKind::EmptyList);

        let atom = choice((
            atom_literal,
            atom_variable,
            atom_wildcard,
            atom_tuple,
            atom_record,
            atom_empty_list,
        ))
        .map_with(|kind, e| Pattern {
            kind,
            span: e.span(),
        })
        .or(pattern.delimited_by(just(Token::SymLParen), just(Token::SymRParen)));

        // pattern ::= pattern :: pattern | pattern | pattern
        atom.pratt((
            infix(
                Associativity::Right(1),
                just(Token::OpCons),
                |lhs, _, rhs, e| Pattern {
                    kind: PatternKind::Cons {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span: e.span(),
                },
            ),
            infix(
                Associativity::Left(2),
                just(Token::OpBitwiseOr),
                |lhs, _, rhs, e| Pattern {
                    kind: PatternKind::Or {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span: e.span(),
                },
            ),
        ))
    });

    // destructor ::= variable | tuple | record | (destructor)
    let destructor = recursive(|destructor| {
        // variable ::= ident
        let atom_variable = ident.clone().map(DestructorKind::Var);

        // tuple ::= (destructor (, destructor)*)
        let atom_tuple = destructor
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(2)
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(DestructorKind::Tuple);

        // record ::= { ident : destructor (, ident : destructor)* }
        let atom_record = ident
            .clone()
            .then_ignore(just(Token::SymColon))
            .then(destructor.clone())
            .separated_by(just(Token::SymComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
            .map(|entries| DestructorKind::Record(indexmap::IndexMap::from_iter(entries)));

        choice((atom_variable, atom_tuple, atom_record))
            .map_with(|kind, e| Destructor {
                kind,
                span: e.span(),
            })
            .or(destructor.delimited_by(just(Token::SymLParen), just(Token::SymRParen)))
    });

    // expr ::= literal | variable | tuple | list | if_expr | match_expr | (expr)
    let expr = recursive(|expr| {
        // literal ::= literal_kind
        let atom_literal = literal_kind.clone().map(ExprKind::Literal);

        // variable ::= ident
        let atom_variable = ident.clone().map(ExprKind::Var);

        // tuple ::= (expr (, expr)*)
        let atom_tuple = expr
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(2)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(ExprKind::Tuple);

        // list ::= [expr (, expr)*] | []
        let atom_list = expr
            .clone()
            .separated_by(just(Token::SymComma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBracket), just(Token::SymRBracket))
            .or(just(Token::SymArray).to(vec![]))
            .map(ExprKind::List);

        // if_expr ::= if expr { expr } else { expr }
        let atom_if = just(Token::KeywordIf)
            .ignore_then(expr.clone())
            .then(
                expr.clone()
                    .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace)),
            )
            .then(
                just(Token::KeywordElse)
                    .ignore_then(
                        expr.clone()
                            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace)),
                    )
                    .or_not(),
            )
            .map(|((condition, body), otherwise)| ExprKind::Conditional {
                condition: Box::new(condition),
                body: Box::new(body),
                otherwise: otherwise.map(|otherwise| Box::new(otherwise)),
            });

        // match_expr ::= match expr { pattern -> expr (, pattern -> expr)* }
        let atom_match = just(Token::KeywordMatch)
            .ignore_then(expr.clone())
            .then(
                pattern
                    .clone()
                    .then_ignore(just(Token::SymArrow))
                    .then(expr.clone())
                    .map_with(|(pattern, expr), e| MatchArm {
                        pattern,
                        expr,
                        span: e.span(),
                    })
                    .separated_by(just(Token::SymComma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace)),
            )
            .map(|(value, arms)| ExprKind::Match {
                value: Box::new(value),
                arms,
            });

        let atom = choice((
            atom_literal,
            atom_variable,
            atom_tuple,
            atom_list,
            atom_if,
            atom_match,
        ))
        .map_with(|kind, e| Expr {
            kind,
            span: e.span(),
        })
        .or(expr
            .clone()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen)));

        // expr ::= lambda | unary_expr | call | binary_expr
        atom.clone().pratt((
            // lambda ::= (destructor (, destructor)* -> expr)
            prefix(
                12,
                destructor
                    .clone()
                    .then(ty.clone().or_not())
                    .map(|(parameter, ty)| LambdaParam { parameter, ty })
                    .separated_by(just(Token::SymComma))
                    .collect::<Vec<_>>()
                    .then_ignore(just(Token::SymArrow)),
                |params, body, e| Expr {
                    kind: ExprKind::Lambda {
                        params,
                        ret_ty: None,
                        body: Box::new(body),
                    },
                    span: e.span(),
                },
            ),
            // unary_expr ::= op expr
            prefix(
                11,
                select! {
                     Token::OpAdd => UnaryOpKind::UnaryPlus,
                     Token::OpBitwiseNot => UnaryOpKind::BitwiseNot,
                     Token::OpNegate => UnaryOpKind::LogicalNot,
                     Token::OpSubtract => UnaryOpKind::Negate,
                }
                .map_with(|kind, e| UnaryOp {
                    kind,
                    span: e.span(),
                }),
                |op, expr, e| Expr {
                    kind: ExprKind::UnaryExpr {
                        expr: Box::new(expr),
                        op,
                    },
                    span: e.span(),
                },
            ),
            // call ::= expr (expr (, expr)*)
            postfix(
                10,
                just(Token::LitUnit).to(vec![]).or(atom
                    .clone()
                    .separated_by(just(Token::SymComma))
                    .at_least(1)
                    .collect::<Vec<_>>()),
                |function, arguments, e| Expr {
                    kind: ExprKind::Call {
                        function: Box::new(function),
                        arguments,
                    },
                    span: e.span(),
                },
            ),
            // binary_expr ::= expr op expr
            infix(
                right(9),
                binary_op(select! {
                    Token::OpExponentiate => BinaryOpKind::Exponentiate
                }),
                binary_expr,
            ),
            infix(
                left(8),
                binary_op(select! {
                    Token::OpMultiply => BinaryOpKind::Multiply,
                    Token::OpDivide => BinaryOpKind::Divide,
                    Token::OpModulo => BinaryOpKind::Modulo,
                }),
                binary_expr,
            ),
            infix(
                left(7),
                binary_op(select! {
                    Token::OpAdd => BinaryOpKind::Add,
                    Token::OpSubtract => BinaryOpKind::Subtract,
                }),
                binary_expr,
            ),
            infix(
                left(6),
                binary_op(select! {
                    Token::OpLessThanOrEqual => BinaryOpKind::LessThanOrEqual,
                    Token::OpGreaterThanOrEqual => BinaryOpKind::GreaterThanOrEqual,
                    Token::OpLessThan => BinaryOpKind::LessThan,
                    Token::OpGreaterThan => BinaryOpKind::GreaterThan,
                }),
                binary_expr,
            ),
            infix(
                left(5),
                binary_op(select! {
                    Token::OpEqual => BinaryOpKind::Equal,
                    Token::OpNotEqual => BinaryOpKind::NotEqual,
                }),
                binary_expr,
            ),
            infix(
                left(4),
                binary_op(select! {
                    Token::OpLogicalAnd => BinaryOpKind::LogicalAnd,
                }),
                binary_expr,
            ),
            infix(
                left(3),
                binary_op(select! {
                    Token::OpLogicalOr => BinaryOpKind::LogicalOr,
                }),
                binary_expr,
            ),
            infix(
                right(2),
                binary_op(select! {
                    Token::OpCons => BinaryOpKind::Cons,
                }),
                binary_expr,
            ),
        ))
    });

    // item_type_alias ::= type ident = ty
    let item_type_alias = just(Token::KeywordType)
        .ignore_then(ident.clone())
        .then_ignore(just(Token::OpAssign))
        .then(ty.clone())
        .map(|(name, ty)| ItemKind::TypeAlias { name, ty });

    // item_import_tree ::= import import_tree
    let item_import_tree = just(Token::KeywordImport)
        .ignore_then(recursive(|import_tree| {
            // item ::= ident (as ident)?
            let item = ident
                .clone()
                .then(just(Token::KeywordAs).ignore_then(ident.clone()).or_not())
                .map(|(name, alias)| ImportTreeKind::Item { name, alias });

            // segment ::= ident :: import_tree
            let segment = ident
                .clone()
                .then_ignore(just(Token::OpCons))
                .then(import_tree.clone())
                .map(|(name, child)| ImportTreeKind::Segment {
                    name,
                    child: Box::new(child),
                });

            // glob ::= *
            let glob = just(Token::OpMultiply).to(ImportTreeKind::Glob);

            // list ::= { import_tree (, import_tree)* }
            let list = import_tree
                .clone()
                .separated_by(just(Token::SymComma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
                .map(ImportTreeKind::List);

            choice((item, segment, glob, list)).map_with(|kind, e| ImportTree {
                kind,
                span: e.span(),
            })
        }))
        .map(ItemKind::Import);

    // item_definition ::= let destructor = expr
    let item_definition = just(Token::KeywordLet)
        .ignore_then(destructor.clone())
        .then_ignore(just(Token::OpAssign))
        .then(expr.clone())
        .map(|(name, expr)| ItemKind::Definition { name, expr });

    // item ::= item_type_alias | item_import_tree | item_definition
    let item =
        choice((item_type_alias, item_import_tree, item_definition)).map_with(|kind, e| Item {
            visibility: Visibility::Inherited,
            kind,
            span: e.span(),
        });

    // parser ::= item (; item)*
    item.separated_by(just(Token::SymSemicolon))
        .collect::<Vec<_>>()
}

/// Parses the given source code into a `Module` representation.
///
/// # Arguments
///
/// * `src` - A string slice containing the source code to be parsed.
///
/// # Returns
///
/// * `Ok(Module)` - If the parsing is successful, returns a `Module` containing the parsed items and a cache.
/// * `Err(Vec<Rich<Token>>)` - If the parsing fails, returns a vector of rich error messages with token information.
///
/// # Example
///
/// ```
/// let source = "let x = 42;";
/// let result = parse_str(source);
/// match result {
///     Ok(module) => println!("Parsed successfully: {:?}", module),
///     Err(errors) => println!("Parsing failed with errors: {:?}", errors),
/// }
/// ```
pub fn parse_str<'src>(src: &'src str) -> Result<Module, Vec<Rich<'src, Token<'src>>>> {
    let token_iter = Token::lexer(src).spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(e) => (Token::Error(e), span.into()),
    });
    let token_stream =
        Stream::from_iter(token_iter).map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

    let mut state = State::default().into();
    parser()
        .parse_with_state(token_stream, &mut state)
        .into_result()
        .map(move |items| Module {
            items,
            cache: state.0.rodeo,
        })
}

/// The `kali!` macro is a utility for parsing Rust-like syntax into a `Module` representation.
///
/// # Arguments
///
/// * `$($tts:tt)*` - A token tree that represents the input source code to be parsed.
///
/// # Returns
///
/// * The macro returns a `Module` object containing the parsed items and a cache. If the parsing fails,
///   the macro will panic with an error.
///
/// # Example
///
/// ```
/// use your_crate_name::kali;
///
/// let module = kali! {
///     let x = 42;
/// };
/// println!("{:?}", module);
/// ```
///
/// In this example, the `kali!` macro parses the input source code `let x = 42;` and returns a `Module` object.
///
/// # Panics
///
/// This macro will panic if the parsing fails, which can happen if the input source code contains syntax errors.
#[macro_export]
macro_rules! kali {
    ($($tts:tt)*) => {{
        let input = stringify!($($tts)*);
        $crate::parse_str(input).unwrap()
    }};
}

fn test() {
    kali! {
        let x = 1
    };
}
