//! Unification of types during type inference.

use std::cmp::Ordering;

use crate::{
    error::{ErrorKind, Origin},
    infer::Infer,
    types::{Type, TypeVar},
};

impl Infer {
    /// Unify two types, returning the unified type. Failures are reported in
    /// `self.errors` and bubble up as `Type::Error`.
    pub(crate) fn unify(&mut self, lhs: &Type, rhs: &Type, origin: Origin) -> Type {
        // Zonk both sides so bound variables never reach the match below.
        match (self.scope.zonk(lhs), self.scope.zonk(rhs)) {
            // Primitive types unify if they are exact.
            (lhs, rhs)
                if lhs == rhs
                    && matches!(
                        lhs,
                        Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit
                    ) =>
            {
                lhs
            }
            // Constructed types unify if their parameters unify.
            (Type::Tuple(lhs), Type::Tuple(rhs)) => {
                if lhs.len() != rhs.len() {
                    let expected = Type::Tuple(lhs);
                    let actual = Type::Tuple(rhs);
                    return self.error(ErrorKind::Mismatch { expected, actual }, origin);
                }
                Type::Tuple(
                    lhs.iter()
                        .zip(rhs.iter())
                        .map(|(lhs, rhs)| self.unify(lhs, rhs, origin.clone()))
                        .collect(),
                )
            }
            (Type::List(lhs), Type::List(rhs)) => Type::List(self.unify(&lhs, &rhs, origin).into()),
            // Records are structural with exact field sets.
            (Type::Record(lhs), Type::Record(rhs)) => {
                if lhs.len() != rhs.len() || lhs.keys().any(|key| !rhs.contains_key(key)) {
                    let expected = Type::Record(lhs);
                    let actual = Type::Record(rhs);
                    return self.error(ErrorKind::Mismatch { expected, actual }, origin);
                }
                Type::Record(
                    lhs.iter()
                        .map(|(key, lhs)| (*key, self.unify(lhs, &rhs[key], origin.clone())))
                        .collect(),
                )
            }
            (Type::Fn(lhs_param, lhs_ret), Type::Fn(rhs_param, rhs_ret)) => Type::Fn(
                self.unify(&lhs_param, &rhs_param, origin.clone()).into(),
                self.unify(&lhs_ret, &rhs_ret, origin).into(),
            ),
            // Two unbound variables become one. Outer scope wins, and constraints are merged.
            (Type::Var(lhs), Type::Var(rhs)) => {
                if lhs == rhs {
                    return Type::Var(lhs);
                }
                let (keep, drop) = match self.scope.level_of(&lhs).cmp(&self.scope.level_of(&rhs)) {
                    Ordering::Less | Ordering::Equal => (lhs, rhs),
                    Ordering::Greater => (rhs, lhs),
                };
                // Merge the constraints onto the survivor.
                match (self.scope.constraint_of(&keep), self.scope.constraint_of(&drop)) {
                    (Some(kept), Some(dropped)) if kept != dropped => {
                        let (constraint, actual) = (*dropped, Type::Var(keep));
                        return self.error(
                            ErrorKind::ConstraintViolation { var: drop, constraint, actual },
                            origin,
                        );
                    }
                    (None, Some(dropped)) => self.scope.constrain(&keep, *dropped),
                    _ => {}
                }
                self.bind_var(&drop, Type::Var(keep), origin)
            }
            // A variable against a concrete type binds it
            (Type::Var(var), actual) | (actual, Type::Var(var)) => {
                self.bind_var(&var, actual, origin)
            }
            // An error was already reported for one side.
            (Type::Error, _) | (_, Type::Error) => Type::Error,
            (expected, actual) => self.error(ErrorKind::Mismatch { expected, actual }, origin),
        }
    }

    /// Bind an unbound variable to `concrete`, making sure the occurs check and
    /// constraint checks pass. `concrete` is already zonked by `unify`.
    fn bind_var(&mut self, var: &TypeVar, concrete: Type, origin: Origin) -> Type {
        // Occurs check: binding `'a` to a type containing `'a` is infinite.
        if concrete.walk_vars().any(|other| other == var) {
            return self.error(ErrorKind::InfiniteType, origin);
        }
        if let Some(constraint) = self.scope.constraint_of(var).copied() {
            if !concrete.satisfies(&constraint) {
                let var = *var;
                return self.error(
                    ErrorKind::ConstraintViolation { var, constraint, actual: concrete },
                    origin,
                );
            }
        }
        self.scope.bind(var, concrete.clone());
        concrete
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        error::{Error, ErrorKind, Origin},
        infer::Infer,
        types::{Constraint, Type},
    };

    #[test]
    fn unify_literals() {
        // Unifying the same primitive type should succeed.
        assert_eq!(Type::Int, Infer::default().unify(&Type::Int, &Type::Int, Origin::Empty));

        // Unifying different primitive types should fail.
        let mut infer = Infer::default();
        assert_eq!(Type::Error, infer.unify(&Type::Int, &Type::Bool, Origin::Empty));
        assert_eq!(
            vec![Error {
                kind: ErrorKind::Mismatch { expected: Type::Int, actual: Type::Bool },
                origin: Origin::Empty
            }],
            infer.errors,
        );
    }

