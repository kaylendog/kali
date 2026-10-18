use std::{collections::BTreeMap, hash::Hash};

use kali_macros::Node;

mod span;
mod traits;
mod visit;

pub use span::Span;
pub use traits::Node;
pub use visit::{Visitor, WalkOrder, Walker};

/// Represents a module in the source code, containing a collection of items and
/// a string interning cache.
#[derive(Debug, Clone)]
pub struct Module {
    /// The items defined within the module, such as functions, structs, or
    /// other modules.
    pub items: Vec<Item>,
    /// A string interning cache used to efficiently store and retrieve strings
    /// within the module.
    pub cache: lasso::Rodeo,
    /// A collection of comments found in the source code, keyed by their span.
    pub comments: BTreeMap<Span, Comment>,
}

/// The type used to uniquely identify nodes in the abstract syntax tree (AST).
/// Aliased in case I ever change this to something fancier like `rustc`'s
/// `NodeId` type.
pub type NodeId = u32;

/// Represents an identifier in the source code, including its textual value and
/// span.
#[derive(Debug, Clone, Copy, Node)]
pub struct Ident {
    /// A unique identifier for the identifier in the abstract syntax tree
    /// (AST).
    #[id]
    pub id: NodeId,
    /// The index of the identifier in the table.
    pub key: lasso::Spur,
    /// The span of the identifier in the source code.
    #[span]
    pub span: Span,
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

/// Represents one of the three types of comments found in the source code.
#[derive(Debug, Clone)]
pub struct Comment {
    /// The contents of the comment.
    pub contents: lasso::Spur,
    /// The kind of comment.
    pub kind: CommentKind,
    /// The span of the comment in the source code.
    pub span: Span,
}

/// An enumeration of the three kinds of comment.
#[derive(Debug, Clone, Copy)]
pub enum CommentKind {
    /// A normal comment, e.g. `// Hello, world!`
    Single,
    /// A doc comment, e.g. `/// Hello, world!`
    Doc,
}

/// Represents an item in the source code, such as a function, struct, or
/// module.
#[derive(Debug, Clone, Node)]
pub struct Item {
    /// A unique identifier for the item in the abstract syntax tree (AST).
    #[id]
    pub id: NodeId,
    /// The span of the item in the source code.
    #[span]
    pub span: Span,
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
    /// Represents a sum type declaration.
    TypeDecl(TypeDecl),
    /// Represents a definition in the source code.
    Definition(Definition),
}

impl ItemKind {
    /// Return the [`ImportTree`] if this item is an import, otherwise return
    /// `None`.
    pub fn as_import(&self) -> Option<&ImportTree> {
        if let ItemKind::Import(tree) = self { Some(tree) } else { None }
    }

    /// Return the [`TypeAlias`] if this item is a type alias, otherwise return
    /// `None`.
    pub fn as_type_alias(&self) -> Option<&TypeAlias> {
        if let ItemKind::TypeAlias(alias) = self { Some(alias) } else { None }
    }

    /// Return the [`TypeDecl`] if this item is a type declaration, otherwise
    /// return `None`.
    pub fn as_type_decl(&self) -> Option<&TypeDecl> {
        if let ItemKind::TypeDecl(decl) = self { Some(decl) } else { None }
    }

