//! A lexer for the Kali programming language.

use std::num::ParseIntError;

use logos::Logos;

/// An enumeration of possbile tokens that can be lexed from source code.
#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(error = LexicalError)]
pub enum Token<'src> {
    #[token("if")]
    KeywordIf,
    #[token("then")]
    KeywordThen,
    #[token("else")]
    KeywordElse,
    #[token("match")]
    KeywordMatch,
    #[token("with")]
    KeywordWith,
    #[token("let")]
    KeywordLet,
    #[token("fn")]
    KeywordFn,
    #[token("type")]
    KeywordType,
    #[token("import")]
    KeywordImport,
    #[token("export")]
    KeywordExport,
    #[token("from")]
    KeywordFrom,
    #[token("as")]
    KeywordAs,
    #[token("int")]
    TypeInteger,
    #[token("nat")]
    TypeNatural,
    #[token("float")]
    TypeFloat,
    #[token("bool")]
    TypeBool,
    #[token("string")]
    TypeString,
    #[token("=")]
    OpAssign,
    #[token("==")]
    OpEqual,
    #[token("!=")]
    OpNotEqual,
    #[token("<")]
    OpLessThan,
    #[token("<=")]
    OpLessThanOrEqual,
    #[token(">")]
    OpGreaterThan,
    #[token(">=")]
    OpGreaterThanOrEqual,
    #[token("+")]
    OpAdd,
    #[token("-")]
    OpSubtract,
    #[token("*")]
    OpMultiply,
    #[token("/")]
    OpDivide,
    #[token("%")]
    OpModulo,
    #[token("**")]
    OpExponentiate,
    #[token("::")]
    OpCons,
    #[token("!")]
    OpNegate,
    #[token("~")]
    OpBitwiseNot,
    #[token("&&")]
    OpLogicalAnd,
    #[token("||")]
    OpLogicalOr,
    #[token("@")]
    OpConcat,
    #[token("&")]
    OpBitwiseAnd,
    #[token("|")]
    OpBitwiseOr,
    #[token("^")]
    OpBitwiseXor,
    #[token("<<")]
    OpBitwiseShiftLeft,
    #[token(">>")]
    OpBitwiseShiftRight,
    #[regex("(\\w|_)+", priority = 0)]
    Ident(&'src str),
    // literals
    #[regex("[0-9][0-9_]*", |lex| lex.slice().parse().ok(), priority = 1)]
    #[regex("0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| prefixed_natural(lex))]
    #[regex("0b[01][01_]*", |lex| prefixed_natural(lex))]
    #[regex("0o[0-7][0-7_]*", |lex| prefixed_natural(lex))]
    #[regex("0d[0-9][0-9_]*", |lex| prefixed_natural(lex))]
    LitNatural(u64),
    #[regex("-[0-9][0-9_]*", |lex| lex.slice().parse().ok())]
    #[regex("-0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| prefixed_integer(lex))]
    #[regex("-0b[01][01_]*", |lex| prefixed_integer(lex))]
    #[regex("-0o[0-7][0-7_]*", |lex| prefixed_integer(lex))]
    #[regex("-0d[0-9][0-9_]*", |lex| prefixed_integer(lex))]
    LitInteger(i64),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    LitBool(bool),
    #[token("()")]
    LitUnit,
    #[regex(r#""[^"]*""#, |lex| &lex.slice()[1..lex.slice().len() - 1])]
    LitString(&'src str),
    // symbols
    #[token("(")]
    SymLParen,
    #[token(")")]
    SymRParen,
    #[token("[")]
    SymLBracket,
    #[token("]")]
    SymRBracket,
    #[token("{")]
    SymLBrace,
    #[token("}")]
    SymRBrace,
    #[token(",")]
    SymComma,
    #[token(":")]
    SymColon,
    #[token("->")]
    SymArrow,
    #[token("[]")]
    SymArray,
    #[token("_")]
    SymWildcard,
    #[token(";")]
    SymSemicolon,
    #[token("...")]
    SymRest,

    #[regex("[\n\t ]+", logos::skip)]
    Whitespace,

    // `allow_greedy` is fine since we prefix it with `#`.
    #[regex("#.*\n", logos::skip, allow_greedy = true)]
    Comment,

    Error(LexicalError),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

fn prefixed_natural<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> Option<u64> {
    let slice = lex.slice();
    let radix = match &slice[..2] {
        "0x" => 16,
        "0b" => 2,
        "0o" => 8,
        "0d" => 10,
        _ => unreachable!(),
    };
    // remove underscore separators and parse
    let slice = slice[2..].replace("_", "");
    u64::from_str_radix(&slice, radix).ok()
}

fn prefixed_integer<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> Option<i64> {
    let slice = lex.slice();
    let negative = slice.starts_with('-');
    let radix = match &slice[1..3] {
        "0x" => 16,
        "0b" => 2,
        "0o" => 8,
        "0d" => 10,
        _ => unreachable!(),
    };
    // remove underscore separators and parse
    let slice = slice[3..].replace("_", "");
    i64::from_str_radix(&slice, radix)
        .ok()
        .map(|n| if negative { -n } else { n })
}
