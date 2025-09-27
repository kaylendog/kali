//! Printing and formatting support.

use std::io::Write;

use kali_ast::{
    BinaryExpr, Call, Conditional, Expr, Identifier, Lambda, Literal, LiteralKind, Match, Module,
    UnaryExpr,
};

/// `Context` holds a mutable reference to a writer implementing `std::io::Write`.
/// It is used to manage output buffers for printing operations.
pub struct Context<'a> {
    /// The current indent depth.
    depth: usize,
    /// The buffer to which output will be written.
    buf: &'a mut (dyn Write + 'a),
}

impl<'a> Context<'a> {
    /// Creates a new `Context` with the given writer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable reference to a type implementing `std::io::Write`.
    pub fn new(buf: &mut dyn Write) -> Context<'_> {
        Context { depth: 0, buf }
    }

    /// Increases the current indentation depth by one.
    pub fn increase(&mut self) {
        self.depth += 1;
    }

    /// Decreases the current indentation depth by one.
    pub fn decrease(&mut self) {
        self.depth -= 1;
    }

    /// Writes a newline followed by indentation corresponding to the current depth.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if writing to the buffer fails.
    pub fn newline(&mut self) -> Result<()> {
        write!(self, "\n")?;
        for _ in 0..self.depth {
            write!(self, "\t")?;
        }
        Ok(())
    }
}

impl Write for Context<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.flush()
    }
}

/// A specialized `Result` type for printing operations, using the custom `Error` type.
type Result<T> = std::result::Result<T, Error>;

/// Error type for printing operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Trait for types that can be printed to a writer via a `Context`.
pub trait Print {
    /// Prints the value to the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - A mutable reference to a `Context` containing the output buffer.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if printing fails.
    fn print(&self, ctx: &mut Context) -> Result<()>;
}

impl<Meta> Print for Module<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        todo!()
    }
}

impl<Meta> Print for Expr<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        match &self {
            Expr::Literal(literal) => literal.print(ctx),
            Expr::Ident(identifier) => identifier.print(ctx),
            Expr::BinaryExpr(binary_expr) => binary_expr.print(ctx),
            Expr::UnaryExpr(unary_expr) => unary_expr.print(ctx),
            Expr::Conditional(conditional) => conditional.print(ctx),
            Expr::Lambda(lambda) => lambda.print(ctx),
            Expr::Match(match_expr) => match_expr.print(ctx),
            Expr::Call(call) => call.print(ctx),
        }
    }
}

impl<Meta> Print for Literal<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        match &self.kind {
            LiteralKind::Natural(x) => write!(ctx, "{}", x)?,
            LiteralKind::Integer(x) => write!(ctx, "{}", x)?,
            LiteralKind::Float(x) => write!(ctx, "{}", x)?,
            LiteralKind::Bool(x) => write!(ctx, "{}", x)?,
            LiteralKind::String(x) => write!(ctx, r#""{}""#, x)?,
            LiteralKind::Unit => write!(ctx, "()")?,
            LiteralKind::Array(exprs) => {
                write!(ctx, "[")?;
                for (i, expr) in exprs.iter().enumerate() {
                    expr.print(ctx)?;
                    if i != exprs.len() - 1 {
                        write!(ctx, ", ")?;
                    }
                }
                write!(ctx, "]")?;
            }
            LiteralKind::Tuple(exprs) => {
                write!(ctx, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    expr.print(ctx)?;
                    if i != exprs.len() - 1 {
                        write!(ctx, ", ")?;
                    }
                }
                write!(ctx, ")")?;
            }
            LiteralKind::Struct(btree_map) => {
                todo!()
            }
        };
        Ok(())
    }
}

impl<Meta> Print for Identifier<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        write!(ctx, "{}", self.value)?;
        Ok(())
    }
}

impl<Meta> Print for BinaryExpr<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        self.lhs.print(ctx)?;
        write!(ctx, " {} ", self.operator)?;
        self.rhs.print(ctx)
    }
}

