use std::{ops::Range, path::PathBuf};

/// Contains information about a parsed module.
pub struct Module {
    /// Information about the source file.
    pub src_info: SrcInfo,
    /// The collection of interned identifiers.
    pub identifiers: lasso::Rodeo,
    /// The root items in the module.
    pub root: Vec<Item>,
}

/// Contains information about the source file.
pub struct SrcInfo {
    /// The file path associated with the source.
    pub path: PathBuf,
}

/// Represents a span in the source code, including the start and end positions and the file identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// The start position of the span (inclusive).
    pub start: usize,
    /// The end position of the span (exclusive).
    pub end: usize,
    /// The identifier of the file this span belongs to.
    pub file_id: u32,
}

impl chumsky::span::Span for Span {
    type Context = u32;
    type Offset = usize;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Span {
            start: range.start,
            end: range.end,
            file_id: context,
        }
    }

    fn context(&self) -> Self::Context {
        self.file_id
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}
impl From<chumsky::span::SimpleSpan> for Span {
    fn from(value: chumsky::span::SimpleSpan) -> Self {
        Span {
            file_id: 0,
            start: value.start,
            end: value.end,
        }
    }
}

/// Represents an item in the source code, such as a function, constant, type, import, or export.
///
/// # Type Parameters
/// - `T`: The kind of item, defaults to [`ItemKind`].
#[derive(Debug, Clone)]
pub struct Item<T = ItemKind> {
    /// Unique identifier for the item.
    pub id: u32,
    /// The span of the item in the source code.
    pub span: Span,
    /// The kind of item (function, constant, type, etc.).
    pub kind: T,
    /// The visibility of the item (private, exported, inherited).
    pub visibility: Visibility,
}

/// Represents the visibility of an item in the source code.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum Visibility {
    /// The item is private and not accessible outside its scope.
    Private,
    /// The item is exported and accessible outside its scope.
    Exported,
    /// The item inherits visibility from its context.
    #[default]
    Inherited,
}

/// Enumeration of item kinds.
#[derive(Debug, Clone)]
pub enum ItemKind {
    /// A function declaration.
    Fn(FnItem),
    /// A constant declaration.
    Const(ConstItem),
    /// A type declaration.
    TypeAlias(TypeAliasItem),
    /// An import.
    Import(ImportItem),
    /// An export.
    Export(ExportItem),
}

#[derive(Debug, Clone)]
/// Represents a function declaration in the source code.
pub struct FnItem {
    /// Unique identifier for the item.
    pub id: u32,
    /// The name of the function.
    pub name: Ident,
    /// The parameters of the function.
    pub parameters: Vec<FnParam>,
    /// The span of the function declaration in the source code.
    pub span: Span,
}

#[derive(Debug, Clone)]
/// Represents a function parameter, including its name and type.
pub struct FnParam {
    /// Unique identifier for the item.
    pub id: u32,
    /// The name of the parameter.
    pub name: Ident,
    /// The type of the parameter.
    pub ty: Option<Type>,
    /// The span of the parameter in the source code.
    pub span: Span,
}

/// A constant item.
#[derive(Debug, Clone)]
pub struct ConstItem {
    /// Unique identifier for the item.
    pub id: u32,
    /// The name of the constant.
    pub name: Ident,
    /// The type of the constant.
    pub ty: Option<Type>,
    /// The content of the constant.
    pub content: Expr,
    /// The span of the constant declaration in the source code.
    pub span: Span,
}

/// Represents a type alias declaration in the source code.
#[derive(Debug, Clone)]
pub struct TypeAliasItem {
    /// Unique identifier for the item.
    pub id: u32,
    /// The name of the type alias.
    pub name: Ident,
    /// The type that is being aliased.
    pub ty: Type,
    /// The span of the constant declaration in the source code.
    pub span: Span,
}

/// Represents an import statement in the source code.
#[derive(Debug, Clone)]
pub struct ImportItem {
    /// Unique identifier for the item.
    pub id: u32,
    /// The span of the constant declaration in the source code.
    pub span: Span,
    /// The kind of import (simple, aliased, list, or wildcard).
    pub kind: ImportTree,
}

