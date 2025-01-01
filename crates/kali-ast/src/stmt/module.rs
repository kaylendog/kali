/// An import statement.
#[derive(Debug, Clone)]
pub enum Import {
    /// A list of named imports, e.g. import { x } from std.bla
    Named { symbols: Vec<String>, path: String },
    /// A wildcard import, e.g. import * from std.bla
    Wildcard { path: String },
    /// A named wildcard import, e.g. import * as bla from std.bla
    NamedWildcard { alias: String, path: String },
}

/// An export statement.
#[derive(Debug, Clone)]
pub struct Export {
    /// The symbols to export.
    pub symbols: Vec<String>,
}
