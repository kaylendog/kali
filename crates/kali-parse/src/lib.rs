use std::collections::BTreeMap;

use chumsky::{
    extra::SimpleState,
    input::{Input, MapExtra, Stream, ValueInput},
    pratt::{Associativity, infix, left, postfix, prefix, right},
    prelude::*,
};
use kali_ast::{
    BinaryOp, BinaryOpKind, Comment, CommentKind, Definition, Destructor, DestructorKind, Expr,
    ExprKind, Ident, ImportTree, ImportTreeKind, Item, ItemKind, LambdaParam, LiteralKind,
    MatchArm, Module, NodeId, Pattern, PatternKind, PrimitiveTypeKind, Type, TypeAlias, TypeKind,
    UnaryOp, UnaryOpKind, Visibility,
};
use logos::Logos;

pub use crate::lexer::Token;

/// Re-export.
pub use chumsky::{self};

mod lexer;

/// Represents the state used during parsing, including a string interner for efficient string handling.
#[derive(Default)]
struct State {
    /// A counter to generate unique identifiers for nodes in the abstract syntax tree (AST).
    next_id: NodeId,
    /// A `Rodeo` instance from the `lasso` crate, used for interning strings to reduce memory usage and improve performance.
    rodeo: lasso::Rodeo,
}

impl State {
    /// Generates a new unique identifier for a node in the abstract syntax tree (AST).
    fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
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
        id: e.state().next_id(),
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
    let ident = select! { Token::Ident(ident) => ident }
        .map_with(|ident, e| {
            let state: &mut SimpleState<State> = e.state();
            Ident {
                key: state.rodeo.get_or_intern(ident),
                span: e.span(),
            }
        })
        .labelled("identifier");