/// Represents the kind of import in an import statement.
#[derive(Debug, Clone)]
pub enum ImportTree {
    /// A simple import of a single identifier.
    Simple(Ident),
    /// An import with an alias (original identifier, alias).
    Aliased(Ident, Ident),
    /// An import of a list of import kinds.
    List {
        id: u32,
        span: Span,
        children: Vec<ImportTree>,
    },
}

/// Represents an export statement in the source code.
#[derive(Debug, Clone)]
pub struct ExportItem {
    /// Unique identifier for the item.
    pub id: u32,
    /// The span of the constant declaration in the source code.
    pub span: Span,
    /// The kind of export (simple, aliased, list, path list, or wildcard).
    pub kind: ExportKind,
}

/// Represents the kind of export in an export statement.
#[derive(Debug, Clone)]
pub enum ExportKind {
    /// A simple export of a single identifier.
    Simple(Ident),
    /// An export with an alias (original identifier, alias).
    Aliased(Ident, Ident),
    /// An export of a list of export kinds.
    List(Vec<ExportKind>),
    /// TODO
    PathList(Vec<ExportKind>, PathBuf),
    /// A wildcard export (e.g., `*`).
    Wildcard(PathBuf),
    /// An item.
    Item(Box<Item>),
}

/// Represents an identifier in the source code, including its textual value and span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    /// Unique identifier for the item.
    pub id: u32,
    /// The index of the identifier in the table.
    pub index: lasso::Spur,
    /// The span of the identifier in the source code.
    pub span: Span,
}

/// Represents a type in the source code, including its span and kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    /// Unique identifier for the item.
    pub id: u32,
    /// The span of the type in the source code.
    pub span: Span,
    /// The kind of type (primitive, function, tuple, etc.).
    pub kind: TypeKind,
}

/// Represents the kind of type in the source code.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TypeKind {
    /// A primitive type (e.g., int, float, bool, string, unit).
    Primitive(PrimitiveTypeKind),
    /// A function type with parameters and a return type.
    Fn(Vec<Type>, Box<Type>),
    /// A tuple type containing multiple types.
    Tuple(Vec<Type>),
    /// A list type containing elements of a single type.
    List(Box<Type>),
    /// A record type with named fields.
    Record(indexmap::IndexMap<Ident, Type>),
    /// An inferred type (type to be determined).
    #[default]
    Infer,
    /// An error type (used for type errors).
    Error,
}

/// Enumeration of primitive types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTypeKind {
    /// Integer type.
    Int,
    /// Floating-point type.
    Float,
    /// Boolean type.
    Bool,
    /// String type.
    String,
    /// Unit type (empty tuple).
    Unit,
}

/// Represents a match expression in the source code.
#[derive(Debug, Clone)]
pub struct Match {
    /// Unique identifier for the match expression.
    pub id: u32,
    /// The span of the match expression in the source code.
    pub span: Span,
    /// The expression being matched on.
    pub value: Box<Expr>,
    /// The list of match arms.
    pub arms: Vec<MatchArm>,
}

/// Represents a single arm of a match expression.
#[derive(Debug, Clone)]
pub struct MatchArm {
    /// Unique identifier for the match arm.
    pub id: u32,
    /// The span of the match arm in the source code.
    pub span: Span,
    /// The pattern to match.
    pub pattern: Pattern,
    /// The expression to execute if the pattern matches.
    pub expr: Expr,
}

/// Represents the kind of pattern in a match arm.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Unique identifier for the pattern.
    pub id: u32,
    /// The span of the pattern in the source code.
    pub span: Span,
    /// The kind of pattern (literal, identifier, tuple, etc.).
    pub kind: PatternKind,
}

/// The different kinds of patterns.
#[derive(Debug, Clone)]
pub enum PatternKind {
    /// A literal pattern (e.g., `42`, `"foo"`, `true`).
    Literal(LiteralKind),
    /// An identifier pattern (e.g., `x`).
    Ident(Ident),
    /// A tuple pattern (e.g., `(a, b)`).
    Tuple(Vec<Pattern>),
    /// A wildcard pattern (`_`).
    Wildcard,
    /// A record pattern (e.g., `{ x, y }`).
    Record(indexmap::IndexMap<Ident, Pattern>),
    /// A list pattern (e.g., `[a, b, ..]`).
    List(Vec<Pattern>),
    /// An or-pattern (e.g., `A | B`).
    Or(Vec<Pattern>),
}

