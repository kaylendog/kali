use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    error::{Error, ErrorKind, Origin},
    types::{Constraint, Scheme, Type, TypeVar},
};

/// The state of a [`VarType``] during inference.
#[derive(Debug)]
enum VarState {
    /// The var is unbound.
    Unbound { level: usize, constraint: Option<Constraint> },
    /// The var is bound an known.
    Bound(Type),
}

impl VarState {
    /// Get the level of the var, if it is unbound.
    pub fn as_unbound(&self) -> Option<(&usize, Option<&Constraint>)> {
        match self {
            VarState::Unbound { level, constraint } => Some((level, constraint.as_ref())),
            VarState::Bound(_) => None,
        }
    }

    /// Get the bound type of the var, if it is bound.
    pub fn as_bound(&self) -> Option<&Type> {
        match self {
            VarState::Unbound { .. } => None,
            VarState::Bound(ty) => Some(ty),
        }
    }
}

/// The inferrence engine state.
#[derive(Debug, Default)]
pub struct Infer {
    /// The state of the various vars.
    vars: Vec<VarState>,
    /// The current scope level. Only variables with level greater than this
    /// value are available.
    level: usize,
    /// Errors collected during inferrence.
    pub errors: Vec<Error>,
}

impl Infer {
    /// Create a new unbound variable.
    fn fresh(&mut self) -> Type {
        self.vars.push(VarState::Unbound { level: self.level, constraint: None });
        Type::Var(TypeVar(self.vars.len() - 1))
    }

    /// Create a new unbound variable with a constraint.
    fn fresh_constrained(&mut self, constraint: Constraint) -> Type {
        self.vars.push(VarState::Unbound { level: self.level, constraint: Some(constraint) });
        Type::Var(TypeVar(self.vars.len() - 1))
    }

    /// Get the level of a type variable, if it is unbound.
    fn level_of(&self, var: &TypeVar) -> Option<usize> {
        match &self.vars[var.0] {
            VarState::Unbound { level, .. } => Some(*level),
            VarState::Bound(_) => None,
        }
    }

    /// Get the constraint of a type variable, if it is unbound.
    fn constraint_of(&self, var: &TypeVar) -> Option<&Constraint> {
        match &self.vars[var.0] {
            VarState::Unbound { constraint, .. } => constraint.as_ref(),
            VarState::Bound(_) => None,
        }
    }

    // --- Resolution ---

