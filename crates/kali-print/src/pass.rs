use std::{collections::BTreeMap, convert::Infallible, ops::Range};

use kali_ast::{Comment, NodeId, Span};

/// Comments attached to one AST node.
#[derive(Debug, Clone, Default)]
pub(crate) struct Comments {
    /// Comments on the lines before the node.
    pub(crate) leading: Vec<Comment>,
    /// Comments after the node that no later node claims.
    pub(crate) trailing: Vec<Comment>,
    /// Comments inside the node's span.
    pub(crate) dangling: Vec<Comment>,
}

/// Attaches each comment to exactly one node. Items must be visited in source
/// order.
pub(crate) struct CommentsPass<'a> {
    /// Every comment in the module, keyed by span.
    raw: &'a BTreeMap<Span, Comment>,
    /// Original source, used to tell `let a = 1 // note` from a comment on its
    /// own line.
    src: &'a str,
    /// Comments claimed so far, keyed by node.
    resolved: BTreeMap<NodeId, Comments>,
    /// End of the last item visited. Comments between here and the next item
    /// lead that item.
    cursor: usize,
    /// The last item visited, which an end-of-line comment trails.
    previous: Option<NodeId>,
}

impl<'a> CommentsPass<'a> {
    pub(crate) fn new(raw: &'a BTreeMap<Span, Comment>, src: &'a str) -> Self {
        Self { raw, src, resolved: BTreeMap::new(), cursor: 0, previous: None }
    }

    /// Whether `comment` sits on the same line as the end of the previous item.
    fn is_end_of_line(&self, comment: &Comment) -> bool {
        self.src.get(self.cursor..comment.span.start).is_some_and(|gap| !gap.contains('\n'))
    }

    /// Returns the comments starting within `range`, in source order.
    fn claim(&self, range: Range<usize>) -> Vec<Comment> {
        let lower: Span = (range.start..range.start).into();
        let upper: Span = (range.end..range.end).into();
        self.raw.range(lower..upper).map(|(_, comment)| comment.clone()).collect()
    }

    /// Attaches any comments after the last item as trailing comments of that
    /// item.
    pub(crate) fn finish(mut self) -> BTreeMap<NodeId, Comments> {
        let rest = self.claim(self.cursor..usize::MAX);
        if let Some(previous) = self.previous {
            self.resolved.entry(previous).or_default().trailing = rest;
        }
        self.resolved
    }
}

impl kali_ast::Visitor for CommentsPass<'_> {
    type Error = Infallible;

    fn visit_item(&mut self, item: &kali_ast::Item) -> Result<(), Self::Error> {
        let mut leading = self.claim(self.cursor..item.span.start);
        // `let a = 1; // note` trails `a` rather than leading the next item.
        if let Some(previous) = self.previous
            && leading.first().is_some_and(|comment| self.is_end_of_line(comment))
        {
            let comment = leading.remove(0);
            self.resolved.entry(previous).or_default().trailing.push(comment);
        }
        let comments = Comments {
            leading,
            dangling: self.claim(item.span.start..item.span.end),
            trailing: Vec::new(),
        };
        self.cursor = item.span.end;
        self.previous = Some(item.id);
        self.resolved.insert(item.id, comments);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use kali_ast::Visitor;

    use super::CommentsPass;

    #[test]
    fn comments_attach_to_one_item_each() {
        let src = "// leading one\nlet one = 1; // after one\nlet two = 2\n// trailing two";
        let (module, errors) = kali_parse::parse_str(src);
        assert!(errors.is_empty(), "{errors:?}");
        let module = module.unwrap();

        let mut pass = CommentsPass::new(&module.comments, src);
        for item in &module.items {
            pass.visit_item(item).unwrap();
        }
        let resolved = pass.finish();

        let counts: Vec<_> = module
            .items
            .iter()
            .map(|item| {
                let comments = &resolved[&item.id];
                (comments.leading.len(), comments.dangling.len(), comments.trailing.len())
            })
            .collect();
        assert_eq!(counts, vec![(1, 0, 1), (0, 0, 1)]);
    }
}
