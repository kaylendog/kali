//! Printing and formatting support.

use std::io::{self, Write};

use kali_ast::{Module, WalkOrder, Walker};

mod impls;
mod pass;

pub use impls::Settings;

use crate::impls::PrettyPrint;

/// Prints a module as Kali source, including its comments, using the pretty printer.
///
/// # Arguments
///
/// - `module` - The module to print.
/// - `src` - The original source code of the module, used for comment placement.
/// - `buf` - The output buffer to write the printed source code to.
///
/// # Returns
///
/// A `Result` indicating success or failure to write to the output buffer.
pub fn print_module(module: &Module, src: &str, buf: &mut dyn Write) -> Result<(), io::Error> {
    format_module(module, src, Settings::default(), buf)
}

/// Prints a module as Kali source, including its comments, using the pretty printer with custom settings.
///
/// # Arguments
///
/// - `module` - The module to print.
/// - `src` - The original source code of the module, used for comment placement.
/// - `settings` - The settings to use for pretty printing.
/// - `buf` - The output buffer to write the printed source code to.
///
/// # Returns
///
/// A `Result` indicating success or failure to write to the output buffer.
pub fn format_module(
    module: &Module,
    src: &str,
    settings: Settings,
    buf: &mut dyn Write,
) -> Result<(), io::Error> {
    let mut pass = pass::CommentsPass::new(&module.comments, src);
    Walker::new(WalkOrder::PreOrder)
        .walk_module(&mut pass, module)
        // Safety: `CommentsPass` never returns an error
        .unwrap();
    let mut ctx = impls::Context::new(settings, &module.cache, src, pass.finish());
    let doc = module.pretty_print(&mut ctx);
    impls::render(&doc, settings, buf)?;
    Ok(())
}
