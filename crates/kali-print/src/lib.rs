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
            LiteralKind::Array(exprs) => todo!(),
            LiteralKind::Tuple(exprs) => todo!(),
            LiteralKind::Struct(btree_map) => todo!(),
        };
        Ok(())
    }
}

impl<Meta> Print for Identifier<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        todo!()
    }
}

impl<Meta> Print for BinaryExpr<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl<Meta> Print for UnaryExpr<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl<Meta> Print for Conditional<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl<Meta> Print for Lambda<Meta> {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        Ok(())
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
