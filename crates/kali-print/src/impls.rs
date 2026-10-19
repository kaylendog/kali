//! Width-aware printer built on Wadler-style documents from the `pretty` crate.
//! Each node builds a `Doc`; `render` lays it out against the configured width.
//!
//! Line breaks are only placed where the parser's automatic `;` insertion
//! cannot misread them: after an operator, a comma, an opening bracket or `->`.
//! Never between a call and its arguments.

use std::{borrow::Cow, collections::BTreeMap, io};

use kali_ast::{
    BinaryOpKind, Comment, CommentKind, DataDecl, Definition, Expr, ExprKind, Ident, ImportTree,
    ImportTreeKind, Item, ItemKind, LambdaParam, LiteralKind, MatchArm, Module, NodeId, Pattern,
    PatternKind, PrimitiveTypeKind, Type, TypeAlias, TypeKind, Variant, Visibility,
};
use pretty::{RcDoc, Render, RenderAnnotated};

use crate::pass::Comments;

type Doc = RcDoc<'static, ()>;

/// Settings for pretty-printing.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// Columns before a group breaks.
    pub width: usize,
    /// Columns per indentation level. With `tabs`, also the width one tab
    /// counts for.
    pub indent: usize,
    /// Indent with tabs instead of spaces.
    pub tabs: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { width: 100, indent: 4, tabs: true }
    }
}

/// State needed to build a module's document: the interner, the source for
/// blank-line preservation, and the comments attached to each node.
pub struct Context<'a> {
    settings: Settings,
    cache: &'a lasso::Rodeo,
    src: &'a str,
    /// Comments attached to nodes. Removed as they are printed.
    pub(crate) comments: BTreeMap<NodeId, Comments>,
}

impl<'a> Context<'a> {
    pub fn new(
        settings: Settings,
        cache: &'a lasso::Rodeo,
        src: &'a str,
        comments: BTreeMap<NodeId, Comments>,
    ) -> Context<'a> {
        Context { settings, cache, src, comments }
    }

    fn indent(&self) -> isize {
        self.settings.indent as isize
    }

    fn text(&self, text: impl Into<Cow<'static, str>>) -> Doc {
        RcDoc::text(text)
    }

    fn resolve(&self, key: &lasso::Spur) -> Doc {
        RcDoc::text(self.cache.resolve(key).to_string())
    }

    /// `open` items `close`, on one line if it fits, else one item per line
    /// with a trailing comma. `pad` puts a space inside the brackets when
    /// flat, as in `{ x: int }`.
    fn delimited(
        &self,
        open: &'static str,
        close: &'static str,
        pad: bool,
        items: Vec<Doc>,
    ) -> Doc {
        if items.is_empty() {
            return self.text(open).append(self.text(close));
        }
        let edge = || if pad { RcDoc::line() } else { RcDoc::line_() };
        self.text(open)
            .append(
                edge()
                    .append(RcDoc::intersperse(items, self.text(",").append(RcDoc::line())))
                    .append(self.text(",").flat_alt(RcDoc::nil()))
                    .nest(self.indent()),
            )
            .append(edge())
            .append(self.text(close))
            .group()
    }

    /// `{` body `}` with the body always on its own lines.
    fn block(&self, body: Doc) -> Doc {
        self.text("{")
            .append(RcDoc::hardline().append(body).nest(self.indent()))
            .append(RcDoc::hardline())
            .append(self.text("}"))
    }

    /// The right-hand side of `=` or `->`: an expression that breaks anyway, or
    /// one that can break inside its own brackets, hugs the operator.
    /// Anything else may move to the next line, indented.
    fn rhs(&self, expr: &Expr, doc: Doc) -> Doc {
        let bracketed =
            matches!(expr.kind, ExprKind::Tuple(_) | ExprKind::List(_) | ExprKind::Record { .. });
        if bracketed || has_hard_break(expr) {
            RcDoc::space().append(doc)
        } else {
            RcDoc::line().append(doc).nest(self.indent()).group()
        }
    }

    /// The separator between two things that were at `from..to` in the source:
    /// a space on the same line, a newline, or one blank line for two or
    /// more. `at_least` forces a minimum.
    fn gap(&self, from: usize, to: usize, at_least: usize) -> Doc {
        let newlines = self.src.get(from..to).map_or(1, |text| text.matches('\n').count());
        match newlines.max(at_least) {
            0 => RcDoc::space(),
            1 => RcDoc::hardline(),
            _ => RcDoc::hardline().append(RcDoc::hardline()),
        }
    }
}

