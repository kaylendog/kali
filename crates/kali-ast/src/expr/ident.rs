///! Identifiers.
use std::hash::Hash;

/// An identifier.
#[derive(Debug, Clone)]
pub struct Identifier<Meta> {
    /// The value of this identifier.
    pub value: String,
    /// Meta associated with this node.
    pub meta: Meta,
}

impl<Meta> PartialEq for Identifier<Meta> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<Meta> Eq for Identifier<Meta> {}

impl<Meta> Hash for Identifier<Meta> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