    /// Follow a bound variable one hop. Anything else is returned as is.
    fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(var) => match &self.vars[var.0] {
                VarState::Unbound { .. } => ty.clone(),
                VarState::Bound(to) => to.clone(),
            },
            _ => ty.clone(),
        }
    }

    /// Resolve all variables in the given type as far as possible.
    fn zonk(&self, ty: &Type) -> Type {
        match ty {
            Type::Tuple(items) => Type::Tuple(items.iter().map(|item| self.zonk(item)).collect()),
            Type::List(ty) => Type::List(Box::new(self.zonk(ty))),
            Type::Record(entries) => {
                Type::Record(entries.iter().map(|(name, ty)| (*name, self.zonk(ty))).collect())
            }
            Type::Fn(arg, ret) => Type::Fn(self.zonk(arg).into(), self.zonk(ret).into()),
            Type::Var(var) => match &self.vars[var.0] {
                VarState::Unbound { .. } => ty.clone(),
                // Recursively resolve.
                VarState::Bound(to) => self.zonk(to),
            },
            // Primitives can't be resolved.
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Error => {
                ty.clone()
            }
        }
    }

    // --- Unification ---

    /// Unify two types.
    fn unify(&mut self, lhs: &Type, rhs: &Type, origin: Origin) -> Type {
        // Zonk both sides to ensure we are working with the most concrete types
        // possible.
        match (self.zonk(lhs), self.zonk(rhs)) {
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
                // TODO: Should we still unify the common elements if the
                // lengths are different?
                if lhs.len() != rhs.len() {
                    self.errors.push(Error {
                        kind: ErrorKind::Mismatch {
                            expected: Type::Tuple(lhs.clone()),
                            actual: Type::Tuple(rhs.clone()),
                        },
                        origin,
                    });
                    return Type::Error;
                }
                Type::Tuple(
                    lhs.iter()
                        .zip(rhs.iter())
                        .map(|(lhs, rhs)| self.unify(lhs, rhs, origin.clone()))
                        .collect(),
                )
            }
            (Type::List(lhs), Type::List(rhs)) => {
                Type::List(self.unify(&lhs, &rhs, origin.clone()).into())
            }
            // Records are structural with exact field sets.
            (Type::Record(lhs), Type::Record(rhs)) => {
                if lhs.len() != rhs.len() || lhs.keys().any(|key| !rhs.contains_key(key)) {
                    self.errors.push(Error {
                        kind: ErrorKind::Mismatch {
                            expected: Type::Record(lhs),
                            actual: Type::Record(rhs),
                        },
                        origin,
                    });
                    return Type::Error;
                }
                Type::Record(
                    lhs.iter()
                        .map(|(key, lhs)| (*key, self.unify(lhs, &rhs[key], origin.clone())))
                        .collect(),
                )
            }
            (Type::Fn(lhs_param, lhs_ret), Type::Fn(rhs_param, rhs_ret)) => Type::Fn(
                self.unify(&lhs_param, &rhs_param, origin.clone()).into(),
                self.unify(&lhs_ret, &rhs_ret, origin.clone()).into(),
            ),
            // Unifying two unbound vars should bind the greater to the lesser. These are
            // necessarily unbound since we zonked them earlier.
            (Type::Var(lhs), Type::Var(rhs)) => {
                // Check we aren't unifying the same var with itself.
                if lhs == rhs {
                    return Type::Var(lhs.clone());
                }

                // Preserve the var with the lower level, as it is more general.
                let (keep, drop) = match self.level_of(&lhs).cmp(&self.level_of(&rhs)) {
                    Ordering::Less | Ordering::Equal => (lhs, rhs),
                    Ordering::Greater => (rhs, lhs),
                };

                // Merge the constraints of the two vars, if any.
                match (self.constraint_of(&keep), self.constraint_of(&drop)) {
                    (Some(keep_constraint), Some(drop_constraint)) => {
                        if keep_constraint != drop_constraint {
                            self.errors.push(Error {
                                kind: ErrorKind::ConstraintViolation {
                                    var: drop.clone(),
                                    constraint: *drop_constraint,
                                    actual: Type::Var(keep.clone()),
                                },
                                origin,
                            });
                            return Type::Error;
                        }
                    }
                    (None, Some(drop_constraint)) => {
                        self.vars[keep.0] = VarState::Unbound {
                            level: self.level_of(&keep).unwrap(),
                            constraint: Some(*drop_constraint),
                        };
                    }
                    _ => {}
                }

                // Bind the dropped var to the kept var.
                self.bind(&drop, Type::Var(keep), origin)
            }
            // Unifying concrete types with a var binds them.
            (Type::Var(var), actual) | (actual, Type::Var(var)) => {
                self.bind(&var, actual.clone(), origin)
            }
            // Propagate existing errors upwards.
            (Type::Error, _) | (_, Type::Error) => Type::Error,
            // Add mismatches to the list.
            (lhs, rhs) => {
                self.errors.push(Error {
                    kind: ErrorKind::Mismatch { expected: lhs.clone(), actual: rhs.clone() },
                    origin,
                });
                Type::Error
            }
        }
    }

    /// Bind a type variable to a concrete type, performing the occurs check and
    /// constraint satisfaction.
    fn bind(&mut self, var: &TypeVar, concrete: Type, origin: Origin) -> Type {
        // Check for infinite types.
        if self.occurs(var, &concrete) {
            self.errors.push(Error { kind: ErrorKind::InfiniteType, origin });
            return Type::Error;
        }

        // Check constraints are satisfied, if any.
        if let VarState::Unbound { constraint: Some(constraint), .. } = &self.vars[var.0] {
            if !concrete.satisfies(constraint) {
                self.errors.push(Error {
                    kind: ErrorKind::ConstraintViolation {
                        var: var.clone(),
                        constraint: *constraint,
                        actual: concrete.clone(),
                    },
                    origin,
                });
                return Type::Error;
            }
        }

        // Everything reachable from `concrete` now lives in this variable's
        // scope.
        let level = self.level_of(var).expect("unify zonks, so a bound var never gets here");
        self.adjust_level(&concrete, level);

        // Bind and celebrate!!
        self.vars[var.0] = VarState::Bound(concrete.clone());
        concrete
    }

    /// Check if a type variable occurs in a type, which would indicate an
    /// infinite type.
    fn occurs(&self, var: &TypeVar, ty: &Type) -> bool {
        match ty {
            // Base cases
            Type::Var(other) if other == var => true,
            Type::Int
            | Type::Float
            | Type::Bool
            | Type::String
            | Type::Unit
            | Type::Error
            | Type::Var(_) => false,
            // Constructed types recur into their parameters.
            Type::Tuple(items) => items.iter().any(|item| self.occurs(var, item)),
            Type::List(ty) => self.occurs(var, ty),
            Type::Record(entries) => entries.values().any(|item| self.occurs(var, item)),
            Type::Fn(param, ret) => self.occurs(var, param) || self.occurs(var, ret),
        }
    }

    /// Lower every unbound variable in `ty` to at most `new_level`. Called when
    /// a variable is bound to `ty`: anything reachable from it now belongs
    /// to the binder's scope, so it must not be generalised by an inner
    /// scope.
    fn adjust_level(&mut self, ty: &Type, new_level: usize) {
        match ty {
            Type::Tuple(items) => items.iter().for_each(|item| self.adjust_level(item, new_level)),
            Type::List(ty) => self.adjust_level(ty, new_level),
            Type::Record(entries) => {
                entries.values().for_each(|ty| self.adjust_level(ty, new_level))
            }
            Type::Fn(arg, ret) => {
                self.adjust_level(arg, new_level);
                self.adjust_level(ret, new_level);
            }
            Type::Var(var) => match &mut self.vars[var.0] {
                VarState::Unbound { level, .. } => *level = (*level).min(new_level),
                VarState::Bound(to) => {
                    let to = to.clone();
                    self.adjust_level(&to, new_level);
                }
            },
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Error => {}
        }
    }

    // --- Schemes ---

    /// Enter a new scope level. This is used to track which variables are
    fn enter(&mut self) {
        self.level += 1;
    }

    /// Exit the current scope level. This is used to track which variables are
    fn exit(&mut self) {
        self.level -= 1;
    }

    /// Generalise a type into a scheme by quantifying over all unbound
    /// variables
    fn generalise(&mut self, ty: &Type) -> Scheme {
        let ty = self.zonk(ty);

        // Walk the type and collect all unbound variables that are at a level
        // greater than the current level.
        let mut vars = Vec::new();
        self.collect_vars(&ty, &mut vars);

        // Collect the constraints of the unbound variables.
        let constraints = vars
            .iter()
            .filter_map(|var| self.constraint_of(var).map(|constraint| (var.clone(), *constraint)))
            .collect();

        Scheme { vars, constraints, ty }
    }

    /// Recursively collect all unbound variables in `ty` that are at a level
    /// greater than the current level.
    fn collect_vars(&mut self, ty: &Type, vars: &mut Vec<TypeVar>) {
        match ty {
            Type::Tuple(items) => items.iter().for_each(|item| self.collect_vars(item, vars)),
            Type::List(ty) => self.collect_vars(ty, vars),
            Type::Record(entries) => entries.values().for_each(|ty| self.collect_vars(ty, vars)),
            Type::Fn(arg, ret) => {
                self.collect_vars(arg, vars);
                self.collect_vars(ret, vars);
            }
            Type::Var(var) => {
                if let Some(level) = self.level_of(var) {
                    // `level_of` only returns Some if the var is unbound, so we
                    // don't need to check that here.
                    if level > self.level && !vars.contains(var) {
                        vars.push(var.clone());
                    }
                }
            }
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Error => {}
        }
    }

    /// Instantiate a scheme by replacing its quantified variables with fresh
    /// unbound variables.
    fn instantiate(&mut self, scheme: &Scheme) -> Type {
        // Create a mapping from the scheme's vars to fresh unbound vars.
        let mut vars = BTreeMap::new();
        for var in &scheme.vars {
            // Apply the constraint of the scheme's var to the fresh var, if
            // any.
            let fresh = if let Some(constraint) = scheme.constraint_of(var) {
                self.fresh_constrained(*constraint)
            } else {
                self.fresh()
            };
            vars.insert(var.clone(), fresh);
        }

        // Replace the scheme's vars in the type with the fresh vars.
        self.replace_vars(&scheme.ty, &vars)
    }

    /// Recursively replace variables in `ty` according to `vars`.
    fn replace_vars(&mut self, ty: &Type, vars: &BTreeMap<TypeVar, Type>) -> Type {
        match ty {
            // Base cases
            Type::Var(var) => vars.get(var).cloned().unwrap_or_else(|| Type::Var(var.clone())),
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Error => {
                ty.clone()
            }
            // Composite types
            Type::Tuple(items) => {
                Type::Tuple(items.iter().map(|item| self.replace_vars(item, vars)).collect())
            }
            Type::List(ty) => Type::List(Box::new(self.replace_vars(ty, vars))),
            Type::Record(entries) => Type::Record(
                entries.iter().map(|(name, ty)| (*name, self.replace_vars(ty, vars))).collect(),
            ),
            Type::Fn(arg, ret) => {
                Type::Fn(self.replace_vars(arg, vars).into(), self.replace_vars(ret, vars).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

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
        let unknown = infer.fresh();
        assert_eq!(
            Type::List(Type::Bool.into()),
            infer.unify(&Type::List(unknown.into()), &Type::List(Type::Bool.into()), Origin::Empty)
        );
    }

    #[test]
    fn unify_vars() {
        // Unifying two unbound vars should bind the greater to the lesser.
        let mut infer = Infer::default();
        let unknown1 = infer.fresh();
        let unknown2 = infer.fresh();
        assert_eq!(
            Type::Var(unknown1.as_var().unwrap()),
            infer.unify(&unknown1, &unknown2, Origin::Empty)
        );
        // They are now one variable: binding either binds both.
        infer.unify(&unknown1, &Type::Int, Origin::Empty);
        assert_eq!(Type::Int, infer.zonk(&unknown2));
        assert!(infer.errors.is_empty());
    }

    #[test]
    fn unify_vars_merges_constraints() {
        let mut infer = Infer::default();
        let plain = infer.fresh();
        let numeric = infer.fresh_constrained(Constraint::Num);
        infer.unify(&plain, &numeric, Origin::Empty);
        assert!(infer.errors.is_empty());
        // The survivor carries the constraint.
        assert_eq!(Type::Error, infer.unify(&plain, &Type::Bool, Origin::Empty));
        assert_eq!(1, infer.errors.len());
    }

    #[test]
    fn unify_var_with_itself_binds_nothing() {
        let mut infer = Infer::default();
        let unknown = infer.fresh();
        assert_eq!(unknown, infer.unify(&unknown, &unknown, Origin::Empty));
        assert!(infer.errors.is_empty());
    }

    #[test]
    fn unify_error_is_silent() {
        let mut infer = Infer::default();
        let numeric = infer.fresh_constrained(Constraint::Num);
        assert_eq!(Type::Error, infer.unify(&Type::Error, &Type::Int, Origin::Empty));
        assert_eq!(Type::Error, infer.unify(&numeric, &Type::Error, Origin::Empty));
        assert!(infer.errors.is_empty());
    }

    #[test]
    fn unify_infinite_type() {
        let mut infer = Infer::default();
        let unknown = infer.fresh();
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
        let unknown = infer.fresh();
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
    fn binding_lowers_levels() {
        let mut infer = Infer::default();
        let outer = infer.fresh();
        infer.level += 1;
        let inner = infer.fresh();
        // Binding the outer var to a list of the inner var pulls the inner var
        // out to the outer level, so it will not be generalised when
        // the inner scope ends.
        infer.unify(&outer, &Type::List(inner.clone().into()), Origin::Empty);
        assert_eq!(Some(0), infer.level_of(&inner.as_var().unwrap()));
    }

    #[test]
    fn unify_constraints() {
        let mut infer = Infer::default();
        // Unifying a numeric constraint with a numeric type should succeed.
        let unknown = infer.fresh_constrained(Constraint::Num);
        assert_eq!(Type::Int, infer.unify(&unknown, &Type::Int, Origin::Empty));

        // Unifying a numeric constraint with a non-numeric type should fail.
        let mut infer = Infer::default();
        let unknown = infer.fresh_constrained(Constraint::Num);

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
    fn unify_var_then_resolve() {
        // Unifying a variable with a concrete type should bind the variable to
        // that type.
        let mut infer = Infer::default();
        let unknown = infer.fresh();
        assert_eq!(Type::Int, infer.unify(&unknown, &Type::Int, Origin::Empty));
        assert_eq!(Type::Int, infer.resolve(&unknown));

        // Zonking recursively resolves variables to their concrete types.
        let mut infer = Infer::default();
        let unknown1 = infer.fresh();
        let unknown2 = infer.fresh();
        assert_eq!(Type::Int, infer.unify(&unknown1, &Type::Int, Origin::Empty));
        assert_eq!(Type::Int, infer.unify(&unknown2, &unknown1, Origin::Empty));
        assert_eq!(Type::Int, infer.zonk(&unknown2));
    }

    #[test]
    fn generalise_quantifies_inner_vars_only() {
        let mut infer = Infer::default();

        // Create an outer and inner variable
        let outer = infer.fresh();
        infer.enter();
        let inner = infer.fresh();
        infer.exit();

        // Generalising a function type that uses both variables should only quantify the inner variable.
        let scheme = infer.generalise(&Type::Fn(inner.clone().into(), outer.clone().into()));
        assert_eq!(vec![inner.as_var().unwrap()], scheme.vars);
    }

    #[test]
    fn generalise_skips_vars_unified_with_the_outer_scope() {
        let mut infer = Infer::default();

        // Create an outer and inner variable, then unify them.
        let outer = infer.fresh();
        infer.enter();
        let inner = infer.fresh();
        infer.unify(&inner, &outer, Origin::Empty);
        infer.exit();

        // Generalising the inner variable should not quantify it, since it is unified with the outer variable.
        assert!(infer.generalise(&inner).vars.is_empty());
    }

    #[test]
    fn instansiate_is_fresh() {
        let mut infer = Infer::default();

        // Construct a scheme with a numeric constraint on the parameter and a
        // fresh return type.
        infer.enter();
        let param = infer.fresh_constrained(Constraint::Num);
        let return_ty = infer.fresh();
        infer.exit();
        let scheme = infer.generalise(&Type::Fn(param.into(), return_ty.into()));
        assert_eq!(2, scheme.vars.len());

        // Instantiate the scheme twice and check that the two instances are
        // different ...
        let inst1 = infer.instantiate(&scheme);
        let inst2 = infer.instantiate(&scheme);
        assert_ne!(inst1, inst2);

        // ... but that they both satisfy the constraints of the original
        // scheme.
        let Type::Fn(param, _) = &inst1 else { panic!() };
        assert_eq!(Some(&Constraint::Num), infer.constraint_of(&param.as_var().unwrap()));
    }
}
