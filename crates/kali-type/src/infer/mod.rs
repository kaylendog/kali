//! The inference state: the scope (variables, levels and namespaces), the
//! node-to-type cache, and the errors collected so far. Everything that can
//! fail lives on `Infer`, one concern per file: `unify`, `expr`. Everything
//! that cannot lives on `Scope`: `scope`, `scheme`.

use crate::{
    cache::Cache,
    error::{Error, ErrorKind, Origin},
    scope::Scope,
    types::Type,
};

mod expr;

/// The inference engine state.
#[derive(Debug, Default)]
pub struct Infer {
    /// The scope manager.
    pub scope: Scope,
    /// The inferred type cache.
    pub(crate) cache: Cache,
    /// Errors collected during inference.
    pub errors: Vec<Error>,
}

impl Infer {
    /// Record an error and return the poisoned type, which unifies with
    /// anything silently.
    pub(crate) fn error(&mut self, kind: ErrorKind, origin: Origin) -> Type {
        self.errors.push(Error { kind, origin });
        Type::Error
    }
}
