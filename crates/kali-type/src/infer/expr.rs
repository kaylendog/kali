//! The walk over the AST: type expressions, expressions, and later patterns and
//! items.

use indexmap::IndexMap;

use crate::{
    builtins,
    error::{ErrorKind, Origin},
    infer::Infer,
    types::{Scheme, Type},
};

impl Infer {
    /// Converts a `kali_ast::Type` into a `Type` in the type system.
    pub(crate) fn type_expr(&mut self, ty: &kali_ast::Type) -> Type {
        use kali_ast::{PrimitiveTypeKind, TypeKind};
        match &ty.kind {
            TypeKind::Primitive(primitive) => match primitive {
                PrimitiveTypeKind::Integer => Type::Int,
                PrimitiveTypeKind::Float => Type::Float,
                PrimitiveTypeKind::Bool => Type::Bool,
                PrimitiveTypeKind::String => Type::String,
                PrimitiveTypeKind::Unit => Type::Unit,
            },
            TypeKind::Named(ident) => match self.scope.lookup_type(ident.key).cloned() {
                Some(ty) => ty,
                None => self.error(ErrorKind::UnknownName(ident.key), Origin::Name(ident.span)),
            },
            TypeKind::Tuple(items) => {
                Type::Tuple(items.iter().map(|item| self.type_expr(item)).collect())
            }
            TypeKind::List(item) => Type::List(self.type_expr(item).into()),
            TypeKind::Record(entries) => Type::Record(
                entries.iter().map(|(name, ty)| (name.key, self.type_expr(ty))).collect(),
            ),
            // `(a, b) -> r` is curried: `a -> b -> r`.
            TypeKind::Fn(params, ret) => {
                let ret = self.type_expr(ret);
                params
                    .iter()
                    .rev()
                    .fold(ret, |acc, param| Type::Fn(self.type_expr(param).into(), acc.into()))
            }
        }
    }

    /// Infers the type of an expression.
    pub(crate) fn expr(&mut self, expr: &kali_ast::Expr) -> Type {
        use kali_ast::{ExprKind, LiteralKind};
        let ty = match &expr.kind {
            ExprKind::Var(ident) => match self.scope.lookup_value(ident.key).cloned() {
                Some(scheme) => self.scope.instantiate(&scheme),
                None => self.error(ErrorKind::UnknownName(ident.key), Origin::Name(ident.span)),
            },
            ExprKind::Literal(kind) => match kind {
                LiteralKind::Integer(_) => Type::Int,
                LiteralKind::Float(_) => Type::Float,
                LiteralKind::Bool(_) => Type::Bool,
                LiteralKind::String(_) => Type::String,
                LiteralKind::Unit => Type::Unit,
            },
            ExprKind::BinaryExpr { op, lhs, rhs } => {
                // Synthesise a function type for the operator.
                let op_ty = self.scope.instantiate(&builtins::binary(op.kind));

                // Infer the types of the operands.
                let lhs_ty = self.expr(lhs);
                let rhs_ty = self.expr(rhs);

                // The result of the operator is a fresh type variable (since it
                // depends on the operator and operands).
                let result = self.scope.fresh();

                // Unify the operator type with the expected function type:
                // `lhs_ty -> rhs_ty -> result`.
                self.unify(
                    &op_ty,
                    &Type::Fn(lhs_ty.into(), Type::Fn(rhs_ty.into(), result.clone().into()).into()),
                    Origin::BinaryExpr { op: op.span, lhs: lhs.span, rhs: rhs.span },
                );
                result
            }
            ExprKind::UnaryExpr { op, expr } => {
                let op_ty = self.scope.instantiate(&builtins::unary(op.kind));
                let expr_ty = self.expr(expr);
                let result = self.scope.fresh();
                self.unify(
                    &op_ty,
                    &Type::Fn(expr_ty.into(), result.clone().into()),
                    Origin::UnaryExpr { op: op.span, expr: expr.span },
                );
                result
            }
            ExprKind::Tuple(exprs) => {
                Type::Tuple(exprs.iter().map(|expr| self.expr(expr)).collect())
            }
            // Every element unifies with one fresh element type; `[]` is a list of anything.
            ExprKind::List(exprs) => {
                let element = self.scope.fresh();
                for expr in exprs {
                    let ty = self.expr(expr);
                    self.unify(&element, &ty, Origin::Name(expr.span));
                }
                Type::List(element.into())
            }
            ExprKind::Record { fields } => Type::Record(
                fields.iter().map(|(name, expr)| (name.key, self.expr(expr))).collect(),
            ),
            ExprKind::Conditional { .. } => todo!(),
            ExprKind::Match { .. } => todo!(),
            ExprKind::Lambda { params, body, .. } => {
                // Enter a new scope for the lambda parameters and body.
                self.scope.enter();
                let param_types: Vec<_> = params
                    .iter()
                    .map(|param| {
                        let ty = self.destructor(&param.parameter);
                        // If the parameter has a type annotation, unify it with
                        // the inferred type.
                        if let Some(ty_expr) = &param.ty {
                            let expected_ty = self.type_expr(ty_expr);
                            let origin = Origin::LambdaAnnotation {
                                annotation: ty_expr.span,
                                value: param.parameter.span,
                            };
                            self.unify(&ty, &expected_ty, origin);
                        }
                        ty
                    })
                    .collect();
                let ret_ty = self.expr(body);
                self.scope.exit();

                // Construct the function type from the parameter types and
                // return type.
                param_types
                    .into_iter()
                    .rev()
                    .fold(ret_ty, |acc, param| Type::Fn(param.into(), acc.into()))
            }
            ExprKind::Call { function, arguments } => {
                let mut acc = self.expr(function);
                // If we have no arguments, then implicitly call with unit.
                if arguments.is_empty() {
                    let result = self.scope.fresh();
                    self.unify(
                        &acc,
                        &Type::Fn(Type::Unit.into(), result.clone().into()),
                        Origin::Name(function.span),
                    );
                    result
                } else {
                    // Otherwise, for each argument, unify the function type
                    // with a function from the argument
                    // type to a fresh result type.
                    for arg in arguments {
                        let arg_ty = self.expr(arg);
                        let result = self.scope.fresh();
                        self.unify(
                            &acc,
                            &Type::Fn(arg_ty.into(), result.clone().into()),
                            Origin::Name(arg.span),
                        );
                        acc = result;
                    }
                    acc
                }
            }
        };
        self.cache.with(expr, ty)
    }

