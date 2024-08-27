/// An import statement.
pub struct Import {
    /// The path to the module being imported.
    pub path: String,
    /// A list of symbols to import.
    pub symbols: Vec<String>,
}

impl Import {
    /// Creates a new import statement.
    pub fn new(path: String, symbols: Vec<String>) -> Self {
        Self { path, symbols }
    }
}

/// An export statement.
pub struct Export {
    /// The symbols to export.
    pub symbols: Vec<String>,
}

impl Export {
    /// Creates a new export statement.
    pub fn new(symbols: Vec<String>) -> Self {
        Self { symbols }
    }
}