impl<Meta> Print for UnaryExpr<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        write!(ctx, "{}", self.operator)?;
        self.inner.print(ctx)
    }
}

impl<Meta> Print for Conditional<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        write!(ctx, "if ")?;
        self.condition.print(ctx)?;
        write!(ctx, " ")?;
        self.body.print(ctx)?;
        write!(ctx, " else ")?;
        self.otherwise.print(ctx)?;
        Ok(())
    }
}

impl<Meta> Print for Lambda<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        todo!()
    }
}

impl<Meta> Print for Match<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl<Meta> Print for Call<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kali_ast::{Identifier, LiteralKind};

    struct TestWriter {
        pub output: Vec<u8>,
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn print_to_string<T: Print>(value: &T) -> String {
        let mut writer = TestWriter { output: Vec::new() };
        let mut ctx = Context::new(&mut writer);
        value.print(&mut ctx).unwrap();
        String::from_utf8(writer.output).unwrap()
    }

    #[test]
    fn test_print_literal_natural() {
        let lit = Literal {
            kind: LiteralKind::Natural(42),
            meta: (),
        };
        assert_eq!(print_to_string(&lit), "42");
    }

    #[test]
    fn test_print_literal_integer() {
        let lit = Literal {
            kind: LiteralKind::Integer(-7),
            meta: (),
        };
        assert_eq!(print_to_string(&lit), "-7");
    }

    #[test]
    fn test_print_literal_float() {
        let lit = Literal {
            kind: LiteralKind::Float(3.14),
            meta: (),
        };
        assert_eq!(print_to_string(&lit), "3.14");
    }

    #[test]
    fn test_print_literal_bool() {
        let lit_true = Literal {
            kind: LiteralKind::Bool(true),
            meta: (),
        };
        let lit_false = Literal {
            kind: LiteralKind::Bool(false),
            meta: (),
        };
        assert_eq!(print_to_string(&lit_true), "true");
        assert_eq!(print_to_string(&lit_false), "false");
    }

    #[test]
    fn test_print_literal_string() {
        let lit = Literal {
            kind: LiteralKind::String("hello".to_string()),
            meta: (),
        };
        assert_eq!(print_to_string(&lit), r#""hello""#);
    }

    #[test]
    fn test_print_literal_unit() {
        let lit = Literal {
            kind: LiteralKind::Unit,
            meta: (),
        };
        assert_eq!(print_to_string(&lit), "()");
    }

    #[test]
    fn test_print_identifier() {
        let ident = Identifier {
            value: "foo".to_string(),
            meta: (),
        };
        assert_eq!(print_to_string(&ident), "foo");
    }

    #[test]
    fn test_print_conditional() {
        let cond = Conditional {
            condition: Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Bool(true),
                meta: (),
            })),
            body: Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Natural(1),
                meta: (),
            })),
            otherwise: Box::new(Expr::Literal(Literal {
                kind: LiteralKind::Natural(0),
                meta: (),
            })),
            meta: (),
        };
        assert_eq!(print_to_string(&cond), "if true 1 else 0");
    }

    #[test]
    fn test_print_literal_array() {
        let arr = Literal {
            kind: LiteralKind::Array(vec![
                Expr::Literal(Literal {
                    kind: LiteralKind::Natural(1),
                    meta: (),
                }),
                Expr::Literal(Literal {
                    kind: LiteralKind::Natural(2),
                    meta: (),
                }),
            ]),
            meta: (),
        };
        assert_eq!(print_to_string(&arr), "[1, 2]");
    }

    #[test]
    fn test_print_literal_tuple() {
        let tup = Literal {
            kind: LiteralKind::Tuple(vec![
                Expr::Literal(Literal {
                    kind: LiteralKind::Natural(1),
                    meta: (),
                }),
                Expr::Literal(Literal {
                    kind: LiteralKind::Natural(2),
                    meta: (),
                }),
            ]),
            meta: (),
        };
        assert_eq!(print_to_string(&tup), "(1, 2)");
    }
}