    /// Infers the type of a destructor pattern, binding any names in the
    /// current scope.
    pub(crate) fn destructor(&mut self, d: &kali_ast::Destructor) -> Type {
        // Bind names in the destructor to current scope
        use kali_ast::DestructorKind;
        match &d.kind {
            DestructorKind::Var(ident) => {
                let ty = self.scope.fresh();
                self.scope.bind_value(
                    ident.key,
                    Scheme { vars: Vec::new(), ty: ty.clone(), constraints: Vec::new() },
                );
                ty
            }
            DestructorKind::Tuple(destructors) => {
                Type::Tuple(destructors.iter().map(|d| self.destructor(d)).collect())
            }
            DestructorKind::Record(index_map) => {
                // For each field in the record:
                // - If it is destructed, recursively infer the type of the
                //   destructor (which may bind names in the current scope).
                // - Otherwise, create a fresh type variable for the field and
                //   bind it in the current scope.
                let mut entry_types = IndexMap::new();
                for (name, d) in index_map {
                    if let Some(d) = d {
                        // If the field is destructed, recursively infer its
                        // type, which may bind names in the current scope.
                        entry_types.insert(name.key, self.destructor(d));
                    } else {
                        // Otherwise, create a fresh type variable for the field
                        // and bind it in the current scope.
                        let ty = self.scope.fresh();
                        self.scope.bind_value(
                            name.key,
                            Scheme { vars: Vec::new(), ty: ty.clone(), constraints: Vec::new() },
                        );
                        entry_types.insert(name.key, ty);
                    }
                }
                Type::Record(entry_types)
            }
            DestructorKind::Cons { lhs, rhs } => {
                let lhs_ty = self.destructor(lhs);
                let rhs_ty = self.destructor(rhs);
                let list_ty = Type::List(lhs_ty.into());
                self.unify(&list_ty, &rhs_ty, Origin::Name(d.span));
                list_ty
            }
            DestructorKind::Rest => Type::List(self.scope.fresh().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use kali_parse::kali_unwrap;
    use pretty_assertions::assert_eq;

    use crate::{error::Error, infer::Infer, types::Type};

    /// The type expression of the `idx`th type alias in the module.
    fn type_expr_of(module: &kali_ast::Module, idx: usize) -> kali_ast::Type {
        let item = module
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                kali_ast::ItemKind::TypeAlias(alias) => Some(alias),
                _ => None,
            })
            .nth(idx)
            .expect("module does not contain enough type aliases");
        item.ty.clone()
    }

    #[test]
    fn type_expr() {
        let module = kali_unwrap! {
            type P = { x: int };
            type Q = [P];
        };
        let p = module.cache.get("P").unwrap();
        let x = module.cache.get("x").unwrap();

        let mut infer = Infer::default();

        let p_ty = infer.type_expr(&type_expr_of(&module, 0));
        // Register the alias by hand, since this is not the full `check`.
        infer.scope.bind_type(p, p_ty.clone());
        let q_ty = infer.type_expr(&type_expr_of(&module, 1));

        assert_eq!(Vec::<Error>::default(), infer.errors);
        assert_eq!(p_ty, Type::Record([(x, Type::Int)].into()));
        assert_eq!(q_ty, Type::List(Type::Record([(x, Type::Int)].into()).into()));
    }

    #[test]
    fn unknown_type_name() {
        let module = kali_unwrap! {
            type R = nope;
        };
        let mut infer = Infer::default();
        assert_eq!(Type::Error, infer.type_expr(&type_expr_of(&module, 0)));
        assert_eq!(1, infer.errors.len());
    }
}
