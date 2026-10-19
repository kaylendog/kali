use crate::NodeId;

/// A trait implemented by all AST nodes.
pub trait Node {
    /// Return the ID of this node in the AST.
    fn id(&self) -> NodeId;

    /// Return the span of the source code comprising this node.
    fn span(&self) -> &kali_span::Span;
}
