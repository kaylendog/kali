//! A lexer for the Kali programming language.

use std::{iter::Peekable, str::Chars};

use tracing::trace;

/// An enumeration of possbile tokens that can be lexed from source code.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    /// A symbol.
    Symbol(Symbol),
    /// An operator.
    Operator(Operator),
    /// A numeric literal.
    Numeric(Numeric),
    /// A keyword.
    Keyword(Keyword),
    /// An identifier.
    Identifier(&'src str),
    /// A string literal.
    String { contents: &'src str },
    /// The start of a block.
    BlockStart,
    /// The end of a block.
    BlockEnd,
}

// A numeric literal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Numeric {
    /// A natural number.
    Natural(u64),
    /// An integer number.
    Integer(i64),
    /// A floating-point number.
    Real(f64),
}

/// A source code symbol. These are distinct from operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Pipe,
    Arrow,
    Colon,
}

/// An enumeration of operator tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

/// A keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    If,
    Else,
    Match,
    Let,
    Fn,
    With,
}

/// A lexer for the Kali programming language.
#[derive(Debug)]
pub struct Lexer<'src> {
    input: &'src str,
    chars: Peekable<Chars<'src>>,
    /// The current position in the input, in bytes.
    position: usize,
    /// The current indentation level.
    indent: usize,
    /// Whether the current indentation is spaces.
    indent_space: bool,
    /// The number of spaces in the current indentation.
    indent_spaces_count: usize,
    /// Backtrack counter.
    backtrack: usize,
}

impl<'src> Lexer<'src> {
    /// Creates a new lexer from the given input.
    pub fn new(input: &'src str) -> Self {
        Self {
            chars: input.chars().peekable(),
            input,
            position: 0,
            indent: 0,
            indent_space: false,
            indent_spaces_count: 0,
            backtrack: 0,
        }
    }

    /// Eats the next character from the input if it satisfies the given predicate, and returns whether a character was eaten.
    #[inline]
    fn eat_if<F>(&mut self, f: F) -> bool
    where
        F: Fn(&char) -> bool,
    {
        match self.chars.peek() {
            Some(c) if f(c) => {
                self.position += c.len_utf8();
                self.chars.next();
                true
            }
            _ => false,
        }
    }

    /// Eats characters from the input while the given predicate is true.
    #[inline]
    fn eat_while<F>(&mut self, f: F)
    where
        F: Fn(&char) -> bool,
    {
        while let Some(c) = self.chars.peek() {
            if f(c) {
                self.position += c.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }
    }

    /// Eats the given character from the input.
    #[inline]
    fn eat_char(&mut self, c: char) -> bool {
        self.eat_if(|x| *x == c)
    }

    /// Eats the given string from the input.
    #[inline]
    fn eat_literal(&mut self, string: &str) -> bool {
        let mut chars = string.chars();
        let start = self.position;
        while let Some(c) = chars.next() {
            if self.chars.peek() != Some(&c) {
                self.restore(start);
                return false;
            }
            self.position += c.len_utf8();
            self.chars.next();
        }
        true
    }

    /// Eats the given character n times from the input.
    #[inline]
    fn eat_exactly_n(&mut self, c: char, n: usize) -> bool {
        let start = self.position;
        for _ in 0..n {
            if !self.eat_char(c) {
                self.restore(start);
                return false;
            }
        }
        true
    }

    /// Eats inline whitespace from the input.
    #[inline]
    fn eat_inline_whitespace(&mut self) {
        self.eat_while(|c| c.is_whitespace() && *c != '\n' || *c == '\r');
    }

    /// Eats characters from the input while the given predicate is true, then returns a slice over the eaten characters.
    #[inline]
    fn emit_while<F>(&mut self, f: F) -> Option<&'src str>
    where
        F: Fn(char) -> bool,
    {
        let start = self.position;
        self.eat_while(|c| f(*c));
        if self.position == start {
            None
        } else {
            Some(&self.input[start..self.position])
        }
    }

    /// Restore the lexer to a previous position.
    #[inline]
    fn restore(&mut self, pos: usize) {
        self.backtrack += 1;
        self.chars = self.input[self.position..].chars().peekable();
        self.position = pos;
    }

    /// Try a number of choices, returning the first successful one.
    ///
    /// TODO: smarter backtracking - if we try "abc" and "abe", we don't need to reset to try the next character. Probably needs compile-time macro shenanigans.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn choose_literal(&mut self, choices: &[&'src str]) -> Option<&'src str> {
        let start = self.position;
        for choice in choices {
            trace!(choice = choice, "trying");
            if self.eat_literal(choice) {
                trace!(choice = choice, "success");
                return Some(choice);
            }
            trace!(choice = choice, "failed");
            self.restore(start);
        }
        None
    }

