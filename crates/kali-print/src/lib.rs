//! Printing and formatting support.

use std::io::Write;

use kali_ast::{Module, WalkOrder, Walker};

mod error;
mod impls;
mod pass;

pub use error::{Error, Result};
pub use impls::{Context, Print};

/// Prints a module as Kali source, including its comments. `src` is the text the module was
/// parsed from, used to keep blank lines and end-of-line comments where they were.
pub fn print_module(module: &Module, src: &str, buf: &mut dyn Write) -> Result<()> {
    let mut pass = pass::CommentsPass::new(&module.comments, src);
    Walker::new(WalkOrder::PreOrder).walk_module(&mut pass, module)?;
    let mut ctx = Context::new(buf, &module.cache, src);
    ctx.comments = pass.finish();
    module.print(&mut ctx)
}
