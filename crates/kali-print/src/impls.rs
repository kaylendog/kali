use std::{
    collections::BTreeMap,
    io::{self, Write},
};

use kali_ast::{
    BinaryOpKind, Comment, CommentKind, Definition, Destructor, DestructorKind, Expr, ExprKind,
    Ident, ImportTree, ImportTreeKind, Item, ItemKind, LambdaParam, LiteralKind, MatchArm, Module,
    NodeId, Pattern, PatternKind, PrimitiveTypeKind, Type, TypeAlias, TypeDecl, TypeKind,
    Visibility,
};

use crate::{error::Result, pass::Comments};

/// Output buffer plus the state needed to print a module: indentation, the string interner, and
/// the comments attached to each node.
pub struct Context<'a> {
    /// The current indent depth.
    depth: usize,
    /// The buffer to which output will be written.
    buf: &'a mut (dyn Write + 'a),
    /// Interner for identifiers, strings and comment contents.
    cache: &'a lasso::Rodeo,
    /// Original source, used to preserve blank lines between items and comments.
    src: &'a str,
    /// Comments attached to nodes. Removed as they are printed.
    pub(crate) comments: BTreeMap<NodeId, Comments>,
}

impl<'a> Context<'a> {
    pub fn new(buf: &'a mut dyn Write, cache: &'a lasso::Rodeo, src: &'a str) -> Context<'a> {
        Context {
            depth: 0,
            buf,
            cache,
            src,
            comments: BTreeMap::new(),
        }
    }

    /// Writes the separator between two things that were at `from..to` in the source: a space on
    /// the same line, a newline, or one blank line for two or more. `at_least` forces a minimum
    /// number of newlines regardless of the source.
    fn gap(&mut self, from: usize, to: usize, at_least: usize) -> io::Result<()> {
        let newlines = self
            .src
            .get(from..to)
            .map_or(1, |text| text.matches('\n').count());
        match newlines.max(at_least) {
            0 => write!(self, " "),
            1 => self.newline(),
            _ => {
                writeln!(self)?;
                self.newline()
            }
        }
    }

    /// Resolves an interned string. Borrows the interner, not the context.
    fn resolve(&self, key: &lasso::Spur) -> &'a str {
        self.cache.resolve(key)
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
    pub fn newline(&mut self) -> io::Result<()> {
        writeln!(self)?;
        for _ in 0..self.depth {
            write!(self, "\t")?;
        }
        Ok(())
    }

    /// Writes `items` between `open` and `close`, with `sep` between consecutive items.
    pub fn write_many<T>(
        &mut self,
        open: &str,
        close: &str,
        sep: &str,
        items: impl IntoIterator<Item = T>,
        f: impl Fn(&mut Context, T) -> Result<()>,
    ) -> Result<()> {
        write!(self, "{open}")?;
        for (index, item) in items.into_iter().enumerate() {
            if index > 0 {
                write!(self, "{sep}")?;
            }
            f(self, item)?;
        }
        write!(self, "{close}")?;
        Ok(())
    }
}

impl Write for Context<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.flush()
    }
}

/// Trait for types that can be printed to a writer via a `Context`.
pub trait Print {
    fn print(&self, ctx: &mut Context) -> Result<()>;
}

// --- Precedence ---
//
// Binding powers mirror the pratt tables in the parser. A child is parenthesised when its
// binding power is below the minimum its position requires.

#[derive(Clone, Copy)]
enum Assoc {
    Left,
    Right,
}

/// Binding power of nodes that are never split by an operator.
const ATOM: u8 = u8::MAX;
const UNARY: u8 = 11;
const CALL: u8 = 10;
const LAMBDA: u8 = 1;

fn binary_op(kind: BinaryOpKind) -> (u8, Assoc) {
    use BinaryOpKind::*;
    match kind {
        Exponentiate => (9, Assoc::Right),
        // Bitwise and concat operators are not parsed yet; placed beside their nearest cousins.
        Multiply | Divide | Modulo | BitwiseShiftLeft | BitwiseShiftRight => (8, Assoc::Left),
        Add | Subtract => (7, Assoc::Left),
        LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => (6, Assoc::Left),
        Equal | NotEqual => (5, Assoc::Left),
        LogicalAnd | BitwiseAnd | BitwiseXor => (4, Assoc::Left),
        LogicalOr | BitwiseOr => (3, Assoc::Left),
        Cons | Concat => (2, Assoc::Right),
    }
}

/// Minimum binding powers for the operands of an infix operator.
fn operand_minimums(precedence: u8, assoc: Assoc) -> (u8, u8) {
    // Saturating because atoms sit at `u8::MAX` and never have operands.
    match assoc {
        Assoc::Left => (precedence, precedence.saturating_add(1)),
        Assoc::Right => (precedence.saturating_add(1), precedence),
    }
}