    /// Consumes an identifier from the input.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn identifier(&mut self) -> Option<&'src str> {
        let start = self.position;
        self.eat_if(|c| unicode_ident::is_xid_start(*c))
            .then(|| self.eat_while(|c| unicode_ident::is_xid_continue(*c)))
            .map(|_| &self.input[start..self.position])
            .or_else(|| {
                self.restore(start);
                None
            })
    }

    /// Consumes a keyword from the input.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn keyword(&mut self) -> Option<Keyword> {
        match self.choose_literal(&["if", "else", "match", "let", "fn", "with"]) {
            Some("if") => Some(Keyword::If),
            Some("else") => Some(Keyword::Else),
            Some("match") => Some(Keyword::Match),
            Some("let") => Some(Keyword::Let),
            Some("fn") => Some(Keyword::Fn),
            Some("with") => Some(Keyword::With),
            _ => None,
        }
    }

    /// Consumes a symbol from the input.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn symbol(&mut self) -> Option<Symbol> {
        match self.choose_literal(&["(", ")", "[", "]", "{", "}", "|", "->", ":"]) {
            Some("(") => Some(Symbol::LeftParen),
            Some(")") => Some(Symbol::RightParen),
            Some("[") => Some(Symbol::LeftBracket),
            Some("]") => Some(Symbol::RightBracket),
            Some("{") => Some(Symbol::LeftBrace),
            Some("}") => Some(Symbol::RightBrace),
            Some("|") => Some(Symbol::Pipe),
            Some("->") => Some(Symbol::Arrow),
            Some(":") => Some(Symbol::Colon),
            _ => None,
        }
    }

    /// Consumes an operator from the input.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn operator(&mut self) -> Option<Operator> {
        let operator = match self.choose_literal(&["+", "-", "*", "/"]) {
            Some("+") => Operator::Add,
            Some("-") => Operator::Sub,
            Some("*") => Operator::Mul,
            Some("/") => Operator::Div,
            _ => return None,
        };
        Some(operator)
    }

    /// Starts a new block.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn start_block(&mut self) -> Option<Token<'src>> {
        let start = self.position;
        // eat opening new-line
        if let None = self.choose_literal(&["\n", "\r\n"]) {
            self.restore(start);
            return None;
        }
        // if indentation is 0, we need to determine if it's spaces or tabs - if neither, then this is just a regular new-line
        if self.indent == 0 {
            self.emit_while(|c| c == ' ')
                .map(|s| {
                    self.indent_space = true;
                    self.indent_spaces_count = s.len();
                })
                .or_else(|| self.emit_while(|c| c == '\t').map(|_| ()));
            return Some(Token::BlockStart);
        }
        // otherwise, attempt to eat the correct indentation for a new block -
        self.indent += 1;

        let correct = match self.indent_space {
            true => self.eat_exactly_n(' ', self.indent_spaces_count * self.indent),
            false => self.eat_exactly_n('\t', self.indent),
        };

        if !correct {
            self.restore(start);
            return None;
        }

        trace!(indent = self.indent, "starting block");

        return Some(Token::BlockStart);
    }

    /// Ends a block.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn end_block(&mut self) -> Option<Token<'src>> {
        let start = self.position;
        // eat closing new-line
        if let None = self.choose_literal(&["\n", "\r\n"]) {
            self.restore(start);
            return None;
        }
        // if the indentation is 0, then this is just a regular new-line
        if self.indent == 0 {
            return None;
        }
        // we expect the indentation to be one level less than the current level
        let correct = match self.indent_space {
            true => self.eat_exactly_n(' ', self.indent_spaces_count * (self.indent - 1)),
            false => self.eat_exactly_n('\t', self.indent - 1),
        };
        if !correct {
            self.restore(start);
            return None;
        }
        self.indent -= 1;

        trace!(indent = self.indent, "ending block");

        Some(Token::BlockEnd)
    }

    /// Parses the next token from the input.
    #[inline]
    #[tracing::instrument(skip(self))]
    fn token(&mut self) -> Option<Token<'src>> {
        // eat preceeding whitespace
        self.eat_inline_whitespace();

        self.end_block()
            .or_else(|| self.start_block())
            .or_else(|| self.keyword().map(Token::Keyword))
            .or_else(|| self.symbol().map(Token::Symbol))
            .or_else(|| self.operator().map(Token::Operator))
            .or_else(|| self.identifier().map(Token::Identifier))
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use super::*;

    #[test]
    fn identifier() {
        let mut lexer = Lexer::new("hello world");
        assert_eq!(lexer.token(), Some(Token::Identifier("hello")));
    }

    #[test]
    fn keyword() {
        let mut lexer = Lexer::new("if else match let fn with");
        assert_eq!(lexer.token(), Some(Token::Keyword(Keyword::If)));
        assert_eq!(lexer.token(), Some(Token::Keyword(Keyword::Else)));
        assert_eq!(lexer.token(), Some(Token::Keyword(Keyword::Match)));
        assert_eq!(lexer.token(), Some(Token::Keyword(Keyword::Let)));
        assert_eq!(lexer.token(), Some(Token::Keyword(Keyword::Fn)));
        assert_eq!(lexer.token(), Some(Token::Keyword(Keyword::With)));
    }

    #[test]
    fn symbol() {
        let mut lexer = Lexer::new("()[]{}|:");
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::LeftParen)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::RightParen)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::LeftBracket)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::RightBracket)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::LeftBrace)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::RightBrace)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::Pipe)));
        assert_eq!(lexer.token(), Some(Token::Symbol(Symbol::Colon)));
    }

    #[test]
    fn operator() {
        let mut lexer = Lexer::new("+-*/");
        assert_eq!(lexer.token(), Some(Token::Operator(Operator::Add)));
        assert_eq!(lexer.token(), Some(Token::Operator(Operator::Sub)));
        assert_eq!(lexer.token(), Some(Token::Operator(Operator::Mul)));
        assert_eq!(lexer.token(), Some(Token::Operator(Operator::Div)));
    }

    #[test]
    fn block() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();

        let mut lexer = Lexer::new("hello\n\tworld\n");

        assert_eq!(lexer.token(), Some(Token::Identifier("hello")));
        assert_eq!(lexer.token(), Some(Token::BlockStart));
        assert_eq!(lexer.token(), Some(Token::Identifier("world")));
        assert_eq!(lexer.token(), Some(Token::BlockEnd));
    }
}