/// Whether `expr` prints with a hard line break, so no group containing it can
/// be laid out flat. `match` and `if` always break; anything else breaks if an
/// operand does.
fn has_hard_break(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Match { .. } | ExprKind::Conditional { .. } => true,
        ExprKind::Lambda { body, .. } => has_hard_break(body),
        ExprKind::UnaryExpr { expr, .. } => has_hard_break(expr),
        ExprKind::BinaryExpr { lhs, rhs, .. } => has_hard_break(lhs) || has_hard_break(rhs),
        ExprKind::Call { function, arguments } => {
            has_hard_break(function) || arguments.iter().any(has_hard_break)
        }
        ExprKind::Tuple(exprs) | ExprKind::List(exprs) => exprs.iter().any(has_hard_break),
        ExprKind::Record { fields } => fields.values().any(has_hard_break),
        ExprKind::Literal(_) | ExprKind::Var(_) => false,
    }
}

/// Implemented by AST nodes that can be pretty-printed.
pub trait PrettyPrint {
    fn pretty_print(&self, ctx: &mut Context) -> Doc;
}

impl<T: PrettyPrint> PrettyPrint for Box<T> {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        self.as_ref().pretty_print(ctx)
    }
}

impl PrettyPrint for Ident {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        ctx.resolve(&self.key)
    }
}

impl PrettyPrint for Comment {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        let marker = match self.kind {
            CommentKind::Single => "//",
            CommentKind::Doc => "///",
        };
        let contents = ctx.cache.resolve(&self.contents);
        if contents.is_empty() {
            ctx.text(marker)
        } else {
            ctx.text(format!("{marker} {contents}"))
        }
    }
}

// --- Module and items ---

impl PrettyPrint for Module {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        // Source offset where the last printed thing ended. `None` until
        // something is printed, so blank lines at the top of the file
        // are dropped.
        let mut last_end: Option<usize> = None;
        let mut out = RcDoc::nil();
        let mut place = |ctx: &Context, out: Doc, doc: Doc, start: usize, end: usize, at_least| {
            let gap = match last_end {
                Some(from) => ctx.gap(from, start, at_least),
                None => RcDoc::nil(),
            };
            last_end = Some(end);
            out.append(gap).append(doc)
        };

        for item in &self.items {
            let comments = ctx.comments.remove(&item.id).unwrap_or_default();
            for comment in &comments.leading {
                let doc = comment.pretty_print(ctx);
                out = place(ctx, out, doc, comment.span.start, comment.span.end, 0);
            }
            // Line ends separate items, so an item always starts a new line.
            let doc = item.pretty_print(ctx);
            out = place(ctx, out, doc, item.span.start, item.span.end, 1);
            // Dangling comments have no slot of their own yet, so they follow
            // the item.
            for comment in &comments.dangling {
                out = out.append(RcDoc::hardline()).append(comment.pretty_print(ctx));
            }
            for comment in &comments.trailing {
                let doc = comment.pretty_print(ctx);
                out = place(ctx, out, doc, comment.span.start, comment.span.end, 0);
            }
        }
        // With no items, the comments pass has nothing to attach the comments
        // to.
        if self.items.is_empty() {
            for comment in self.comments.values() {
                let doc = comment.pretty_print(ctx);
                out = place(ctx, out, doc, comment.span.start, comment.span.end, 0);
            }
        }
        if last_end.is_some() {
            out = out.append(RcDoc::hardline());
        }
        out
    }
}

