use kali_ast::{Definition, Expr, ExprKind, ItemKind, LiteralKind, Module, Pattern, PatternKind};
pub use pretty_assertions::assert_eq;

/// Parses `src`, panicking on any error.
pub fn parses(src: &str) -> Module {
    let (module, errors) = kali_parse::parse_str(src);
    assert!(errors.is_empty(), "{src:?} failed to parse: {errors:?}");
    module.expect("no errors but no module")
}

/// Asserts that `src` does not parse.
pub fn rejects(src: &str) {
    let (_, errors) = kali_parse::parse_str(src);
    assert!(!errors.is_empty(), "{src:?} parsed but should not");
}

/// Parses `src` as the right-hand side of a definition and renders it as an s-expression, so
/// grouping can be asserted independently of the printer's precedence table.
pub fn sexp(src: &str) -> String {
    let module = parses(&format!("let it = {src}"));
    let ItemKind::Definition(Definition { expr, .. }) = &module.items[0].kind else {
        panic!("expected a definition");
    };
    render(expr, &module)
}

/// Parses `src` as a match arm's pattern and renders it as an s-expression.
pub fn pattern_sexp(src: &str) -> String {
    let module = parses(&format!("let it = match v {{ | {src} -> 0 }}"));
    let ItemKind::Definition(Definition { expr, .. }) = &module.items[0].kind else {
        panic!("expected a definition");
    };
    let ExprKind::Match { arms, .. } = &expr.kind else {
        panic!("expected a match");
    };
    render_pattern(&arms[0].pattern, &module)
}

fn render_pattern(pattern: &Pattern, module: &Module) -> String {
    let name = |ident: &kali_ast::Ident| module.cache.resolve(&ident.key).to_string();
    let many = |patterns: &[Pattern]| {
        patterns
            .iter()
            .map(|pattern| render_pattern(pattern, module))
            .collect::<Vec<_>>()
            .join(" ")
    };
    match &pattern.kind {
        PatternKind::Var(ident) => name(ident),
        PatternKind::Literal(LiteralKind::Integer(value)) => value.to_string(),
        PatternKind::Wildcard => "_".into(),
        PatternKind::EmptyList => "[]".into(),
        PatternKind::Tuple(patterns) => format!("(tuple {})", many(patterns)),
        PatternKind::Cons { lhs, rhs } => {
            format!(
                "(:: {} {})",
                render_pattern(lhs, module),
                render_pattern(rhs, module)
            )
        }
        PatternKind::Or { lhs, rhs } => {
            format!(
                "(| {} {})",
                render_pattern(lhs, module),
                render_pattern(rhs, module)
            )
        }
        PatternKind::Constructor {
            name: ctor,
            payload,
        } => match payload {
            Some(payload) => format!("({} {})", name(ctor), render_pattern(payload, module)),
            None => name(ctor),
        },
        other => format!("<{}>", std::any::type_name_of_val(other)),
    }
}

fn render(expr: &Expr, module: &Module) -> String {
    let name = |ident: &kali_ast::Ident| module.cache.resolve(&ident.key).to_string();
    match &expr.kind {
        ExprKind::Var(ident) => name(ident),
        ExprKind::Literal(LiteralKind::Integer(value)) => value.to_string(),
        ExprKind::Literal(LiteralKind::Float(value)) => value.to_string(),
        ExprKind::Literal(LiteralKind::Bool(value)) => value.to_string(),
        ExprKind::Literal(LiteralKind::String(key)) => format!("{:?}", module.cache.resolve(key)),
        ExprKind::Literal(LiteralKind::Unit) => "()".into(),
        ExprKind::BinaryExpr { op, lhs, rhs } => {
            format!(
                "({} {} {})",
                op.kind,
                render(lhs, module),
                render(rhs, module)
            )
        }
        ExprKind::UnaryExpr { op, expr } => format!("({} {})", op.kind, render(expr, module)),
        ExprKind::Call {
            function,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| render(argument, module))
                .collect::<Vec<_>>()
                .join(" ");
            format!("(call {} [{arguments}])", render(function, module))
        }
        ExprKind::Lambda { params, body, .. } => {
            format!("(lambda {} {})", params.len(), render(body, module))
        }
        ExprKind::Tuple(items) | ExprKind::List(items) => {
            let open = if matches!(expr.kind, ExprKind::Tuple(_)) {
                "(tuple"
            } else {
                "(list"
            };
            let items = items
                .iter()
                .map(|item| render(item, module))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{open} {items})")
        }
        other => format!("<{}>", std::any::type_name_of_val(other)),
    }
}
