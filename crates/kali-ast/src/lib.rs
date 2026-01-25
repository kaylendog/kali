use std::hash::Hash;

mod visit;

/// Represents a module in the source code, containing a collection of items and a string interning cache.
#[derive(Debug, Clone)]
pub struct Module {
    /// The items defined within the module, such as functions, structs, or other modules.
    pub items: Vec<Item>,
    /// A string interning cache used to efficiently store and retrieve strings within the module.
    pub cache: lasso::Rodeo,
}

/// Represents an identifier in the source code, including its textual value and span.
#[derive(Debug, Clone, Copy)]
pub struct Ident {
    /// The index of the identifier in the table.
    pub key: lasso::Spur,
    /// The span of the identifier in the source code.
    pub span: chumsky::span::SimpleSpan,
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Ident {}

impl Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

/// Represents an item in the source code, such as a function, struct, or module.
#[derive(Debug, Clone)]
pub struct Item {
    /// The span of the item in the source code.
    pub span: chumsky::span::SimpleSpan,
    /// The kind of the item (e.g., function, struct, etc.).
    pub kind: ItemKind,
    /// The visibility of the item (e.g., private, exported, or inherited).
    pub visibility: Visibility,
}

/// An enumeration of [`Item`] kinds.
#[derive(Debug, Clone)]
pub enum ItemKind {
    /// Represents an import statement in the source code.
    Import(ImportTree),
    /// Represents a type alias.
    TypeAlias(TypeAlias),
    /// Represents a definition in the source code.
    Definition(Definition),
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

/// A node in an import tree.
#[derive(Debug, Clone)]
pub struct ImportTree {
    /// The kind of the import tree.
    pub kind: ImportTreeKind,
    /// The span of the import tree in the source code.
    pub span: chumsky::span::SimpleSpan,
}

/// A tree of imports.
#[derive(Debug, Clone)]
pub enum ImportTreeKind {
    /// Represents a specific item in the import tree with an optional alias.
    Item {
        /// The name of the item.
        name: Ident,
        /// An optional alias for the item.
        alias: Option<Ident>,
    },
    /// Represents a segment in the import tree with a name and a child node, e.g. `module::`
    Segment {
        /// The name of the segment.
        name: Ident,
        /// The child node of the segment.
        child: Box<ImportTree>,
    },
    /// Represents a glob import (e.g., `*`).
    Glob,
    /// Represents a list wof import trees.
    List(Vec<ImportTree>),
}

/// Represents a type alias.
#[derive(Debug, Clone)]
pub struct TypeAlias {
    /// The name of the type alias.
    pub name: Ident,
    /// The aliased type.
    pub ty: Type,
}

/// Represents a type in the source code, including its span and kind.
#[derive(Debug, Clone)]
pub struct Type {
    /// The span of the type in the source code.
    pub span: chumsky::span::SimpleSpan,
    /// The kind of type (primitive, function, tuple, etc.).
    pub kind: TypeKind,
}

/// Represents the kind of type in the source code.
#[derive(Debug, Clone)]
pub enum TypeKind {
    /// A primitive type (e.g., int, float, bool, string, unit).
    Primitive(PrimitiveTypeKind),
    /// A named type, which refers to a type with a specific identifier.
    Named(Ident),
    /// A tuple type containing multiple types.
    Tuple(Vec<Type>),
    /// A list type containing elements of a single type.
    List(Box<Type>),
    /// A record type with named fields.
    Record(indexmap::IndexMap<Ident, Type>),
    /// A function type with parameters and a return type.
    Fn(Vec<Type>, Box<Type>),
    /// An intersection type, representing a type that satisfies multiple constraints.
    Intersection {
        /// The left-hand side type in the intersection.
        lhs: Box<Type>,
        /// The right-hand side type in the intersection.
        rhs: Box<Type>,
    },
    /// A union type, representing a type that can be one of several alternatives.
    Union {
        /// The left-hand side type in the union.
        lhs: Box<Type>,
        /// The right-hand side type in the union.
        rhs: Box<Type>,
    },
}

/// Enumeration of primitive types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTypeKind {
    /// Integer type.
    Integer,
    /// Natural number type.
    Natural,
    /// Floating-point type.
    Float,
    /// Boolean type.
    Bool,
    /// String type.
    String,
    /// Unit type (empty tuple).
    Unit,
}

/// Represents a definition in the source code.
#[derive(Debug, Clone)]
pub struct Definition {
    /// The name of the definition.
    pub name: Destructor,
    /// The expression associated with the definition.
    pub expr: Expr,
}

/// Represents an expression item in the source code.
#[derive(Debug, Clone)]
pub struct Expr {
    /// The span of the expression in the source code.
    pub span: chumsky::span::SimpleSpan,
    /// The kind of expression.
    pub kind: ExprKind,
}

/// Represents the kind of expression.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// A variable expression.
    Var(Ident),
    /// A literal expression.
    Literal(LiteralKind),
    /// A binary expression.
    BinaryExpr {
        /// The binary operator applied in the expression.
        op: BinaryOp,
        /// The left-hand side expression.
        lhs: Box<Expr>,
        /// The right-hand side expression.
        rhs: Box<Expr>,
    },
    /// A unary expression.
    UnaryExpr { op: UnaryOp, expr: Box<Expr> },
    /// A tuple expression.
    Tuple(Vec<Expr>),
    /// A list expression.
    List(Vec<Expr>),
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
    Match {
        /// The expression being matched on.
        value: Box<Expr>,
        /// The list of match arms.
        arms: Vec<MatchArm>,
    },
    /// A lambda expression.
    Lambda {
        /// The params to the lambda function.
        params: Vec<LambdaParam>,
        /// The optional return type of the lambda function.
        ret_ty: Option<Type>,
        /// The body of the lambda function, represented as an expression.
        body: Box<Expr>,
    },
    /// A call expression.
    Call {
        /// The function being called.
        function: Box<Expr>,
        /// The arguments passed to the function.
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
/// Represents the kind of literal in the source code.
pub enum LiteralKind {
    /// A natural number literal (e.g., `42`).
    Natural(u64),
    /// An integer literal (e.g., `-7`).
    Integer(i64),
    /// A floating-point literal (e.g., `3.14`).
    Float(f64),
    /// A boolean literal (`true` or `false`).
    Bool(bool),
    /// A string literal (e.g., `"hello"`).
    String(lasso::Spur),
    /// A unit literal (empty tuple, `()`).
    Unit,
}

/// Represents a binary operator in the source code, including its kind and span.
#[derive(Debug, Clone)]
pub struct BinaryOp {
    /// The kind of binary operator (e.g., addition, subtraction, etc.).
    pub kind: BinaryOpKind,
    /// The span of the binary operator in the source code.
    pub span: chumsky::span::SimpleSpan,
}

/// Represents the kind of binary operator in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum BinaryOpKind {
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
    #[strum(to_string = "@")]
    Concat,
}

/// Represents a unary operator in the source code, including its kind and span.
#[derive(Debug, Clone)]
pub struct UnaryOp {
    /// The kind of unary operator (e.g., negation, logical not, bitwise not).
    pub kind: UnaryOpKind,
    /// The span of the unary operator in the source code.
    pub span: chumsky::span::SimpleSpan,
}

/// An enumeration of unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum UnaryOpKind {
    /// The negation operator.
    #[strum(to_string = "-")]
    Negate,
    /// The unary plus operator.
    #[strum(to_string = "+")]
    UnaryPlus,
    /// The logical not operator.
    #[strum(to_string = "!")]
    LogicalNot,
    /// The bitwise not operator.
    #[strum(to_string = "~")]
    BitwiseNot,
}