fn expr_precedence(expr: &Expr) -> u8 {
    match &expr.kind {
        ExprKind::BinaryExpr { op, .. } => binary_op(op.kind).0,
        ExprKind::UnaryExpr { .. } => UNARY,
        ExprKind::Call { .. } => CALL,
        ExprKind::Lambda { .. } => LAMBDA,
        _ => ATOM,
    }
}

fn pattern_precedence(pattern: &Pattern) -> (u8, Assoc) {
    match &pattern.kind {
        PatternKind::Cons { .. } => (1, Assoc::Right),
        PatternKind::Or { .. } => (2, Assoc::Left),
        _ => (ATOM, Assoc::Left),
    }
}

/// Prints `child`, parenthesised if its binding power is below `minimum`.
fn print_child<T: Print>(ctx: &mut Context, child: &T, precedence: u8, minimum: u8) -> Result<()> {
    if precedence < minimum {
        write!(ctx, "(")?;
        child.print(ctx)?;
        write!(ctx, ")")?;
    } else {
        child.print(ctx)?;
    }
    Ok(())
}

// --- Module and items ---

impl Print for Module {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        // Source offset where the last printed thing ended. `None` until something is printed,
        // so blank lines at the top of the file are dropped.
        let mut last_end: Option<usize> = None;
        let mut print_at =
            |ctx: &mut Context, node: &dyn Print, start, end, at_least| -> Result<()> {
                if let Some(from) = last_end {
                    ctx.gap(from, start, at_least)?;
                }
                node.print(ctx)?;
                last_end = Some(end);
                Ok(())
            };

        for item in &self.items {
            let comments = ctx.comments.remove(&item.id).unwrap_or_default();
            for comment in &comments.leading {
                print_at(ctx, comment, comment.span.start, comment.span.end, 0)?;
            }
            // Line ends separate items, so an item always starts a new line.
            print_at(ctx, item, item.span.start, item.span.end, 1)?;
            // Dangling comments have no slot of their own yet, so they follow the item.
            for comment in &comments.dangling {
                ctx.newline()?;
                comment.print(ctx)?;
            }
            for comment in &comments.trailing {
                print_at(ctx, comment, comment.span.start, comment.span.end, 0)?;
            }
        }
        if last_end.is_some() {
            ctx.newline()?;
        }
        Ok(())
    }
}

impl Print for Comment {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        let marker = match self.kind {
            CommentKind::Single => "//",
            CommentKind::Doc => "///",
        };
        let contents = ctx.resolve(&self.contents);
        if contents.is_empty() {
            write!(ctx, "{marker}")?;
        } else {
            write!(ctx, "{marker} {contents}")?;
        }
        Ok(())
    }
}

impl Print for Ident {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        let name = ctx.resolve(&self.key);
        Ok(write!(ctx, "{name}")?)
    }
}

impl Print for Item {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        if self.visibility == Visibility::Exported {
            write!(ctx, "export ")?;
        }
        match &self.kind {
            ItemKind::Definition(definition) => definition.print(ctx),
            ItemKind::TypeAlias(alias) => alias.print(ctx),
            ItemKind::TypeDecl(decl) => decl.print(ctx),
            ItemKind::Import(import) => {
                write!(ctx, "import ")?;
                import.print(ctx)
            }
        }
    }
}

impl Print for Definition {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        write!(ctx, "let ")?;
        self.name.print(ctx)?;
        write!(ctx, " = ")?;
        self.expr.print(ctx)
    }
}

impl Print for TypeAlias {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        write!(ctx, "type ")?;
        self.name.print(ctx)?;
        write!(ctx, " = ")?;
        self.ty.print(ctx)
    }
}

impl Print for ImportTree {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        match &self.kind {
            ImportTreeKind::Item { name, alias } => {
                name.print(ctx)?;
                if let Some(alias) = alias {
                    write!(ctx, " as ")?;
                    alias.print(ctx)?;
                }
                Ok(())
            }
            ImportTreeKind::Segment { name, child } => {
                name.print(ctx)?;
                write!(ctx, " :: ")?;
                child.print(ctx)
            }
            ImportTreeKind::Glob => Ok(write!(ctx, "*")?),
            ImportTreeKind::List(trees) => {
                ctx.write_many("{ ", " }", ", ", trees, |ctx, tree| tree.print(ctx))
            }
        }
    }
}

// --- Types ---

