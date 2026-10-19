//! Main entrypoints for the Kali compiler.
//!
//! This crate will eventually handle cross-module compilation, module codegen,
//! and linking, but for now it just provides a simple interface to the parser
//! and type checker.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc,
};

/// Re-export of error type from the `kali-error` crate.
pub use kali_error::Error;

/// A context for parsing and type checking a kali program.
#[derive(Default)]
pub struct Context {
    paths: BTreeMap<kali_span::SourceId, PathBuf>,
    sources: BTreeMap<kali_span::SourceId, ariadne::Source<rc::Rc<str>>>,
    modules: BTreeMap<kali_span::SourceId, kali_ast::Module>,
}

impl Context {
    /// Get a reference to the module corresponding to the given source ID, if
    /// it has been parsed and type checked.
    pub fn get_module(&self, id: kali_span::SourceId) -> Option<&kali_ast::Module> {
        self.modules.get(&id)
    }

    /// Load a Kali source file at the given path into the context, returning a
    /// `SourceId` that can be used to refer to the source file in subsequent
    /// operations.
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<kali_span::SourceId, Vec<Error>> {
        let src = std::fs::read_to_string(&path).map_err(|e| vec![e.into()])?;
        let id = kali_span::SourceId(self.sources.len());
        self.sources.insert(id, ariadne::Source::from(rc::Rc::from(src)));
        self.paths.insert(id, path.as_ref().to_path_buf());
        Ok(id)
    }

    /// Parses a kali source file at the given path, returning the parsed module
    /// or an error if parsing fails.
    pub fn parse(&mut self, id: kali_span::SourceId) -> Result<(), Vec<Error>> {
        let src = &self.sources[&id];
        let context = kali_span::Context { src: id };
        let (module, parse_errors) = kali_parse::parse_str(src.text(), context);
        // If parsing failed, return errors.
        let Some(module) = module else {
            return Err(parse_errors.into_iter().map(Into::into).collect());
        };
        self.modules.insert(context.src, module);
        Ok(())
    }

    /// Type checks a kali module, returning an error if type checking fails.
    pub fn typecheck(&mut self, src_id: kali_span::SourceId) -> Result<(), Vec<Error>> {
        let module = self.modules.get(&src_id).expect("Module not found for source ID");
        let infer = kali_type::check(module);
        if !infer.errors.is_empty() {
            return Err(infer.errors.into_iter().map(Into::into).collect::<Vec<_>>());
        }
        Ok(())
    }

    /// Compiles a kali source file at the given path, returning the parsed
    /// module or an error if parsing or type checking fails.
    pub fn compile(&mut self, path: impl AsRef<Path>) -> Result<&kali_ast::Module, Vec<Error>> {
        let id = self.load(path)?;
        self.parse(id)?;
        self.typecheck(id)?;
        Ok(&self.modules[&id])
    }

    /// Formats a kali source file at the given path, returning the formatted
    /// source
    pub fn format_one(
        &mut self,
        id: kali_span::SourceId,
        settings: kali_print::Settings,
    ) -> Result<String, Vec<Error>> {
        let module = self.modules.get(&id).expect("Module not found for source ID");
        let src = &self.sources[&id];
        let mut output = Vec::new();
        kali_print::format_module(module, src.text(), settings, &mut output)
            .map_err(|e| vec![e.into()])?;
        Ok(String::from_utf8(output).expect("Output is not valid UTF-8"))
    }

    /// Formats a kali source file at the given path with default settings,
    /// returning the formatted source
    pub fn format_one_default(&mut self, id: kali_span::SourceId) -> Result<String, Vec<Error>> {
        self.format_one(id, kali_print::Settings::default())
    }
}

impl ariadne::Cache<kali_span::SourceId> for Context {
    type Storage = rc::Rc<str>;

    fn fetch(
        &mut self,
        id: &kali_span::SourceId,
    ) -> Result<&ariadne::Source<Self::Storage>, Box<dyn std::fmt::Debug + '_>> {
        self.sources.get(id).ok_or_else(|| {
            Box::new(format!("Source ID {:?} not found in context", id)) as Box<dyn std::fmt::Debug>
        })
    }

    fn display<'a>(&self, id: &'a kali_span::SourceId) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.paths
            .get(id)
            .map(|path| Box::new(path.to_string_lossy().into_owned()) as Box<dyn std::fmt::Display>)
    }
}
