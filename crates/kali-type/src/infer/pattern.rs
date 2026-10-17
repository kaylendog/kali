use indexmap::IndexMap;

use crate::{error::Origin, types::TypeVar, Infer, Type};

impl Infer {
    /// Infer the type of a pattern, returning the type and recording any errors.
    pub(crate) fn pattern(&mut self, pat: &kali_ast::Pattern, expected: &Type) {
        use kali_ast::PatternKind;
        match (&pat.kind, expected) {
            (
                PatternKind::Literal(_),
                Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit,
            ) => {
                // Nothing to bind.
            }
            (PatternKind::Var(ident), expected) => {
                // Bind the variable to the expected type in the current scope.
                self.scope.bind_value(ident.key, self.scope.generalise(expected));
            }
            (PatternKind::Tuple(patterns), Type::Tuple(items)) => {
                for (pat, ty) in patterns.iter().zip(items.iter()) {
                    self.pattern(pat, ty);
                }
            }
            (PatternKind::Wildcard, _) => {
                // Nothing to bind.
            }
            (PatternKind::Record(pat_entries), Type::Record(ty_entries)) => {
                for (name, pattern) in pat_entries.iter() {
                    if let Some(ty_entry) = ty_entries.get(&name.key) {
                        self.pattern(pattern, ty_entry);
                    } else {
                        self.error(
                            crate::error::ErrorKind::UnknownName(name.key),
                            Origin::Pattern(pat.span),
                        );
                    }
                }

                // No need to check for missing fields if the pattern has a rest binding.
                if pat_entries.values().any(|p| matches!(p.kind, PatternKind::Rest)) {
                    return;
                }

                // Check for missing fields in the pattern.
                for name in ty_entries.keys() {
                    todo!()
                }
            }
            (PatternKind::Cons { lhs, rhs }, Type::List(inner)) => {
                self.pattern(lhs, inner);
                self.pattern(rhs, expected);
            }
            (PatternKind::EmptyList, Type::List(_)) => {
                // Nothing to bind.
            }
            (PatternKind::Or { lhs, rhs }, _) => {
                self.pattern(lhs, expected);
                self.pattern(rhs, expected);
            }
            (PatternKind::Constructor { name, payload }, _) => {
                todo!("constructors")
            }
            (PatternKind::Rest, _) => {
                // Nothing to bind.
            }
            // We don't have any information about the type, so attempt to infer it from the pattern directly.
            (_, ty) if matches!(ty, Type::Var(_)) => {
                let pat_ty: &Type = &self.scope.fresh();
                let pat_ty = self.pattern_type(pat, pat_ty);
                self.unify(&pat_ty, &ty, Origin::Pattern(pat.span));
            }

            // All other combinations are type errors.
            (_, ty) => {
                self.error(
                    crate::error::ErrorKind::PatternTypeMismatch {
                        pattern: pat.clone(),
                        expected: ty.clone(),
                    },
                    Origin::Pattern(pat.span),
                );
            }
        }
    }

    /// A helper function to make a best-effort attempt to infer the type of a pattern.
    fn pattern_type(&mut self, pat: &kali_ast::Pattern, var: &Type) -> Type {
        use kali_ast::PatternKind;
        match &pat.kind {
            PatternKind::Literal(lit) => match lit {
                kali_ast::LiteralKind::Integer(_) => Type::Int,
                kali_ast::LiteralKind::Float(_) => Type::Float,
                kali_ast::LiteralKind::Bool(_) => Type::Bool,
                kali_ast::LiteralKind::String(_) => Type::String,
                kali_ast::LiteralKind::Unit => Type::Unit,
            },
            PatternKind::Var(ident) => {
                self.scope.bind_value(ident.key, self.scope.generalise(var));
                var.clone()
            }
            PatternKind::Wildcard => var.clone(),
            PatternKind::Tuple(patterns) => Type::Tuple(
                patterns
                    .iter()
                    .map(|p| {
                        let element = self.scope.fresh();
                        self.pattern_type(p, &element)
                    })
                    .collect(),
            ),
            PatternKind::Record(entries) => {
                let mut ty_entries = IndexMap::new();
                for (name, pattern) in entries.iter() {
                    let entry_ty = self.scope.fresh();
                    ty_entries.insert(name.key, self.pattern_type(pattern, &entry_ty));
                }
                Type::Record(ty_entries)
            }
            PatternKind::Cons { lhs, rhs } => {
                let el_ty = self.scope.fresh();
                self.unify(var, &Type::List(el_ty.clone().into()), Origin::Pattern(pat.span));
                self.pattern_type(rhs, var);
                Type::List(Box::new(self.pattern_type(lhs, &el_ty)))
            }
            PatternKind::EmptyList => Type::List(Box::new(self.scope.fresh())),
            PatternKind::Or { lhs, rhs } => {
                let lhs_ty = self.scope.fresh();
                let rhs_ty = self.scope.fresh();
                let lhs_ty = self.pattern_type(lhs, &lhs_ty);
                let rhs_ty = self.pattern_type(rhs, &rhs_ty);
                self.unify(&lhs_ty, &rhs_ty, Origin::Pattern(pat.span));
                lhs_ty
            }
            PatternKind::Constructor { name: _, payload } => {
                todo!()
            }
            PatternKind::Rest => var.clone(),
        }
    }
}
