use kali_parse::Token;

use crate::common::{assert_eq, rejects};

/// Tokens of `src`, without newlines.
fn tokens(src: &str) -> Vec<Token<'_>> {
    kali_parse::lex_str(src)
        .into_iter()
        .map(|token| token.inner)
        .filter(|token| *token != Token::Newline)
        .collect()
}

#[test]
fn integers() {
    assert_eq!(
        tokens("0 42 1_000_000"),
        vec![
            Token::LitInteger(0),
            Token::LitInteger(42),
            Token::LitInteger(1_000_000),
        ]
    );
    assert_eq!(
        tokens("0xff 0xF_F 0b1010 0o17 0d10"),
        vec![
            Token::LitInteger(255),
            Token::LitInteger(255),
            Token::LitInteger(10),
            Token::LitInteger(15),
            Token::LitInteger(10),
        ]
    );
}

#[test]
fn floats() {
    assert_eq!(
        tokens("0.0 3.14 1_000.000_001"),
        vec![
            Token::LitFloat(0.0),
            Token::LitFloat(3.14),
            Token::LitFloat(1_000.000_001),
        ]
    );
    assert_eq!(
        tokens("1e10 2E-5 3.14e+2"),
        vec![
            Token::LitFloat(1e10),
            Token::LitFloat(2e-5),
            Token::LitFloat(3.14e2),
        ]
    );
}

#[test]
fn integer_overflow_is_an_error() {
    assert!(matches!(tokens("9223372036854775808")[0], Token::Error(_)));
    assert!(matches!(
        tokens("0x1_0000_0000_0000_0000")[0],
        Token::Error(_)
    ));
    assert_eq!(
        tokens("9223372036854775807"),
        vec![Token::LitInteger(i64::MAX)]
    );
}

#[test]
fn minus_is_always_an_operator() {
    assert_eq!(
        tokens("n-1"),
        vec![Token::Ident("n"), Token::OpSubtract, Token::LitInteger(1),]
    );
    assert_eq!(tokens("-5"), vec![Token::OpSubtract, Token::LitInteger(5)]);
}

#[test]
fn strings() {
    assert_eq!(
        tokens(r#""Hello, world!""#),
        vec![Token::LitString("Hello, world!")]
    );
    assert_eq!(tokens(r#""""#), vec![Token::LitString("")]);
    assert_eq!(
        tokens(r#""こんにちは世界" "😊""#),
        vec![Token::LitString("こんにちは世界"), Token::LitString("😊"),]
    );
    assert!(matches!(tokens(r#""unterminated"#)[0], Token::Error(_)));
}

#[test]
fn escapes_are_not_supported() {
    // The literal ends at the first `"`, so `\"` does not escape.
    assert_eq!(tokens(r#""a\"b""#)[0], Token::LitString(r"a\"));
}

#[test]
fn identifiers() {
    assert_eq!(
        tokens("add μέσος _x x1 snake_case"),
        vec![
            Token::Ident("add"),
            Token::Ident("μέσος"),
            Token::Ident("_x"),
            Token::Ident("x1"),
            Token::Ident("snake_case"),
        ]
    );
    // Keywords only match whole words.
    assert_eq!(
        tokens("iffy lettuce"),
        vec![Token::Ident("iffy"), Token::Ident("lettuce"),]
    );
    assert_eq!(tokens("_"), vec![Token::SymWildcard]);
}

#[test]
fn keywords_and_type_names() {
    assert_eq!(
        tokens("if then else match with let fn type import export from as"),
        vec![
            Token::KeywordIf,
            Token::KeywordThen,
            Token::KeywordElse,
            Token::KeywordMatch,
            Token::KeywordWith,
            Token::KeywordLet,
            Token::KeywordFn,
            Token::KeywordType,
            Token::KeywordImport,
            Token::KeywordExport,
            Token::KeywordFrom,
            Token::KeywordAs,
        ]
    );
    assert_eq!(
        tokens("int float bool string true false ()"),
        vec![
            Token::TypeInteger,
            Token::TypeFloat,
            Token::TypeBool,
            Token::TypeString,
            Token::LitBool(true),
            Token::LitBool(false),
            Token::LitUnit,
        ]
    );
}

#[test]
fn longest_operator_wins() {
    assert_eq!(
        tokens("= == ! != < <= << > >= >> * ** : :: - ->"),
        vec![
            Token::OpAssign,
            Token::OpEqual,
            Token::OpNegate,
            Token::OpNotEqual,
            Token::OpLessThan,
            Token::OpLessThanOrEqual,
            Token::OpBitwiseShiftLeft,
            Token::OpGreaterThan,
            Token::OpGreaterThanOrEqual,
            Token::OpBitwiseShiftRight,
            Token::OpMultiply,
            Token::OpExponentiate,
            Token::SymColon,
            Token::OpCons,
            Token::OpSubtract,
            Token::SymArrow,
        ]
    );
    assert_eq!(
        tokens("[] [ ] () ( ) ..."),
        vec![
            Token::SymArray,
            Token::SymLBracket,
            Token::SymRBracket,
            Token::LitUnit,
            Token::SymLParen,
            Token::SymRParen,
            Token::SymRest,
        ]
    );
}

#[test]
fn comments_run_to_end_of_line() {
    assert_eq!(
        tokens("// a\n/// b\n1 // c\n//\n///"),
        vec![
            Token::Comment("a"),
            Token::DocComment("b"),
            Token::LitInteger(1),
            Token::Comment("c"),
            Token::Comment(""),
            Token::DocComment(""),
        ]
    );
    // `////` is a doc comment whose contents start with `/`.
    assert_eq!(tokens("//// d"), vec![Token::DocComment("/ d")]);
}

#[test]
fn whitespace_and_newlines() {
    assert_eq!(
        tokens("1 \t 2\r\n3"),
        vec![
            Token::LitInteger(1),
            Token::LitInteger(2),
            Token::LitInteger(3),
        ]
    );
    let newlines = kali_parse::lex_str("1\n\n2")
        .into_iter()
        .filter(|token| token.inner == Token::Newline)
        .count();
    assert_eq!(newlines, 2);
}

#[test]
fn invalid_characters_are_errors() {
    assert!(matches!(tokens("$")[0], Token::Error(_)));
    rejects("let x = $");
}