    // literal_kind ::= LitBool | LitInteger | LitNatural | LitUnit | LitString
    let literal_kind = choice((
        select! {
            Token::LitBool(value) => LiteralKind::Bool(value),
            Token::LitInteger(value) => LiteralKind::Integer(value),
            Token::LitNatural(value) => LiteralKind::Natural(value),
            // TODO: Floats
            // Token::LitFloat(value) => LiteralKind::Float(value),
            Token::LitUnit => LiteralKind::Unit,
        }
        .labelled("literal"),
        select! {
            Token::LitString(value) => value
        }
        .map_with(|value, e| {
            let state: &mut SimpleState<State> = e.state();
            LiteralKind::String(state.rodeo.get_or_intern(value))
        })
        .labelled("string literal"),
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
        .map(TypeKind::Primitive)
        .labelled("primitive type");

        // named ::= ident
        let named = ident.clone().map(TypeKind::Named).labelled("named type");

        // tuple ::= (ty (, ty)*)
        let tuple = ty
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(TypeKind::Tuple)
            .labelled("tuple type");

        // list ::= [ty]
        let list = ty
            .clone()
            .delimited_by(just(Token::SymLBracket), just(Token::SymRBracket))
            .map(|ty| TypeKind::List(Box::new(ty)))
            .labelled("list type");

        // record ::= { ident : ty (, ident : ty)* }
        let record = ident
            .clone()
            .then_ignore(just(Token::SymColon))
            .then(ty.clone())
            .separated_by(just(Token::SymComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
            .map(|entries| TypeKind::Record(indexmap::IndexMap::from_iter(entries)))
            .labelled("record type");

        let atom = choice((primitive, named, tuple, list, record))
            .map_with(|kind, e| {
                let state: &mut SimpleState<State> = e.state();
                Type {
                    id: state.next_id(),
                    kind,
                    span: e.span(),
                }
            })
            .or(ty
                .clone()
                .delimited_by(just(Token::SymLParen), just(Token::SymRParen)));

        // ty ::= ty & ty | ty | ty
        atom.pratt((
            infix(right(1), just(Token::OpBitwiseAnd), |lhs, _, rhs, e| {
                let state: &mut SimpleState<State> = e.state();
                Type {
                    id: state.next_id(),
                    kind: TypeKind::Intersection {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span: e.span(),
                }
            }),
            infix(right(2), just(Token::OpBitwiseOr), |lhs, _, rhs, e| {
                let state: &mut SimpleState<State> = e.state();
                Type {
                    id: state.next_id(),
                    kind: TypeKind::Union {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span: e.span(),
                }
            }),
        ))
    })
    .labelled("type");

    // pattern ::= literal | variable | wildcard | tuple | record | empty_list | (pattern)
    let pattern = recursive(|pattern| {
        // literal ::= literal_kind
        let atom_literal = literal_kind
            .clone()
            .map(PatternKind::Literal)
            .labelled("literal pattern");

        // variable ::= ident
        let atom_variable = ident
            .clone()
            .map(PatternKind::Var)
            .labelled("variable pattern");

        // wildcard ::= _
        let atom_wildcard = just(Token::SymWildcard)
            .to(PatternKind::Wildcard)
            .labelled("wildcard pattern");

        // tuple ::= (pattern (, pattern)*)
        let atom_tuple = pattern
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(2)
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(PatternKind::Tuple)
            .labelled("tuple pattern");

        // record ::= { ident : pattern (, ident : pattern)* }
        let atom_record = ident
            .clone()
            .then_ignore(just(Token::SymColon))
            .then(pattern.clone())
            .separated_by(just(Token::SymComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
            .map(|entries| PatternKind::Record(indexmap::IndexMap::from_iter(entries)))
            .labelled("record pattern");

        // empty_list ::= []
        let atom_empty_list = just(Token::SymArray)
            .to(PatternKind::EmptyList)
            .labelled("empty list pattern");

        let atom = choice((
            atom_literal,
            atom_variable,
            atom_wildcard,
            atom_tuple,
            atom_record,
            atom_empty_list,
        ))
        .map_with(|kind, e| {
            let state: &mut SimpleState<State> = e.state();
            Pattern {
                id: state.next_id(),
                kind,
                span: e.span(),
            }
        })
        .or(pattern.delimited_by(just(Token::SymLParen), just(Token::SymRParen)))
        .labelled("pattern");

        // pattern ::= pattern :: pattern | pattern | pattern
        atom.pratt((
            infix(
                Associativity::Right(1),
                just(Token::OpCons),
                |lhs, _, rhs, e| {
                    let state: &mut SimpleState<State> = e.state();
                    Pattern {
                        id: state.next_id(),
                        kind: PatternKind::Cons {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span: e.span(),
                    }
                },
            ),
            infix(
                Associativity::Left(2),
                just(Token::OpBitwiseOr),
                |lhs, _, rhs, e| {
                    let state: &mut SimpleState<State> = e.state();
                    Pattern {
                        id: state.next_id(),
                        kind: PatternKind::Or {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span: e.span(),
                    }
                },
            ),
        ))
    })
    .labelled("pattern");

    // destructor ::= variable | tuple | record | (destructor)
    let destructor = recursive(|destructor| {
        // variable ::= ident
        let atom_variable = ident
            .clone()
            .map(DestructorKind::Var)
            .labelled("variable destructor");

        // tuple ::= (destructor (, destructor)*)
        let atom_tuple = destructor
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(2)
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(DestructorKind::Tuple)
            .labelled("tuple destructor");

        // record_item := ident (: destructor)?
        let atom_record_item = ident
            .clone()
            .then(
                just(Token::SymColon)
                    .ignore_then(destructor.clone())
                    .or_not(),
            )
            .map(|(name, value)| (name, value))
            .labelled("record item");

        // record ::= { record_item (, record_item)* }
        let atom_record = atom_record_item
            .clone()
            .separated_by(just(Token::SymComma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
            .map(|entries| DestructorKind::Record(indexmap::IndexMap::from_iter(entries)))
            .labelled("record destructor");

        choice((atom_variable, atom_tuple, atom_record))
            .map_with(|kind, e| {
                let state: &mut SimpleState<State> = e.state();
                Destructor {
                    id: state.next_id(),
                    kind,
                    span: e.span(),
                }
            })
            .or(destructor.delimited_by(just(Token::SymLParen), just(Token::SymRParen)))
    })
    .labelled("destructor");

    // expr ::= literal | variable | tuple | list | if_expr | match_expr | (expr)
    let expr = recursive(|expr| {
        // literal ::= literal_kind
        let atom_literal = literal_kind
            .clone()
            .map(ExprKind::Literal)
            .labelled("literal expression");

        // variable ::= ident
        let atom_variable = ident
            .clone()
            .map(ExprKind::Var)
            .labelled("variable expression");

        // tuple ::= (expr (, expr)*)
        let atom_tuple = expr
            .clone()
            .separated_by(just(Token::SymComma))
            .at_least(2)
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
            .map(ExprKind::Tuple)
            .labelled("tuple expression");

        // list ::= [expr (, expr)*] | []
        let atom_list = expr
            .clone()
            .separated_by(just(Token::SymComma))
            .collect::<Vec<_>>()
            .delimited_by(just(Token::SymLBracket), just(Token::SymRBracket))
            .or(just(Token::SymArray).to(vec![]))
            .map(ExprKind::List)
            .labelled("list expression");

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
            })
            .labelled("if expression");

        // match_expr ::= match expr { (| pattern -> expr)+ }
        let atom_match = just(Token::KeywordMatch)
            .ignore_then(expr.clone())
            .then(
                just(Token::OpBitwiseOr)
                    .ignore_then(pattern.clone())
                    .then_ignore(just(Token::SymArrow))
                    .then(expr.clone())
                    .map_with(|(pattern, expr), e| {
                        let state: &mut SimpleState<State> = e.state();
                        MatchArm {
                            id: state.next_id(),
                            pattern,
                            expr,
                            span: e.span(),
                        }
                    })
                    .repeated()
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace)),
            )
            .map(|(value, arms)| ExprKind::Match {
                value: Box::new(value),
                arms,
            })
            .labelled("match expression");

        let atom = choice((
            atom_literal,
            atom_variable,
            atom_tuple,
            atom_list,
            atom_if,
            atom_match,
        ))
        .map_with(|kind, e| {
            let state: &mut SimpleState<State> = e.state();
            Expr {
                id: state.next_id(),
                kind,
                span: e.span(),
            }
        })
        .or(expr
            .clone()
            .delimited_by(just(Token::SymLParen), just(Token::SymRParen)));

        // expr ::= lambda | unary_expr | call | binary_expr
        atom.clone().pratt((
            // lambda ::= param+ -> expr
            // param ::= destructor | (destructor : ty)
            // OCaml style: params are juxtaposed, annotations need parens. Lowest binding power
            // so the body extends over every operator.
            prefix(
                1,
                choice((
                    destructor
                        .clone()
                        .then_ignore(just(Token::SymColon))
                        .then(ty.clone())
                        .delimited_by(just(Token::SymLParen), just(Token::SymRParen))
                        .map(|(parameter, ty)| LambdaParam {
                            parameter,
                            ty: Some(ty),
                        }),
                    destructor.clone().map(|parameter| LambdaParam {
                        parameter,
                        ty: None,
                    }),
                ))
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>()
                .then_ignore(just(Token::SymArrow)),
                |params, body, e| {
                    let state: &mut SimpleState<State> = e.state();
                    Expr {
                        id: state.next_id(),
                        kind: ExprKind::Lambda {
                            params,
                            ret_ty: None,
                            body: Box::new(body),
                        },
                        span: e.span(),
                    }
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
                |op, expr, e| {
                    let state: &mut SimpleState<State> = e.state();
                    Expr {
                        id: state.next_id(),
                        kind: ExprKind::UnaryExpr {
                            expr: Box::new(expr),
                            op,
                        },
                        span: e.span(),
                    }
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
                |function, arguments, e| {
                    let state: &mut SimpleState<State> = e.state();
                    Expr {
                        id: state.next_id(),
                        kind: ExprKind::Call {
                            function: Box::new(function),
                            arguments,
                        },
                        span: e.span(),
                    }
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
    })
    .labelled("expression");

    // item_type_alias ::= type ident = ty
    let item_type_alias = just(Token::KeywordType)
        .ignore_then(ident.clone())
        .then_ignore(just(Token::OpAssign))
        .then(ty.clone())
        .map(|(name, ty)| ItemKind::TypeAlias(TypeAlias { name, ty }))
        .labelled("type alias");

    // item_import_tree ::= import import_tree
    let item_import_tree = just(Token::KeywordImport)
        .ignore_then(recursive(|import_tree| {
            // item ::= ident (as ident)?
            let item = ident
                .clone()
                .then(just(Token::KeywordAs).ignore_then(ident.clone()).or_not())
                .map(|(name, alias)| ImportTreeKind::Item { name, alias })
                .labelled("import item");

            // segment ::= ident :: import_tree
            let segment = ident
                .clone()
                .then_ignore(just(Token::OpCons))
                .then(import_tree.clone())
                .map(|(name, child)| ImportTreeKind::Segment {
                    name,
                    child: Box::new(child),
                })
                .labelled("import segment");

            // glob ::= *
            let glob = just(Token::OpMultiply)
                .to(ImportTreeKind::Glob)
                .labelled("import glob");

            // list ::= { import_tree (, import_tree)* }
            let list = import_tree
                .clone()
                .separated_by(just(Token::SymComma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::SymLBrace), just(Token::SymRBrace))
                .map(ImportTreeKind::List)
                .labelled("import list");

            // `segment` before `item`: both start with an identifier, and `item` would
            // succeed on `std` alone, leaving `:: {..}` unparsed.
            choice((segment, item, glob, list)).map_with(|kind, e| ImportTree {
                kind,
                span: e.span(),
            })
        }))
        .map(ItemKind::Import)
        .labelled("import tree");

    // item_definition ::= let destructor = expr
    let item_definition = just(Token::KeywordLet)
        .ignore_then(destructor.clone())
        .then_ignore(just(Token::OpAssign))
        .then(expr.clone())
        .map(|(name, expr)| ItemKind::Definition(Definition { name, expr }))
        .labelled("definition");

    // item ::= item_type_alias | item_import_tree | item_definition
    let item = choice((item_type_alias, item_import_tree, item_definition)).map_with(|kind, e| {
        let state: &mut SimpleState<State> = e.state();
        Item {
            id: state.next_id(),
            visibility: Visibility::Inherited,
            kind,
            span: e.span(),
        }
    });

    // module ::= item (; item)* ;?
    item.separated_by(just(Token::SymSemicolon))
        .allow_trailing()
        .collect::<Vec<_>>()
}

/// Whether a newline after `token` ends the item, like Go's automatic semicolon insertion.
fn ends_item(token: &Token) -> bool {
    matches!(
        token,
        Token::Ident(_)
            | Token::LitNatural(_)
            | Token::LitInteger(_)
            | Token::LitBool(_)
            | Token::LitUnit
            | Token::LitString(_)
            | Token::TypeInteger
            | Token::TypeNatural
            | Token::TypeFloat
            | Token::TypeBool
            | Token::TypeString
            | Token::SymRParen
            | Token::SymRBracket
            | Token::SymRBrace
            | Token::SymArray
            // `import std :: *` ends in a glob, so `*` at end of line terminates too.
            | Token::OpMultiply
    )
}

/// Lexes the given source code into a list of tokens.
///
/// # Arguments
///
/// * `src` - A string slice containing the source code to be parsed.
///
/// # Returns
///
/// * `Vec<Spanned<Token>>` - A vector of tokens with their corresponding spans.
pub fn lex_str<'src>(src: &'src str) -> Vec<Spanned<Token<'src>>> {
    Token::lexer(src)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => Spanned {
                inner: tok,
                span: span.into(),
            },
            Err(e) => Spanned {
                inner: Token::Error(e),
                span: span.into(),
            },
        })
        .collect()
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
/// let (module, errors) = kali_parse::parse_str(source);
/// println!("Parsed module: {:?}", module);
/// println!("Parsing errors: {:?}", errors);
/// ```
pub fn parse_str<'src>(src: &'src str) -> (Option<Module>, Vec<Rich<'src, Token<'src>>>) {
    let mut state: SimpleState<State> = State::default().into();

    let tokens: Vec<(Token, SimpleSpan)> = Token::lexer(src)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => (tok, span.into()),
            Err(e) => (Token::Error(e), span.into()),
        })
        .collect();

    // Split comments out into their own table, and turn line ends into `;` where the line
    // ends an item, unless the next significant token closes a bracket. Go does the same.
    let mut comments = BTreeMap::new();
    let mut significant = Vec::with_capacity(tokens.len());
    let mut previous_ends_item = false;
    for (index, (tok, span)) in tokens.iter().enumerate() {
        match tok {
            Token::Comment(contents) | Token::DocComment(contents) => {
                let kind = match tok {
                    Token::Comment(_) => CommentKind::Single,
                    _ => CommentKind::Doc,
                };
                let contents = state.0.rodeo.get_or_intern(contents);
                let comment = Comment {
                    contents,
                    kind,
                    span: *span,
                };
                comments.insert(*span, comment);
            }
            Token::Newline => {}
            _ => {
                previous_ends_item = ends_item(tok);
                significant.push((tok.clone(), *span));
                continue;
            }
        }
        let next_closes = tokens[index + 1..]
            .iter()
            .map(|(tok, _)| tok)
            .find(|tok| {
                !matches!(
                    tok,
                    Token::Newline | Token::Comment(_) | Token::DocComment(_)
                )
            })
            .is_some_and(|tok| {
                // `|` starts a match arm, so a line end before it continues the match.
                matches!(
                    tok,
                    Token::SymRParen | Token::SymRBracket | Token::SymRBrace | Token::OpBitwiseOr
                )
            });
        if std::mem::take(&mut previous_ends_item) && !next_closes {
            significant.push((Token::SymSemicolon, *span));
        }
    }

    let token_stream =
        Stream::from_iter(significant).map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

    let (items, errors) = parser()
        .parse_with_state(token_stream, &mut state)
        .into_output_errors();

    (
        items.map(move |items| Module {
            cache: state.0.rodeo,
            items,
            comments,
        }),
        errors,
    )
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
/// use kali_parse::kali;
///
/// let (module, errors) = kali! {
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
        $crate::parse_str(input)
    }};
}

#[cfg(test)]
mod tests {
    use kali_ast::{Definition, ExprKind, ItemKind};

    #[test]
    fn lambda_body_spans_binary_expr() {
        let (module, errors) = kali! {
            let add = a b -> a + b
        };
        assert!(errors.is_empty(), "{errors:?}");
        let module = module.unwrap();
        let ItemKind::Definition(Definition { expr, .. }) = &module.items[0].kind else {
            panic!("expected a definition");
        };
        let ExprKind::Lambda { params, body, .. } = &expr.kind else {
            panic!("expected a lambda, found {expr:?}");
        };
        assert_eq!(params.len(), 2);
        assert!(params.iter().all(|param| param.ty.is_none()));
        assert!(matches!(body.kind, ExprKind::BinaryExpr { .. }));
    }
}