/// Represents a single arm of a match expression.
#[derive(Debug, Clone)]
pub struct MatchArm {
    /// The span of the match arm in the source code.
    pub span: chumsky::span::SimpleSpan,
    /// The pattern to match.
    pub pattern: Pattern,
    /// The expression to execute if the pattern matches.
    pub expr: Expr,
}

/// Represents the kind of pattern in a match arm.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// The span of the pattern in the source code.
    pub span: chumsky::span::SimpleSpan,
    /// The kind of pattern (literal, identifier, tuple, etc.).
    pub kind: PatternKind,
}

/// The different kinds of patterns.
#[derive(Debug, Clone)]
pub enum PatternKind {
    /// A literal pattern (e.g., `42`, `"foo"`, `true`).
    Literal(LiteralKind),
    /// An identifier pattern (e.g., `x`).
    Var(Ident),
    /// A tuple pattern (e.g., `(a, b)`).
    Tuple(Vec<Pattern>),
    /// A wildcard pattern (`_`).
    Wildcard,
    /// A record pattern (e.g., `{ x, y }`).
    Record(indexmap::IndexMap<Ident, Pattern>),
    /// A cons pattern (e.g., `a :: b`).
    Cons {
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
    },
    /// The empty list.
    EmptyList,
    /// An or-pattern (e.g., `A | B`).
    Or {
        lhs: Box<Pattern>,
        rhs: Box<Pattern>,
    },
    /// A rest pattern.
    Rest,
}

/// Argument to a lambda expression.
#[derive(Debug, Clone)]
pub struct LambdaParam {
    /// A destructor that unpacks the argument.
    pub parameter: Destructor,
    /// The optional type annotation for the argument.
    pub ty: Option<Type>,
}

/// Represents a destructor pattern, which is a pattern with only free variables.
#[derive(Debug, Clone)]
pub struct Destructor {
    /// The span of the destructor in the source code.
    pub span: chumsky::span::SimpleSpan,
    /// THe kind of destructor.
    pub kind: DestructorKind,
}

/// Represents the kind of a destructor pattern.
#[derive(Debug, Clone)]
pub enum DestructorKind {
    /// A simple destructor with a single identifier.
    Var(Ident),
    /// A tuple destructor with multiple patterns.
    Tuple(Vec<Destructor>),
    /// A record destructor with named fields.
    Record(indexmap::IndexMap<Ident, Destructor>),
    /// A cons destructor (e.g., `a :: b`).
    Cons {
        lhs: Box<Destructor>,
        rhs: Box<Destructor>,
    },
    /// A rest destructor.
    Rest,
}