    /// Return the [`Definition`] if this item is a definition, otherwise return
    /// `None`.
    pub fn as_definition(&self) -> Option<&Definition> {
        if let ItemKind::Definition(def) = self { Some(def) } else { None }
    }
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
#[derive(Debug, Clone, Node)]
pub struct ImportTree {
    /// A unique identifier for the import tree in the abstract syntax tree
    /// (AST).
    #[id]
    pub id: NodeId,
    /// The kind of the import tree.
    pub kind: ImportTreeKind,
    /// The span of the import tree in the source code.
    #[span]
    pub span: Span,
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
    /// Represents a segment in the import tree with a name and a child node,
    /// e.g. `module::`
    Segment {
        /// The name of the segment.
        name: Ident,
        /// The child node of the segment.
        child: Box<ImportTree>,
    },
    /// Represents a glob import (e.g., `*`).
    Glob,
    /// Represents a list of import trees.
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

/// A sum type declaration, e.g. `type Shape = Circle(int) | Point`.
#[derive(Debug, Clone)]
pub struct TypeDecl {
    /// The name of the type.
    pub name: Ident,
    /// Generic parameters. Always empty until generics are parsed.
    pub params: Vec<Ident>,
    /// The variants, in declaration order.
    pub variants: Vec<Variant>,
}

/// One variant of a sum type. Its constructor is a function from the payload to
/// the type.
#[derive(Debug, Clone, Node)]
pub struct Variant {
    /// A unique identifier for the variant in the abstract syntax tree (AST).
    #[id]
    pub id: NodeId,
    /// The constructor name.
    pub name: Ident,
    /// The payload type. `Circle(int)` is `int`, `Rectangle(int, int)` is a
    /// tuple, `Point` is none.
    pub payload: Option<Type>,
    /// The span of the variant in the source code.
    #[span]
    pub span: Span,
}

/// Represents a type in the source code, including its span and kind.
#[derive(Debug, Clone, Node)]
pub struct Type {
    /// A unique identifier for the type in the abstract syntax tree (AST).
    #[id]
    pub id: NodeId,
    /// The span of the type in the source code.
    #[span]
    pub span: Span,
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
}

/// Enumeration of primitive types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTypeKind {
    /// Integer type.
    Integer,
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
    /// The pattern that defines the variable(s) being introduced.
    pub name: Pattern,
    /// The expression associated with the definition.
    pub expr: Expr,
}

/// Represents an expression item in the source code.
#[derive(Debug, Clone, Node)]
pub struct Expr {
    /// A unique identifier for the expression in the abstract syntax tree
    /// (AST).
    #[id]
    pub id: NodeId,
    #[span]
    /// The span of the expression in the source code.
    pub span: Span,
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
        then: Box<Expr>,
        /// The expression to execute if the condition is false.
        otherwise: Box<Expr>,
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

#[derive(Debug, Clone, PartialEq)]
/// Represents the kind of literal in the source code.
pub enum LiteralKind {
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

// Safety: the parser cannot produce non-finite floats.
impl Eq for LiteralKind {}

/// Represents a binary operator in the source code, including its kind and
/// span.
#[derive(Debug, Clone, Node)]
pub struct BinaryOp {
    /// A unique identifier for the binary operator in the abstract syntax tree
    /// (AST).
    #[id]
    pub id: NodeId,
    /// The kind of binary operator (e.g., addition, subtraction, etc.).
    pub kind: BinaryOpKind,
    /// The span of the binary operator in the source code.
    #[span]
    pub span: Span,
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
#[derive(Debug, Clone, Node)]
pub struct UnaryOp {
    /// A unique identifier for the unary operator in the abstract syntax tree
    /// (AST).
    #[id]
    pub id: NodeId,
    /// The kind of unary operator (e.g., negation, logical not, bitwise not).
    pub kind: UnaryOpKind,
    /// The span of the unary operator in the source code.
    #[span]
    pub span: Span,
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
#[derive(Debug, Clone, Node)]
pub struct MatchArm {
    /// A unique identifier for the arm in the abstract syntax tree (AST).
    #[id]
    pub id: NodeId,
    /// The span of the match arm in the source code.
    #[span]
    pub span: Span,
    /// The pattern to match.
    pub pattern: Pattern,
    /// The expression to execute if the pattern matches.
    pub expr: Expr,
}

/// Represents the kind of pattern in a match arm.
#[derive(Debug, Clone, PartialEq, Eq, Node)]
pub struct Pattern {
    /// A unique identifier for the pattern in the abstract syntax tree (AST).
    #[id]
    pub id: NodeId,
    /// The span of the pattern in the source code.
    #[span]
    pub span: Span,
    /// The kind of pattern (literal, identifier, tuple, etc.).
    pub kind: PatternKind,
}

/// The different kinds of patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Cons { lhs: Box<Pattern>, rhs: Box<Pattern> },
    /// The empty list.
    EmptyList,
    /// An or-pattern (e.g., `A | B`).
    Or { lhs: Box<Pattern>, rhs: Box<Pattern> },
    /// A constructor pattern (e.g., `Circle(r)` or `Point`).
    Constructor {
        /// The constructor name.
        name: Ident,
        /// The payload pattern. `Rectangle(w, h)` has a tuple payload.
        payload: Option<Box<Pattern>>,
    },
    /// A rest pattern.
    Rest,
}

/// Argument to a lambda expression.
#[derive(Debug, Clone)]
pub struct LambdaParam {
    /// A pattern that unpacks the argument.
    pub parameter: Pattern,
    /// The optional type annotation for the argument.
    pub ty: Option<Type>,
}