    #[test]
    fn unify_constructors() {
        let mut infer = Infer::default();
        let unknown = infer.scope.fresh();
        assert_eq!(
            Type::List(Type::Bool.into()),
            infer.unify(&Type::List(unknown.into()), &Type::List(Type::Bool.into()), Origin::Empty)
        );
    }

    #[test]
    fn unify_vars() {
        // Unifying two unbound vars should bind the greater to the lesser.
        let mut infer = Infer::default();
        let unknown1 = infer.scope.fresh();
        let unknown2 = infer.scope.fresh();
        assert_eq!(
            Type::Var(unknown1.as_var().unwrap()),
            infer.unify(&unknown1, &unknown2, Origin::Empty)
        );
        // They are now one variable: binding either binds both.
        infer.unify(&unknown1, &Type::Int, Origin::Empty);
        assert_eq!(Type::Int, infer.scope.zonk(&unknown2));
        assert!(infer.errors.is_empty());
    }

    #[test]
    fn unify_vars_merges_constraints() {
        let mut infer = Infer::default();
        let plain = infer.scope.fresh();
        let numeric = infer.scope.fresh_constrained(Constraint::Num);
        infer.unify(&plain, &numeric, Origin::Empty);
        assert!(infer.errors.is_empty());
        // The survivor carries the constraint.
        assert_eq!(Type::Error, infer.unify(&plain, &Type::Bool, Origin::Empty));
        assert_eq!(1, infer.errors.len());
    }

    #[test]
    fn unify_var_with_itself_binds_nothing() {
        let mut infer = Infer::default();
        let unknown = infer.scope.fresh();
        assert_eq!(unknown, infer.unify(&unknown, &unknown, Origin::Empty));
        assert!(infer.errors.is_empty());
    }

    #[test]
    fn unify_error_is_silent() {
        let mut infer = Infer::default();
        let numeric = infer.scope.fresh_constrained(Constraint::Num);
        assert_eq!(Type::Error, infer.unify(&Type::Error, &Type::Int, Origin::Empty));
        assert_eq!(Type::Error, infer.unify(&numeric, &Type::Error, Origin::Empty));
        assert!(infer.errors.is_empty());
    }

    #[test]
    fn unify_infinite_type() {
        let mut infer = Infer::default();
        let unknown = infer.scope.fresh();
        let list = Type::List(unknown.clone().into());
        assert_eq!(Type::Error, infer.unify(&unknown, &list, Origin::Empty));
        assert_eq!(
            vec![Error { kind: ErrorKind::InfiniteType, origin: Origin::Empty }],
            infer.errors
        );
    }

    #[test]
    fn unify_records_need_the_same_fields() {
        let mut rodeo = lasso::Rodeo::default();
        let x = rodeo.get_or_intern("x");
        let y = rodeo.get_or_intern("y");
        let record = |fields: Vec<(lasso::Spur, Type)>| Type::Record(fields.into_iter().collect());

        let mut infer = Infer::default();
        let unknown = infer.scope.fresh();
        assert_eq!(
            record(vec![(x, Type::Int)]),
            infer.unify(&record(vec![(x, unknown)]), &record(vec![(x, Type::Int)]), Origin::Empty)
        );
        assert_eq!(
            Type::Error,
            infer.unify(
                &record(vec![(x, Type::Int)]),
                &record(vec![(y, Type::Int)]),
                Origin::Empty
            )
        );
        assert_eq!(1, infer.errors.len());
    }

    #[test]
    fn unify_constraints() {
        let mut infer = Infer::default();
        // Unifying a numeric constraint with a numeric type should succeed.
        let unknown = infer.scope.fresh_constrained(Constraint::Num);
        assert_eq!(Type::Int, infer.unify(&unknown, &Type::Int, Origin::Empty));

        // Unifying a numeric constraint with a non-numeric type should fail.
        let mut infer = Infer::default();
        let unknown = infer.scope.fresh_constrained(Constraint::Num);

        assert_eq!(Type::Error, infer.unify(&unknown, &Type::Bool, Origin::Empty));
        assert_eq!(
            vec![Error {
                kind: ErrorKind::ConstraintViolation {
                    var: unknown.as_var().unwrap(),
                    constraint: Constraint::Num,
                    actual: Type::Bool
                },
                origin: Origin::Empty
            }],
            infer.errors,
        );
    }

    #[test]
    fn unify_var_then_zonk() {
        // Unifying a variable with a concrete type should bind the variable to
        // that type.
        let mut infer = Infer::default();
        let unknown = infer.scope.fresh();
        assert_eq!(Type::Int, infer.unify(&unknown, &Type::Int, Origin::Empty));
        assert_eq!(Type::Int, infer.scope.zonk(&unknown));

        // Zonking recursively resolves variables to their concrete types.
        let mut infer = Infer::default();
        let unknown1 = infer.scope.fresh();
        let unknown2 = infer.scope.fresh();
        assert_eq!(Type::Int, infer.unify(&unknown1, &Type::Int, Origin::Empty));
        assert_eq!(Type::Int, infer.unify(&unknown2, &unknown1, Origin::Empty));
        assert_eq!(Type::Int, infer.scope.zonk(&unknown2));
    }
}