impl Print for Type {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        match &self.kind {
            TypeKind::Primitive(primitive) => {
                let name = match primitive {
                    PrimitiveTypeKind::Integer => "int",
                    PrimitiveTypeKind::Float => "float",
                    PrimitiveTypeKind::Bool => "bool",
                    PrimitiveTypeKind::String => "string",
                    PrimitiveTypeKind::Unit => "()",
                };
                Ok(write!(ctx, "{name}")?)
            }
            TypeKind::Named(name) => name.print(ctx),
            TypeKind::Tuple(types) => {
                ctx.write_many("(", ")", ", ", types, |ctx, ty| ty.print(ctx))
            }
            TypeKind::List(ty) => {
                write!(ctx, "[")?;
                ty.print(ctx)?;
                Ok(write!(ctx, "]")?)
            }
            TypeKind::Record(fields) => {
                ctx.write_many("{ ", " }", ", ", fields, |ctx, (name, ty)| {
                    name.print(ctx)?;
                    write!(ctx, ": ")?;
                    ty.print(ctx)
                })
            }
            TypeKind::Fn(params, ret) => {
                ctx.write_many("(", ")", ", ", params, |ctx, ty| ty.print(ctx))?;
                write!(ctx, " -> ")?;
                ret.print(ctx)
            }
        }
    }
}

impl Print for TypeDecl {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        write!(ctx, "type ")?;
        self.name.print(ctx)?;
        write!(ctx, " =")?;
        ctx.increase();
        for variant in &self.variants {
            ctx.newline()?;
            write!(ctx, "| ")?;
            variant.name.print(ctx)?;
            // `Rectangle(int, int)` is one tuple payload, printed without double parens.
            match variant.payload.as_ref().map(|ty| &ty.kind) {
                None => {}
                Some(TypeKind::Tuple(types)) => {
                    ctx.write_many("(", ")", ", ", types, |ctx, ty| ty.print(ctx))?;
                }
                Some(_) => {
                    write!(ctx, "(")?;
                    variant.payload.as_ref().unwrap().print(ctx)?;
                    write!(ctx, ")")?;
                }
            }
        }
        ctx.decrease();
        Ok(())
    }
}

// --- Patterns and destructors ---

impl Print for LiteralKind {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        match self {
            LiteralKind::Integer(value) => write!(ctx, "{value}")?,
            // Debug keeps the decimal point so the literal lexes as a float again.
            LiteralKind::Float(value) => write!(ctx, "{value:?}")?,
            LiteralKind::Bool(value) => write!(ctx, "{value}")?,
            LiteralKind::String(key) => {
                let contents = ctx.resolve(key);
                write!(ctx, "\"{contents}\"")?
            }
            LiteralKind::Unit => write!(ctx, "()")?,
        }
        Ok(())
    }
}

impl Print for Pattern {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        let (precedence, assoc) = pattern_precedence(self);
        let (lhs_min, rhs_min) = operand_minimums(precedence, assoc);
        let print_operand = |ctx: &mut Context, pattern: &Pattern, minimum| {
            print_child(ctx, pattern, pattern_precedence(pattern).0, minimum)
        };
        match &self.kind {
            PatternKind::Literal(literal) => literal.print(ctx),
            PatternKind::Var(name) => name.print(ctx),
            PatternKind::Wildcard => Ok(write!(ctx, "_")?),
            PatternKind::EmptyList => Ok(write!(ctx, "[]")?),
            PatternKind::Rest => Ok(write!(ctx, "...")?),
            PatternKind::Constructor { name, payload } => {
                name.print(ctx)?;
                match payload.as_deref().map(|pattern| &pattern.kind) {
                    None => Ok(()),
                    Some(PatternKind::Tuple(patterns)) => {
                        ctx.write_many("(", ")", ", ", patterns, |ctx, pattern| pattern.print(ctx))
                    }
                    Some(_) => {
                        write!(ctx, "(")?;
                        payload.as_ref().unwrap().print(ctx)?;
                        Ok(write!(ctx, ")")?)
                    }
                }
            }
            PatternKind::Tuple(patterns) => {
                ctx.write_many("(", ")", ", ", patterns, |ctx, pattern| pattern.print(ctx))
            }
            PatternKind::Record(fields) => {
                ctx.write_many("{ ", " }", ", ", fields, |ctx, (name, pattern)| {
                    name.print(ctx)?;
                    write!(ctx, ": ")?;
                    pattern.print(ctx)
                })
            }
            PatternKind::Cons { lhs, rhs } => {
                print_operand(ctx, lhs, lhs_min)?;
                write!(ctx, " :: ")?;
                print_operand(ctx, rhs, rhs_min)
            }
            PatternKind::Or { lhs, rhs } => {
                print_operand(ctx, lhs, lhs_min)?;
                write!(ctx, " | ")?;
                print_operand(ctx, rhs, rhs_min)
            }
        }
    }
}