impl PrettyPrint for Item {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        let doc = match &self.kind {
            ItemKind::Definition(definition) => definition.pretty_print(ctx),
            ItemKind::TypeAlias(alias) => alias.pretty_print(ctx),
            ItemKind::DataDecl(decl) => decl.pretty_print(ctx),
            ItemKind::Import(import) => ctx.text("import ").append(import.pretty_print(ctx)),
        };
        if self.visibility == Visibility::Exported { ctx.text("export ").append(doc) } else { doc }
    }
}

impl PrettyPrint for Definition {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        let expr = self.expr.pretty_print(ctx);
        ctx.text("let ")
            .append(self.name.pretty_print(ctx))
            .append(ctx.text(" ="))
            .append(ctx.rhs(&self.expr, expr))
    }
}

impl PrettyPrint for TypeAlias {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        ctx.text("type ")
            .append(self.name.pretty_print(ctx))
            .append(ctx.text(" = "))
            .append(self.ty.pretty_print(ctx))
    }
}

impl PrettyPrint for DataDecl {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        // `type S = A | B` when it fits, else one variant per line, each after
        // a leading `|`.
        let variants = self
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let pipe =
                    if index == 0 { ctx.text("| ").flat_alt(RcDoc::nil()) } else { ctx.text("| ") };
                RcDoc::line().append(pipe).append(variant.pretty_print(ctx))
            })
            .collect::<Vec<_>>();
        ctx.text("data ")
            .append(self.name.pretty_print(ctx))
            .append(ctx.text(" ="))
            .append(RcDoc::concat(variants).nest(ctx.indent()).group())
    }
}

impl PrettyPrint for Variant {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        // `Rectangle(int, int)` is one tuple payload, printed without double
        // parens.
        let payload = match self.payload.as_ref().map(|ty| &ty.kind) {
            None => RcDoc::nil(),
            Some(TypeKind::Tuple(types)) => {
                let types = types.iter().map(|ty| ty.pretty_print(ctx)).collect();
                ctx.delimited("(", ")", false, types)
            }
            Some(_) => {
                let ty = self.payload.as_ref().unwrap().pretty_print(ctx);
                ctx.delimited("(", ")", false, vec![ty])
            }
        };
        self.name.pretty_print(ctx).append(payload)
    }
}

impl PrettyPrint for ImportTree {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        match &self.kind {
            ImportTreeKind::Item { name, alias } => {
                let name = name.pretty_print(ctx);
                match alias {
                    Some(alias) => name.append(ctx.text(" as ")).append(alias.pretty_print(ctx)),
                    None => name,
                }
            }
            // Rust style: `std::io::{read, write}`, no spaces around `::` or inside braces.
            ImportTreeKind::Segment { name, child } => {
                name.pretty_print(ctx).append(ctx.text("::")).append(child.pretty_print(ctx))
            }
            ImportTreeKind::Glob => ctx.text("*"),
            ImportTreeKind::List(trees) => {
                let trees = trees.iter().map(|tree| tree.pretty_print(ctx)).collect();
                ctx.delimited("{", "}", false, trees)
            }
        }
    }
}

// --- Types ---

impl PrettyPrint for Type {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        match &self.kind {
            TypeKind::Primitive(primitive) => ctx.text(match primitive {
                PrimitiveTypeKind::Integer => "int",
                PrimitiveTypeKind::Float => "float",
                PrimitiveTypeKind::Bool => "bool",
                PrimitiveTypeKind::String => "string",
                PrimitiveTypeKind::Unit => "()",
            }),
            TypeKind::Named { name, params } => {
                if params.is_empty() {
                    name.pretty_print(ctx)
                } else {
                    let params = params.iter().map(|ty| ty.pretty_print(ctx)).collect();
                    name.pretty_print(ctx).append(ctx.delimited("<", ">", false, params))
                }
            }
            TypeKind::Tuple(types) => {
                let types = types.iter().map(|ty| ty.pretty_print(ctx)).collect();
                ctx.delimited("(", ")", false, types)
            }
            TypeKind::List(ty) => ctx.text("[").append(ty.pretty_print(ctx)).append(ctx.text("]")),
            TypeKind::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, ty)| {
                        name.pretty_print(ctx).append(ctx.text(": ")).append(ty.pretty_print(ctx))
                    })
                    .collect();
                ctx.delimited("{", "}", true, fields)
            }
            TypeKind::Fn(params, ret) => {
                let params = params.iter().map(|ty| ty.pretty_print(ctx)).collect();
                ctx.delimited("(", ")", false, params)
                    .append(ctx.text(" -> "))
                    .append(ret.pretty_print(ctx))
            }
        }
    }
}

