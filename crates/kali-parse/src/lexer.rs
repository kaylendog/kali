//! A lexer for the Kali programming language.

use std::num::ParseIntError;

use logos::Logos;
use strum::Display;

/// An enumeration of possbile tokens that can be lexed from source code.
#[derive(Debug, Clone, PartialEq, Display, Logos)]
#[logos(error = LexicalError)]
pub enum Token<'src> {
    #[token("if")]
    #[strum(to_string = "if")]
    KeywordIf,
    #[token("then")]
    #[strum(to_string = "then")]
    KeywordThen,
    #[token("else")]
    #[strum(to_string = "else")]
    KeywordElse,
    #[token("match")]
    #[strum(to_string = "match")]
    KeywordMatch,
    #[token("with")]
    #[strum(to_string = "with")]
    KeywordWith,
    #[token("let")]
    #[strum(to_string = "let")]
    KeywordLet,
    #[token("fn")]
    #[strum(to_string = "fn")]
    KeywordFn,
    #[token("type")]
    #[strum(to_string = "type")]
    KeywordType,
    #[token("import")]
    #[strum(to_string = "import")]
    KeywordImport,
    #[token("export")]
    #[strum(to_string = "export")]
    KeywordExport,
    #[token("from")]
    #[strum(to_string = "from")]
    KeywordFrom,
    #[token("as")]
    #[strum(to_string = "as")]
    KeywordAs,
    #[token("int")]
    #[strum(to_string = "int")]
    TypeInteger,
    #[token("float")]
    #[strum(to_string = "float")]
    TypeFloat,
    #[token("bool")]
    #[strum(to_string = "bool")]
    TypeBool,
    #[token("string")]
    #[strum(to_string = "string")]
    TypeString,

    #[token("=")]
    #[strum(to_string = "=")]
    OpAssign,
    #[token("==")]
    #[strum(to_string = "==")]
    OpEqual,
    #[token("!=")]
    #[strum(to_string = "!=")]
    OpNotEqual,
    #[token("<")]
    #[strum(to_string = "<")]
    OpLessThan,
    #[token("<=")]
    #[strum(to_string = "<=")]
    OpLessThanOrEqual,
    #[token(">")]
    #[strum(to_string = ">")]
    OpGreaterThan,
    #[token(">=")]
    #[strum(to_string = ">=")]
    OpGreaterThanOrEqual,
    #[token("+")]
    #[strum(to_string = "+")]
    OpAdd,
    #[token("-")]
    #[strum(to_string = "-")]
    OpSubtract,
    #[token("*")]
    #[strum(to_string = "*")]
    OpMultiply,
    #[token("/")]
    #[strum(to_string = "/")]
    OpDivide,
    #[token("%")]
    #[strum(to_string = "%")]
    OpModulo,
    #[token("**")]
    #[strum(to_string = "**")]
    OpExponentiate,
    #[token("::")]
    #[strum(to_string = "::")]
    OpCons,
    #[token("!")]
    #[strum(to_string = "!")]
    OpNegate,
    #[token("~")]
    #[strum(to_string = "~")]
    OpBitwiseNot,
    #[token("&&")]
    #[strum(to_string = "&&")]
    OpLogicalAnd,
    #[token("||")]
    #[strum(to_string = "||")]
    OpLogicalOr,
    #[token("@")]
    #[strum(to_string = "@")]
    OpConcat,
    #[token("&")]
    #[strum(to_string = "&")]
    OpBitwiseAnd,
    #[token("|")]
    #[strum(to_string = "|")]
    OpBitwiseOr,
    #[token("^")]
    #[strum(to_string = "^")]
    OpBitwiseXor,
    #[token("<<")]
    #[strum(to_string = "<<")]
    OpBitwiseShiftLeft,
    #[token(">>")]
    #[strum(to_string = ">>")]
    OpBitwiseShiftRight,

    #[regex("(\\w|_)+", priority = 0)]
    #[strum(to_string = "identifier")]
    Ident(&'src str),

    // literals
    #[regex("[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse().ok())]
    #[regex("0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| prefixed_integer(lex))]
    #[regex("0b[01][01_]*", |lex| prefixed_integer(lex))]
    #[regex("0o[0-7][0-7_]*", |lex| prefixed_integer(lex))]
    #[regex("0d[0-9][0-9_]*", |lex| prefixed_integer(lex))]
    #[strum(to_string = "integer literal")]
    LitInteger(i64),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    #[strum(to_string = "boolean literal")]
    LitBool(bool),
    #[token("()")]
    #[strum(to_string = "unit literal")]
    LitUnit,
    #[regex(r#""[^"]*""#, |lex| &lex.slice()[1..lex.slice().len() - 1])]
    #[strum(to_string = "string literal")]
    LitString(&'src str),
    // symbols
    #[token("(")]
    #[strum(to_string = "(")]
    SymLParen,
    #[token(")")]
    #[strum(to_string = ")")]
    SymRParen,
    #[token("[")]
    #[strum(to_string = "[")]
    SymLBracket,
    #[token("]")]
    #[strum(to_string = "]")]
    SymRBracket,
    #[token("{")]
    #[strum(to_string = "{")]
    SymLBrace,
    #[token("}")]
    #[strum(to_string = "{}}")]
    SymRBrace,
    #[token(",")]
    #[strum(to_string = ",")]
    SymComma,
    #[token(":")]
    #[strum(to_string = ":")]
    SymColon,
    #[token("->")]
    #[strum(to_string = "->")]
    SymArrow,
    #[token("[]")]
    #[strum(to_string = "[]")]
    SymArray,
    #[token("_")]
    #[strum(to_string = "_")]
    SymWildcard,
    #[token(";")]
    #[strum(to_string = ";")]
    SymSemicolon,
    #[token("...")]
    #[strum(to_string = "...")]
    SymRest,

    #[regex("[\t\r ]+", logos::skip)]
    #[strum(to_string = "whitespace")]
    Whitespace,

    // Kept so the parser can insert `;` at line ends, Go style. Dropped before parsing.
    #[token("\n")]
    #[strum(to_string = "newline")]
    Newline,

    #[regex("//", |lex| comment(lex))]
    #[strum(to_string = "comment")]
    Comment(&'src str),

    #[regex("///", |lex| comment(lex))]
    #[strum(to_string = "doc comment")]
    DocComment(&'src str),

    #[strum(to_string = "error")]
    Error(LexicalError),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

fn prefixed_integer<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> Option<i64> {
    let slice = lex.slice();
    let radix = match &slice[..2] {
        "0x" => 16,
        "0b" => 2,
        "0o" => 8,
        "0d" => 10,
        _ => unreachable!(),
    };
    i64::from_str_radix(
        // remove underscores and base prefix
        &slice[2..].replace('_', ""),
        radix,
    )
    .ok()
}

fn comment<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> Option<&'src str> {
    let start = lex.span().end;
    let remainder = lex.remainder();
    // Scan until EOL or EOI.
    let end = start + remainder.find("\n").unwrap_or(remainder.len());
    lex.bump(end - start);
    Some(&lex.source()[start..end].trim_start())
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::Token;

    #[test]
    fn lex_comment() {
        let src = r#"
            // Hello, world!
            /// Hello, world!
            1234 // Hello, world!
            5678 /// Hello, world!"#;
        let tokens = Token::lexer(src)
            .filter_map(|r| r.ok())
            .filter(|tok| *tok != Token::Newline)
            .collect::<Vec<Token>>();

        pretty_assertions::assert_eq!(
            vec![
                Token::Comment("Hello, world!"),
                Token::DocComment("Hello, world!"),
                Token::LitInteger(1234),
                Token::Comment("Hello, world!"),
                Token::LitInteger(5678),
                Token::DocComment("Hello, world!")
            ],
            tokens
        );
    }
}
