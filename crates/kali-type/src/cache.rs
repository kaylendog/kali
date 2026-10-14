//! A cache for storing the inferred types of nodes in the abstract syntax tree
//! (AST).

use std::collections::BTreeMap;

use crate::types::Type;

/// A cache of [`kali_ast::NodeId`] to its inferred type.
#[derive(Debug, Default)]
pub struct Cache {
    inner: BTreeMap<kali_ast::NodeId, Type>,
}

impl Cache {
    /// Get the cached type of a node, if it has been inferred.
    pub fn get<T>(&self, target: &T) -> Option<&Type>
    where
        T: kali_ast::Node,
    {
        self.inner.get(&target.id())
    }

    /// Insert a type for a node into the cache and return it.
    pub fn with<T>(&mut self, target: &T, ty: Type) -> Type
    where
        T: kali_ast::Node,
    {
        self.inner.insert(target.id(), ty.clone());
        ty
    }
}