// --- Precedence ---
//
// Binding powers mirror the pratt tables in the parser. A child is
// parenthesised when its binding power is below the minimum its position
// requires.

#[derive(Clone, Copy)]
enum Assoc {
    Left,
    Right,
}

/// Binding power of nodes that are never split by an operator.
const ATOM: u8 = u8::MAX;
const CALL: u8 = 11;
const UNARY: u8 = 10;
const LAMBDA: u8 = 1;

fn binary_op(kind: BinaryOpKind) -> (u8, Assoc) {
    use BinaryOpKind::*;
    match kind {
        Exponentiate => (9, Assoc::Right),
        // Bitwise and concat operators are not parsed yet; placed beside their nearest cousins.
        Multiply | Divide | Modulo | BitwiseShiftLeft | BitwiseShiftRight => (8, Assoc::Left),
        Add | Subtract => (7, Assoc::Left),
        // Between arithmetic and comparison, as in OCaml and Haskell.
        Cons | Concat => (6, Assoc::Right),
        LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => (5, Assoc::Left),
        Equal | NotEqual => (4, Assoc::Left),
        LogicalAnd | BitwiseAnd | BitwiseXor => (3, Assoc::Left),
        LogicalOr | BitwiseOr => (2, Assoc::Left),
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
        // `|` is loosest, as in OCaml and Rust: `a :: b | c` is `(a :: b) | c`.
        PatternKind::Cons { .. } => (2, Assoc::Right),
        PatternKind::Or { .. } => (1, Assoc::Left),
        _ => (ATOM, Assoc::Left),
    }
}

/// Prints `child`, parenthesised if its binding power is below `minimum`.
fn print_child<T: PrettyPrint>(ctx: &mut Context, child: &T, precedence: u8, minimum: u8) -> Doc {
    let doc = child.pretty_print(ctx);
    if precedence < minimum { ctx.text("(").append(doc).append(ctx.text(")")) } else { doc }
}

// --- Patterns and destructors ---

impl PrettyPrint for LiteralKind {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        match self {
            LiteralKind::Integer(value) => ctx.text(value.to_string()),
            // Debug keeps the decimal point so the literal lexes as a float again.
            LiteralKind::Float(value) => ctx.text(format!("{value:?}")),
            LiteralKind::Bool(value) => ctx.text(value.to_string()),
            LiteralKind::String(key) => ctx.text(format!("\"{}\"", ctx.cache.resolve(key))),
            LiteralKind::Unit => ctx.text("()"),
        }
    }
}