impl Print for Destructor {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        match &self.kind {
            DestructorKind::Var(name) => name.print(ctx),
            DestructorKind::Rest => Ok(write!(ctx, "...")?),
            DestructorKind::Tuple(destructors) => {
                ctx.write_many("(", ")", ", ", destructors, |ctx, destructor| {
                    destructor.print(ctx)
                })
            }
            DestructorKind::Record(fields) => {
                ctx.write_many("{ ", " }", ", ", fields, |ctx, (name, destructor)| {
                    name.print(ctx)?;
                    // `{ x }` binds the field to a variable of the same name.
                    let Some(destructor) = destructor else {
                        return Ok(());
                    };
                    write!(ctx, ": ")?;
                    destructor.print(ctx)
                })
            }
            // Cons destructors are not parsed yet, so no precedence handling.
            DestructorKind::Cons { lhs, rhs } => {
                lhs.print(ctx)?;
                write!(ctx, " :: ")?;
                rhs.print(ctx)
            }
        }
    }
}

// --- Expressions ---

impl Print for Expr {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        let print_operand = |ctx: &mut Context, expr: &Expr, minimum| {
            print_child(ctx, expr, expr_precedence(expr), minimum)
        };
        match &self.kind {
            ExprKind::Literal(literal) => literal.print(ctx),
            ExprKind::Var(name) => name.print(ctx),
            ExprKind::BinaryExpr { op, lhs, rhs } => {
                let (precedence, assoc) = binary_op(op.kind);
                let (lhs_min, rhs_min) = operand_minimums(precedence, assoc);
                print_operand(ctx, lhs, lhs_min)?;
                write!(ctx, " {} ", op.kind)?;
                print_operand(ctx, rhs, rhs_min)
            }
            ExprKind::UnaryExpr { op, expr } => {
                write!(ctx, "{}", op.kind)?;
                print_operand(ctx, expr, UNARY)
            }
            ExprKind::Tuple(exprs) => {
                ctx.write_many("(", ")", ", ", exprs, |ctx, expr| expr.print(ctx))
            }
            ExprKind::List(exprs) => {
                ctx.write_many("[", "]", ", ", exprs, |ctx, expr| expr.print(ctx))
            }
            ExprKind::Record { fields } => {
                ctx.write_many("{ ", " }", ", ", fields, |ctx, (name, expr)| {
                    name.print(ctx)?;
                    write!(ctx, ": ")?;
                    expr.print(ctx)
                })
            }
            ExprKind::Conditional {
                condition,
                body,
                otherwise,
            } => {
                write!(ctx, "if ")?;
                condition.print(ctx)?;
                write!(ctx, " {{")?;
                ctx.increase();
                ctx.newline()?;
                body.print(ctx)?;
                ctx.decrease();
                ctx.newline()?;
                write!(ctx, "}}")?;
                if let Some(otherwise) = otherwise {
                    write!(ctx, " else {{")?;
                    ctx.increase();
                    ctx.newline()?;
                    otherwise.print(ctx)?;
                    ctx.decrease();
                    ctx.newline()?;
                    write!(ctx, "}}")?;
                }
                Ok(())
            }
            ExprKind::Match { value, arms } => {
                write!(ctx, "match ")?;
                value.print(ctx)?;
                write!(ctx, " {{")?;
                ctx.increase();
                for arm in arms {
                    ctx.newline()?;
                    write!(ctx, "| ")?;
                    arm.print(ctx)?;
                }
                ctx.decrease();
                ctx.newline()?;
                Ok(write!(ctx, "}}")?)
            }
            // ponytail: `ret_ty` is dropped. The parser has no grammar for lambda return types yet.
            ExprKind::Lambda { params, body, .. } => {
                ctx.write_many("", "", " ", params, |ctx, param| param.print(ctx))?;
                write!(ctx, " -> ")?;
                print_operand(ctx, body, LAMBDA)
            }
            ExprKind::Call {
                function,
                arguments,
            } => {
                print_operand(ctx, function, CALL)?;
                write!(ctx, " ")?;
                if arguments.is_empty() {
                    return Ok(write!(ctx, "()")?);
                }
                // Arguments must be atoms, so anything with an operator is parenthesised.
                ctx.write_many("", "", ", ", arguments, |ctx, argument| {
                    print_operand(ctx, argument, ATOM)
                })
            }
        }
    }
}

impl Print for LambdaParam {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        // OCaml style: an annotated parameter is written `(a: int)`.
        let Some(ty) = &self.ty else {
            return self.parameter.print(ctx);
        };
        write!(ctx, "(")?;
        self.parameter.print(ctx)?;
        write!(ctx, ": ")?;
        ty.print(ctx)?;
        Ok(write!(ctx, ")")?)
    }
}

impl Print for MatchArm {
    fn print(&self, ctx: &mut Context) -> Result<()> {
        self.pattern.print(ctx)?;
        write!(ctx, " -> ")?;
        self.expr.print(ctx)
    }
}
