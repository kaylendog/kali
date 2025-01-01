//! A lexer for the Kali programming language.

use kali_ast::Span;
use logos::Logos;

/// An enumeration of possbile tokens that can be lexed from source code.
#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(skip r" ", error = LexerError)]
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
    #[token("=")]
    OpAssign,
    #[token("==")]
    OpEq,
    #[token("!=")]
    OpNe,
    #[token("<=")]
    OpLe,
    #[token(">=")]
    OpGe,
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
    #[token("**")]
    OpPow,
    #[token("::")]
    OpCons,
    #[token("!")]
    OpNeg,
    #[token("~")]
    OpBitNot,
    #[token("&&")]
    OpAnd,
    #[token("||")]
    OpOr,
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
    #[token("|")]
    SymPipe,
    #[token("[]")]
    SymArray,
    #[token("<")]
    SymLAngle,
    #[token(">")]
    SymRAngle,
    // indentation - ignores trailing whitespace at ends of lines
    #[regex("[\n\t ]*\n", |_| Indent { length: 0, kind: IndentationKind::Unknown })]
    #[regex("[\n\t ]*\n\t+", |lex| Indent { length: indent_length(lex), kind: IndentationKind::Tabs })]
    #[regex("[\n\t ]*\n +", |lex| Indent { length: indent_length(lex), kind: IndentationKind::Spaces })]
    Newline(Indent),
    /// Signals the start of a block.
    BlockStart,
    /// Signals the end of a block.
    BlockEnd,

    #[regex("[\n\t ]*#.*", logos::skip)]
    Ignored,
}

/// A token paired with its span.
pub type SpannedToken<'src> = (Result<Token<'src>, LexerError>, Span);

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
    let last_newline = lex.slice().rfind('\n').unwrap();
    lex.slice()[last_newline..].len() - 1
}

/// A lexer capable of denesting blocks using indentation by replacing block delimiters with parentheses.
pub struct IndentLexer<'src> {
    lexer: logos::SpannedIter<'src, Token<'src>>,
    level: usize,
    kind: IndentationKind,
    size: usize,
    unindents: usize,
}

/// An enumeration of possible errors that can occur during lexing.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexerError {
    /// An unknown error occurred.
    #[default]
    Unknown,
    /// A bad type of indentation character was found.
    BadIndentationCharacter {
        expected: IndentationKind,
        found: IndentationKind,
    },
    /// A bad size of indentation was found.
    BadIndentationSize { expected: usize, actual: usize },
}

impl<'src> IndentLexer<'src> {
    /// Creates a new denested lexer from the given source code.
    pub fn new(src: &'src str) -> Self {
        Self {
            lexer: Token::lexer(src).spanned(),
            level: 0,
            size: 0,
            kind: IndentationKind::Unknown,
            unindents: 0,
        }
    }

    /// Returns the next token from the lexer.
    pub fn next(&mut self) -> Option<SpannedToken<'src>> {
        if self.unindents > 0 {
            self.unindents -= 1;
            return Some((
                Ok(Token::BlockEnd),
                Span::new(self.lexer.span().start, self.lexer.span().start),
            ));
        }

        let (indent, span) = match self.lexer.next()? {
            (Ok(Token::Newline(indent)), span) => (indent, Span::new(span.start, span.end)),
            (Ok(token), span) => return Some((Ok(token), Span::new(span.start, span.end))),
            (Err(e), span) => return Some((Err(e), Span::new(span.start, span.end))),
        };

        // infer indentation kind if unknown
        if self.kind == IndentationKind::Unknown {
            self.kind = indent.kind;
            self.size = indent.length;
        } else if self.kind != indent.kind {
            // indent kind is only unknown if it's de-nesting of one or more blocks
            if indent.kind == IndentationKind::Unknown {
                self.unindents = self.level - 1;
                self.level = 0;
                return Some((Ok(Token::BlockEnd), span));
            }

            // otherwise, the user is using the wrong kind of indentation
            return Some((
                Err(LexerError::BadIndentationCharacter {
                    expected: self.kind.clone(),
                    found: indent.kind,
                }),
                span,
            ));
        }

        // if the indent length is 0, then this is just a newline - emit next
        if indent.length == 0 {
            return self
                .lexer
                .next()
                .map(|(result, span)| (result, Span::new(span.start, span.end)));
        }

        // depth = length / size
        if indent.length % self.size != 0 {
            return Some((
                Err(LexerError::BadIndentationSize {
                    expected: self.size,
                    actual: indent.length % self.size,
                }),
                span,
            ));
        }

        let depth = indent.length / self.size;

        // if depth is greater than the current level, emit a block start token
        if depth > self.level {
            self.level = depth;
            return Some((Ok(Token::BlockStart), span));
        }

        // if depth is less than the current level, emit a block end token
        if depth < self.level {
            self.level = depth;
            return Some((Ok(Token::BlockEnd), span));
        }

        // if depth is equal to the current level, emit the next token
        self.lexer
            .next()
            .map(|(result, span)| (result, Span::new(span.start, span.end)))
    }
}

impl<'src> Iterator for IndentLexer<'src> {
    type Item = SpannedToken<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// Lexes the given source code into a vector of tokens.
pub fn lex_to_vec<'src>(src: &'src str) -> Vec<SpannedToken<'src>> {
    IndentLexer::new(src).collect()
}

/// Unwraps the result of the lexer into a vector of tokens.
pub fn unwrap_to_vec<'src>(src: &'src str) -> Vec<(Token<'src>, Span)> {
    IndentLexer::new(src)
        .map(|(result, span)| (result.unwrap(), span))
        .collect()
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::{IndentLexer, Token};

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
        let src = include_str!("../tests/lexer.kali");
        let tokens = IndentLexer::new(src)
            .map(|(token, _)| token)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        println!("{:?}", tokens);
    }

    #[test]
    fn ident() {
        // hello
        assert_eq!(
            Token::lexer("hello").next(),
            Some(Ok(Token::Ident("hello")))
        );
        // _world
        assert_eq!(
            Token::lexer("_world").next(),
            Some(Ok(Token::Ident("_world")))
        );
        // hello_world
        assert_eq!(
            Token::lexer("hello_world").next(),
            Some(Ok(Token::Ident("hello_world")))
        );

        // 你好
        assert_eq!(Token::lexer("你好").next(), Some(Ok(Token::Ident("你好"))));

        // привет
        assert_eq!(
            Token::lexer("привет").next(),
            Some(Ok(Token::Ident("привет")))
        );
    }
}
