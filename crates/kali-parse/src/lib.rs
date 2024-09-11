use std::{error::Error, path::Path};

use kali_ast::{Node, Stmt};
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

/// Parse a file into an AST.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<Node<Stmt>>, Box<dyn Error>> {
    let input = std::fs::read_to_string(path)?;
    parse_str(&input)
}

/// Parse a string into an AST.
pub fn parse_str(input: &str) -> Result<Vec<Node<Stmt>>, Box<dyn Error>> {
    let stmts = grammar::StmtsParser::new().parse(input).unwrap();
    Ok(stmts)
}
