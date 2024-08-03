//! Attributes, decorators, and annotations.

/// An attribute.
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// The value of the attribute.
    pub value: Option<String>,
}
