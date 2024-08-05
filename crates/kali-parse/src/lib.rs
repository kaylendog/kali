use std::{error::Error, path::Path};

use kali_ast::Expr;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

/// Parse a file into an AST.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Expr, Box<dyn Error>> {
    let input = std::fs::read_to_string(path)?;
    let expr = grammar::ExprParser::new().parse(&input).unwrap();
    Ok(expr)
}

/// Parse a string into an AST.
pub fn parse_str(input: &str) -> Result<Expr, Box<dyn Error>> {
    let expr = grammar::ExprParser::new().parse(input).unwrap();
    Ok(expr)
}
