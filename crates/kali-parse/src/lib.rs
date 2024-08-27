use std::{error::Error, path::Path};

use kali_ast::{Expr, Stmt};
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

/// Parse a file into an AST.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Stmt, Box<dyn Error>> {
    let input = std::fs::read_to_string(path)?;
    parse_str(&input)
}

/// Parse a string into an AST.
pub fn parse_str(input: &str) -> Result<Stmt, Box<dyn Error>> {
    let stmt = grammar::StmtParser::new().parse(input).unwrap();
    Ok(stmt)
}