/// Represents an expression item in the source code.
#[derive(Debug, Clone)]
pub struct Expr {
    /// Unique identifier for the item.
    pub id: u32,
    /// The span of the expression in the source code.
    pub span: Span,
    /// The kind of expression.
    pub kind: ExprKind,
}

/// Represents the kind of expression.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// A literal expression.
    Literal { kind: LiteralKind },
    /// A binary expression.
    BinaryExpr {
        /// The binary operator applied in the expression.
        operator: BinOp,
        /// The left-hand side expression.
        lhs: Box<Expr>,
        /// The right-hand side expression.
        rhs: Box<Expr>,
    },
    /// A unary expression.
    UnaryExpr { operator: UnaryOp, expr: Box<Expr> },
    /// A tuple expression.
    Tuple(Vec<Expr>),
    /// An array expression.
    Array(Vec<Expr>),
    /// A record expression.
    Record {
        /// The fields of the record, mapping identifiers to expressions.
        fields: indexmap::IndexMap<Ident, Expr>,
    },
    /// A conditional expression (if-else).
    Conditional {
        /// The condition to evaluate.
        condition: Box<Expr>,
        /// The expression to execute if the condition is true.
        body: Box<Expr>,
        /// The expression to execute if the condition is false (optional).
        otherwise: Option<Box<Expr>>,
    },
    /// A match expression.
    Match(Match),
}

#[derive(Debug, Clone)]
/// Represents the kind of literal in the source code.
pub enum LiteralKind {
    /// A natural number literal (e.g., `42`).
    Natural,
    /// An integer literal (e.g., `-7`).
    Integer,
    /// A floating-point literal (e.g., `3.14`).
    Float,
    /// A boolean literal (`true` or `false`).
    Bool,
    /// A string literal (e.g., `"hello"`).
    String,
    /// A unit literal (empty tuple, `()`).
    Unit,
}
/// Represents a binary operator in the source code, including its kind and span.
#[derive(Debug, Clone)]
pub struct BinOp {
    /// The kind of binary operator (e.g., addition, subtraction, etc.).
    pub kind: BinOpKind,
    /// The span of the binary operator in the source code.
    pub span: Span,
}

/// Represents the kind of binary operator in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum BinOpKind {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Subtract,
    #[strum(to_string = "*")]
    Multiply,
    #[strum(to_string = "/")]
    Divide,
    #[strum(to_string = "**")]
    Exponentiate,
    #[strum(to_string = "%")]
    Modulo,
    #[strum(to_string = "==")]
    Equal,
    #[strum(to_string = "!=")]
    NotEqual,
    #[strum(to_string = "<")]
    LessThan,
    #[strum(to_string = "<=")]
    LessThanOrEqual,
    #[strum(to_string = ">")]
    GreaterThan,
    #[strum(to_string = ">=")]
    GreaterThanOrEqual,
    #[strum(to_string = "&&")]
    LogicalAnd,
    #[strum(to_string = "||")]
    LogicalOr,
    #[strum(to_string = "&")]
    BitwiseAnd,
    #[strum(to_string = "|")]
    BitwiseOr,
    #[strum(to_string = "^")]
    BitwiseXor,
    #[strum(to_string = "<<")]
    BitwiseShiftLeft,
    #[strum(to_string = ">>")]
    BitwiseShiftRight,
    #[strum(to_string = "::")]
    Cons,
}

/// Represents a unary operator in the source code, including its kind and span.
#[derive(Debug, Clone)]
pub struct UnaryOp {
    /// The kind of unary operator (e.g., negation, logical not, bitwise not).
    pub kind: UnaryOpKind,
    /// The span of the unary operator in the source code.
    pub span: Span,
}

/// An enumeration of unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum UnaryOpKind {
    /// The negation operator.
    #[strum(to_string = "-")]
    Negate,
    /// The logical not operator.
    #[strum(to_string = "!")]
    LogicalNot,
    /// The bitwise not operator.
    #[strum(to_string = "~")]
    BitwiseNot,
}