impl PrettyPrint for Pattern {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        let (precedence, assoc) = pattern_precedence(self);
        let (lhs_min, rhs_min) = operand_minimums(precedence, assoc);
        let operand = |ctx: &mut Context, pattern: &Pattern, minimum| {
            print_child(ctx, pattern, pattern_precedence(pattern).0, minimum)
        };
        match &self.kind {
            PatternKind::Literal(literal) => literal.pretty_print(ctx),
            PatternKind::Var(name) => name.pretty_print(ctx),
            PatternKind::Wildcard => ctx.text("_"),
            PatternKind::EmptyList => ctx.text("[]"),
            PatternKind::Rest => ctx.text("..."),
            PatternKind::Constructor { name, payload } => {
                let payload = match payload.as_deref().map(|pattern| &pattern.kind) {
                    None => RcDoc::nil(),
                    Some(PatternKind::Tuple(patterns)) => {
                        let patterns = patterns.iter().map(|p| p.pretty_print(ctx)).collect();
                        ctx.delimited("(", ")", false, patterns)
                    }
                    Some(_) => {
                        let pattern = payload.as_ref().unwrap().pretty_print(ctx);
                        ctx.delimited("(", ")", false, vec![pattern])
                    }
                };
                name.pretty_print(ctx).append(payload)
            }
            PatternKind::Tuple(patterns) => {
                let patterns = patterns.iter().map(|p| p.pretty_print(ctx)).collect();
                ctx.delimited("(", ")", false, patterns)
            }
            PatternKind::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, pattern)| {
                        if let PatternKind::Var(binding) = &pattern.kind
                            && binding.key == name.key
                        {
                            name.pretty_print(ctx)
                        } else {
                            name.pretty_print(ctx)
                                .append(ctx.text(": "))
                                .append(pattern.pretty_print(ctx))
                        }
                    })
                    .collect();
                ctx.delimited("{", "}", true, fields)
            }
            PatternKind::Cons { lhs, rhs } => operand(ctx, lhs, lhs_min)
                .append(ctx.text(" :: "))
                .append(operand(ctx, rhs, rhs_min)),
            PatternKind::Or { lhs, rhs } => operand(ctx, lhs, lhs_min)
                .append(ctx.text(" | "))
                .append(operand(ctx, rhs, rhs_min)),
        }
    }
}

// --- Expressions ---

impl PrettyPrint for Expr {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        let operand = |ctx: &mut Context, expr: &Expr, minimum| {
            print_child(ctx, expr, expr_precedence(expr), minimum)
        };
        match &self.kind {
            ExprKind::Literal(literal) => literal.pretty_print(ctx),
            ExprKind::Var(name) => name.pretty_print(ctx),
            ExprKind::BinaryExpr { op, lhs, rhs } => {
                let (precedence, assoc) = binary_op(op.kind);
                let (lhs_min, rhs_min) = operand_minimums(precedence, assoc);
                // The break goes after the operator: an operator at the end of
                // a line continues the item, an operand at the
                // end of a line terminates it. An operand that
                // breaks anyway, such as a `match`, keeps the operator on its
                // line.
                let separator = if has_hard_break(lhs) || has_hard_break(rhs) {
                    RcDoc::space()
                } else {
                    RcDoc::line()
                };
                operand(ctx, lhs, lhs_min)
                    .append(ctx.text(format!(" {}", op.kind)))
                    .append(separator.append(operand(ctx, rhs, rhs_min)).nest(ctx.indent()))
                    .group()
            }
            ExprKind::UnaryExpr { op, expr } => {
                // Always parenthesise a compound operand: `-f y` reads as `(-f)
                // y` to anyone used to OCaml, so print `-(f y)`
                // even though the parser needs no parens.
                ctx.text(op.kind.to_string()).append(operand(ctx, expr, ATOM))
            }
            ExprKind::Tuple(exprs) => {
                let exprs = exprs.iter().map(|expr| expr.pretty_print(ctx)).collect();
                ctx.delimited("(", ")", false, exprs)
            }
            ExprKind::List(exprs) => {
                let exprs = exprs.iter().map(|expr| expr.pretty_print(ctx)).collect();
                ctx.delimited("[", "]", false, exprs)
            }
            ExprKind::Record { fields } => {
                let fields = fields
                    .iter()
                    .map(|(name, expr)| {
                        name.pretty_print(ctx).append(ctx.text(": ")).append(expr.pretty_print(ctx))
                    })
                    .collect();
                ctx.delimited("{", "}", true, fields)
            }
            ExprKind::Conditional { condition, then: body, otherwise } => {
                let body = body.pretty_print(ctx);
                let mut doc = ctx
                    .text("if ")
                    .append(condition.pretty_print(ctx))
                    .append(RcDoc::space())
                    .append(ctx.block(body));
                let otherwise_doc = otherwise.pretty_print(ctx);
                doc = doc.append(ctx.text(" else "));
                // `else { if .. }` prints as `else if ..`, so chains stay
                // flat.
                doc = if matches!(otherwise.kind, ExprKind::Conditional { .. }) {
                    doc.append(otherwise_doc)
                } else {
                    doc.append(ctx.block(otherwise_doc))
                };
                doc
            }
            ExprKind::Match { value, arms } => {
                let arms = arms
                    .iter()
                    .map(|arm| {
                        RcDoc::hardline().append(ctx.text("| ")).append(arm.pretty_print(ctx))
                    })
                    .collect::<Vec<_>>();
                ctx.text("match ")
                    .append(value.pretty_print(ctx))
                    .append(ctx.text(" {"))
                    .append(RcDoc::concat(arms).nest(ctx.indent()))
                    .append(RcDoc::hardline())
                    .append(ctx.text("}"))
            }
            // ponytail: `ret_ty` is dropped. The parser has no grammar for lambda return types yet.
            ExprKind::Lambda { params, body, .. } => {
                let params = params.iter().map(|param| param.pretty_print(ctx)).collect::<Vec<_>>();
                let body_doc = operand(ctx, body, LAMBDA);
                RcDoc::intersperse(params, RcDoc::space())
                    .append(ctx.text(" ->"))
                    .append(ctx.rhs(body, body_doc))
            }
            ExprKind::Call { function, arguments } => {
                // Never breaks: a newline after an argument would end the item.
                let function = operand(ctx, function, CALL).append(RcDoc::space());
                if arguments.is_empty() {
                    return function.append(ctx.text("()"));
                }
                // Arguments must be atoms, so anything with an operator is
                // parenthesised.
                let arguments = arguments
                    .iter()
                    .map(|argument| operand(ctx, argument, ATOM))
                    .collect::<Vec<_>>();
                function.append(RcDoc::intersperse(arguments, ctx.text(", ")))
            }
        }
    }
}

