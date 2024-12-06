use kali_ast::{Node, Stmt};
use lalrpop_util::{lalrpop_mod, lexer::Token};

lalrpop_mod!(pub grammar);

pub type ParseError<'src> = lalrpop_util::ParseError<usize, Token<'src>, &'static str>;

/// Parse a string into an AST.
pub fn parse_str<'src>(input: &'src str) -> Result<Vec<Node<Stmt>>, ParseError<'src>> {
    grammar::StmtsParser::new().parse(input)
}
