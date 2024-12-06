use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use kali_ast::Stmt;
use kali_ir::Module;
use kali_type::Typed;

/// Compile into a module.
pub fn build_module<'src, P: AsRef<Path>>(
    path: P,
    contents: &'src str,
    print_ast: bool,
) -> Result<Module, kali_error::Error<'src>> {
    let module = kali_parse::parse_str(contents).map_err(kali_error::Error::SyntaxError)?;

    println!("{:#?}", module);

    // type check the program
    let mut ctx = kali_type::Context::default();

    // declare top-level types
    for stmt in module
        .iter()
        .filter(|stmt| matches!(stmt.inner, Stmt::Type(..)))
    {
        let ty = stmt.ty(&mut ctx);
        if let Err(e) = ty {
            eprintln!("Type Error: {}", e);
        }
    }

    // type check declarations
    for stmt in module
        .iter()
        .filter(|stmt| !matches!(stmt.inner, Stmt::Type(..)))
    {
        let ty = stmt.ty(&mut ctx).map_err(kali_error::Error::TypeError);
    }

    Ok(Module {
        version: 0,
        imports: HashMap::new(),
        functions: BTreeMap::new(),
    })
}
