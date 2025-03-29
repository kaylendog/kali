//! Imports and exports.

use crate::Identifier;

/// An import statement.
#[derive(Debug, Clone)]
pub struct Import<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The kind of import.
    pub kind: ImportKind<Meta>,
}

/// The kind of import statement.
#[derive(Debug, Clone)]
pub enum ImportKind<Meta> {
    /// A list of named imports, e.g. import { x } from std.bla
    Named {
        symbols: Vec<Identifier<Meta>>,
        path: String,
    },
    /// A wildcard import, e.g. import * from std.bla
    Wildcard { path: String },
    /// A named wildcard import, e.g. import * as bla from std.bla
    NamedWildcard {
        alias: Identifier<Meta>,
        path: String,
    },
}

/// An export statement.
#[derive(Debug, Clone)]
pub struct Export<Meta> {
    /// Meta for this node.
    pub meta: Meta,
    /// The symbols to export.
    pub symbols: Vec<Identifier<Meta>>,
}