impl PrettyPrint for LambdaParam {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        // OCaml style: an annotated parameter is written `(a: int)`.
        let Some(ty) = &self.ty else {
            return self.parameter.pretty_print(ctx);
        };
        ctx.text("(")
            .append(self.parameter.pretty_print(ctx))
            .append(ctx.text(": "))
            .append(ty.pretty_print(ctx))
            .append(ctx.text(")"))
    }
}

impl PrettyPrint for MatchArm {
    fn pretty_print(&self, ctx: &mut Context) -> Doc {
        let expr = self.expr.pretty_print(ctx);
        self.pattern.pretty_print(ctx).append(ctx.text(" ->")).append(ctx.rhs(&self.expr, expr))
    }
}

// --- Rendering ---

/// Writes a document to an `io::Write`, turning the spaces `pretty` emits after
/// each newline into tabs. Indentation arrives as its own `write_str` call
/// straight after the `"\n"`, so a run of spaces anywhere else, such as inside
/// a string literal, is left alone.
struct Writer<'w> {
    out: &'w mut dyn io::Write,
    indent: usize,
    tabs: bool,
    after_newline: bool,
}

impl Render for Writer<'_> {
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        let is_indent = self.after_newline && !s.is_empty() && s.bytes().all(|b| b == b' ');
        self.after_newline = s == "\n";
        if is_indent && self.tabs {
            for _ in 0..s.len() / self.indent {
                self.out.write_all(b"\t")?;
            }
            self.out.write_all(&s.as_bytes()[..s.len() % self.indent])?;
        } else {
            self.out.write_all(s.as_bytes())?;
        }
        Ok(s.len())
    }

    fn fail_doc(&self) -> io::Error {
        io::Error::other("document failed to render")
    }
}

impl RenderAnnotated<'_, ()> for Writer<'_> {
    fn push_annotation(&mut self, _: &()) -> io::Result<()> {
        Ok(())
    }

    fn pop_annotation(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Lays out `doc` against `settings.width` and writes it to `out`.
pub(crate) fn render(doc: &Doc, settings: Settings, out: &mut dyn io::Write) -> io::Result<()> {
    let mut writer =
        Writer { out, indent: settings.indent.max(1), tabs: settings.tabs, after_newline: false };
    doc.render_raw(settings.width, &mut writer)
}
