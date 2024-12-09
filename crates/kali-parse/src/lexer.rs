//! A lexer for the Kali programming language.

use logos::Logos;

/// An enumeration of possbile tokens that can be lexed from source code.
#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(skip r" ", error = LexerError)]
pub enum Token<'src> {
    #[token("if")]
    KeywordIf,
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
    #[token("=")]
    OpAssign,
    #[token("==")]
    OpEq,
    #[token("+")]
    OpAdd,
    #[token("-")]
    OpSub,
    #[token("*")]
    OpMul,
    #[token("/")]
    OpDiv,
    #[token("%")]
    OpMod,
    #[token("::")]
    OpCons,
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(&'src str),
    // literals
    #[regex("[0-9][0-9_]*", |lex| lex.slice().parse().ok())]
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
    #[token(",")]
    SymComma,
    #[token(":")]
    SymColon,
    #[token("->")]
    SymArrow,
    #[token("|")]
    SymPipe,
    // indentation - ignores trailing whitespace at ends of lines
    #[regex("[\t ]*\n", |_| Indent { length: 0, kind: IndentationKind::Unknown })]
    #[regex("[\t ]*\n\t+", |lex| Indent { length: indent_length(lex), kind: IndentationKind::Tabs })]
    #[regex("[\t ]*\n +", |lex| Indent { length: indent_length(lex), kind: IndentationKind::Spaces })]
    Newline(Indent),
    /// Signals the start of a block.
    BlockStart,
    /// Signals the end of a block.
    BlockEnd,
}

/// Stores the length and kind of an indentation.
#[derive(Debug, Clone, PartialEq)]
pub struct Indent {
    length: usize,
    kind: IndentationKind,
}

/// An enumeration of possible kinds of indentation.
#[derive(Debug, Clone, PartialEq)]
pub enum IndentationKind {
    Spaces,
    Tabs,
    Unknown,
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

fn indent_length<'src>(lex: &mut logos::Lexer<'src, Token<'src>>) -> usize {
    // ignore any leading whitespace
    lex.slice()
        .chars()
        .skip_while(|c| *c == ' ' || *c == '\t')
        .skip(1)
        .count()
}

/// A lexer capable of denesting blocks using indentation by replacing block delimiters with parentheses.
pub struct DenestedLexer<'src> {
    lexer: logos::Lexer<'src, Token<'src>>,
    level: usize,
    kind: IndentationKind,
    size: usize,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexerError {
    #[default]
    Unknown,
    BadIndentationCharacter {
        expected: IndentationKind,
        found: IndentationKind,
    },
    BadIndentationSize {
        expected: usize,
        actual: usize,
    },
}

impl<'src> DenestedLexer<'src> {
    /// Creates a new denested lexer from the given source code.
    pub fn new(src: &'src str) -> Self {
        Self {
            lexer: Token::lexer(src),
            level: 0,
            size: 0,
            kind: IndentationKind::Unknown,
        }
    }

    /// Returns the next token from the lexer.
    pub fn next(&mut self) -> Option<Result<Token<'src>, LexerError>> {
        // indenting
        let indent = match self.lexer.next()? {
            Ok(Token::Newline(indent)) => indent,
            Ok(token) => return Some(Ok(token)),
            Err(e) => return Some(Err(e)),
        };

        // infer indentation kind if unknown
        if self.kind == IndentationKind::Unknown {
            self.kind = indent.kind;
            self.size = indent.length;
        } else if self.kind != indent.kind {
            return Some(Err(LexerError::BadIndentationCharacter {
                expected: self.kind.clone(),
                found: indent.kind,
            }));
        }

        // depth = length / size
        if indent.length % self.size != 0 {
            return Some(Err(LexerError::BadIndentationSize {
                expected: self.size,
                actual: indent.length % self.size,
            }));
        }

        let depth = indent.length / self.size;

        // if depth is greater than the current level, emit a block start token
        if depth > self.level {
            self.level = depth;
            return Some(Ok(Token::BlockStart));
        }

        // if depth is less than the current level, emit a block end token
        if depth < self.level {
            self.level = depth;
            return Some(Ok(Token::BlockEnd));
        }

        // if depth is equal to the current level, emit the next token
        self.lexer.next()
    }
}

impl<'src> Iterator for DenestedLexer<'src> {
    type Item = Result<Token<'src>, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::lexer::Token;

    use super::DenestedLexer;

    #[test]
    fn natural() {
        // 1234
        assert_eq!(
            Token::lexer("1234").next(),
            Some(Ok(Token::LitNatural(1234)))
        );
        // 0xdead_beef
        assert_eq!(
            Token::lexer("0xdead_beef").next(),
            Some(Ok(Token::LitNatural(0xdead_beef)))
        );
    }

    #[test]
    fn integer() {
        // -1234
        assert_eq!(
            Token::lexer("-1234").next(),
            Some(Ok(Token::LitInteger(-1234)))
        );
        // -0xdead_beef
        assert_eq!(
            Token::lexer("-0xdead_beef").next(),
            Some(Ok(Token::LitInteger(-0xdead_beef)))
        );
    }

    #[test]
    fn string() {
        // "hello, world!"
        assert_eq!(
            Token::lexer("\"hello, world!\"").next(),
            Some(Ok(Token::LitString("hello, world!")))
        );

        // "\t\t\n\t\n"
        assert_eq!(
            Token::lexer(r#""\t\t\n\t\n""#).next(),
            Some(Ok(Token::LitString("\\t\\t\\n\\t\\n")))
        );
    }

    #[test]
    fn block() {
        let src = include_str!("../tests/lexer/fib.kali");
        let tokens: Vec<_> = DenestedLexer::new(src).collect();
        println!("{:?}", tokens);
    }
}
