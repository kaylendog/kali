use indexmap::IndexMap;

use crate::{error::Origin, Infer, Type};

impl Infer {
    /// A helper function to make a best-effort attempt to infer the type of a pattern.
    pub(crate) fn pattern(&mut self, pat: &kali_ast::Pattern, input_ty: &Type) -> Type {
        use kali_ast::PatternKind;
        let pat_ty = match &pat.kind {
            PatternKind::Literal(lit) => match lit {
                kali_ast::LiteralKind::Integer(_) => Type::Int,
                kali_ast::LiteralKind::Float(_) => Type::Float,
                kali_ast::LiteralKind::Bool(_) => Type::Bool,
                kali_ast::LiteralKind::String(_) => Type::String,
                kali_ast::LiteralKind::Unit => Type::Unit,
            },
            PatternKind::Var(ident) => {
                self.scope.bind_value(ident.key, self.scope.generalise(input_ty));
                input_ty.clone()
            }
            PatternKind::Wildcard => input_ty.clone(),
            PatternKind::Tuple(patterns) => Type::Tuple(
                patterns
                    .iter()
                    .map(|p| {
                        let element = self.scope.fresh();
                        self.pattern(p, &element)
                    })
                    .collect(),
            ),
            PatternKind::Record(entries) => {
                let mut ty_entries = IndexMap::new();
                for (name, pattern) in entries.iter() {
                    let entry_ty = self.scope.fresh();
                    ty_entries.insert(name.key, self.pattern(pattern, &entry_ty));
                }
                Type::Record(ty_entries)
            }
            PatternKind::Cons { lhs, rhs } => {
                let el_ty = self.scope.fresh();
                self.unify(input_ty, &Type::List(el_ty.clone().into()), Origin::Pattern(pat.span));
                self.pattern(rhs, input_ty);
                Type::List(Box::new(self.pattern(lhs, &el_ty)))
            }
            PatternKind::EmptyList => Type::List(Box::new(self.scope.fresh())),
            PatternKind::Or { lhs, rhs } => {
                let pat_lhs_ty = self.scope.fresh();
                let pat_rhs_ty = self.scope.fresh();
                let pat_lhs_ty = self.pattern(lhs, &pat_lhs_ty);
                let pat_rhs_ty = self.pattern(rhs, &pat_rhs_ty);

                self.unify(&pat_lhs_ty, &pat_rhs_ty, Origin::Pattern(pat.span));

                pat_lhs_ty
            }
            PatternKind::Constructor { name: _, payload } => {
                todo!()
            }
            PatternKind::Rest => input_ty.clone(),
        };

        self.unify(&pat_ty, &input_ty, Origin::Pattern(pat.span))
    }
}
