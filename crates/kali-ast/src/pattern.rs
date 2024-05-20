/// Represents a pattern in the AST.
pub enum Pattern {
    /// A wildcard pattern.
    Wildcard,
    /// An identifier pattern.
    Identifier(String),
    /// A literal pattern.
    Tuple(Vec<Pattern>),
    /// A struct pattern.
    Record(Vec<(String, Pattern)>),
    List(Vec<Pattern>),
    /// A tuple pattern.
    Cons(Box<Pattern>, Box<Pattern>),
    /// A range pattern.
    Range(Box<Pattern>, Box<Pattern>),
}
