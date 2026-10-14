//! Value and type environments for type inference, and the type variables
//! whose levels they define. Everything here is infallible; unification and
//! the AST walk, which can report errors, live on `Infer`.

use std::collections::BTreeMap;

use lasso::Spur;

use crate::types::{Constraint, Scheme, Type, TypeVar};

/// The state of a type variable during inference.
#[derive(Debug)]
enum VarState {
    /// Not yet known. `level` is the scope it was created in, lowered when it
    /// is reached from an outer scope's binding, so generalisation can tell
    /// inner variables from outer ones.
    Unbound { level: usize, constraint: Option<Constraint> },
    /// Known: a pointer to the type it stands for, which may itself contain
    /// variables.
    Bound(Type),
}

/// Stack-based scoping for type inference, with separate scopes for values and
/// types. The type variables live here too, because a variable's level is the
/// depth of the value stack when it was created.
#[derive(Debug, Default)]
pub struct Scope {
    /// Every type variable created so far, indexed by `TypeVar`.
    vars: Vec<VarState>,
    /// The scope stack for values.
    stack: Vec<BTreeMap<Spur, Scheme>>,
    /// The type environment. Unlike values, type definitions/aliases are
    /// global.
    types: BTreeMap<Spur, Type>,
}

impl Scope {
    /// Push a value scope.
    pub(crate) fn enter(&mut self) {
        self.stack.push(BTreeMap::new());
    }

    /// Pop the innermost value scope, returning its bindings.
    pub(crate) fn exit(&mut self) -> BTreeMap<Spur, Scheme> {
        self.stack.pop().unwrap_or_default()
    }

    /// The current scope level, where 0 is the outermost (empty) scope.
    pub(crate) fn level(&self) -> usize {
        self.stack.len()
    }

    // --- Variables ---

    /// Create a new unbound variable at the current scope level.
    pub(crate) fn fresh(&mut self) -> Type {
        self.vars.push(VarState::Unbound { level: self.level(), constraint: None });
        Type::Var(TypeVar(self.vars.len() - 1))
    }

    /// Create a new unbound variable with a constraint.
    pub(crate) fn fresh_constrained(&mut self, constraint: Constraint) -> Type {
        self.vars.push(VarState::Unbound { level: self.level(), constraint: Some(constraint) });
        Type::Var(TypeVar(self.vars.len() - 1))
    }

    /// The level of a variable, if it is unbound.
    pub(crate) fn level_of(&self, var: &TypeVar) -> Option<usize> {
        match &self.vars[var.0] {
            VarState::Unbound { level, .. } => Some(*level),
            VarState::Bound(_) => None,
        }
    }

    /// The constraint on a variable, if it is unbound and has one.
    pub(crate) fn constraint_of(&self, var: &TypeVar) -> Option<&Constraint> {
        match &self.vars[var.0] {
            VarState::Unbound { constraint, .. } => constraint.as_ref(),
            VarState::Bound(_) => None,
        }
    }

    /// The type a variable is bound to, if it is bound.
    pub(crate) fn binding_of(&self, var: &TypeVar) -> Option<&Type> {
        match &self.vars[var.0] {
            VarState::Unbound { .. } => None,
            VarState::Bound(ty) => Some(ty),
        }
    }

    /// Replace every bound variable by what it stands for, all the way down.
    pub(crate) fn zonk(&self, ty: &Type) -> Type {
        ty.map_vars(&mut |var| match self.binding_of(var) {
            None => Type::Var(*var),
            Some(to) => self.zonk(to),
        })
    }

    /// Bind an unbound variable to a type. The caller has done the occurs and
    /// constraint checks. Everything reachable from `ty` now belongs to the
    /// variable's scope, so their levels are lowered to match: an inner scope
    /// must not generalise them.
    pub(crate) fn bind(&mut self, var: &TypeVar, ty: Type) {
        let level = self.level_of(var).expect("binding an already bound variable");
        for inner in self.zonk(&ty).walk_vars() {
            if let VarState::Unbound { level: inner_level, .. } = &mut self.vars[inner.0] {
                *inner_level = (*inner_level).min(level);
            }
        }
        self.vars[var.0] = VarState::Bound(ty);
    }

    /// Put a constraint on an unbound variable. A bound variable is left
    /// alone, since the constraint was checked when it was bound.
    pub(crate) fn constrain(&mut self, var: &TypeVar, new: Constraint) {
        if let VarState::Unbound { constraint, .. } = &mut self.vars[var.0] {
            *constraint = Some(new);
        }
    }

    // --- Schemes ---

    /// Generalise a type into a scheme by quantifying over every unbound
    /// variable that belongs to a scope deeper than the current one.
    pub(crate) fn generalise(&self, ty: &Type) -> Scheme {
        let ty = self.zonk(ty);
        let vars = self.inner_vars(&ty);
        let constraints = vars
            .iter()
            .filter_map(|var| self.constraint_of(var).map(|constraint| (*var, *constraint)))
            .collect();
        Scheme { vars, constraints, ty }
    }

    /// The unbound variables in `ty` above the current level, in order of first
    /// appearance.
    fn inner_vars(&self, ty: &Type) -> Vec<TypeVar> {
        let mut vars = Vec::new();
        for var in ty.walk_vars() {
            if self.level_of(var).is_some_and(|level| level > self.level()) && !vars.contains(var) {
                vars.push(*var);
            }
        }
        vars
    }

    /// Instantiate a scheme, replaicing every quantified variable with a fresh
    /// one while preserving its constraint. Free variables are left alone.
    pub(crate) fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let fresh: BTreeMap<TypeVar, Type> = scheme
            .vars
            .iter()
            .map(|var| {
                let fresh = match scheme.constraint_of(var) {
                    Some(constraint) => self.fresh_constrained(*constraint),
                    None => self.fresh(),
                };
                (*var, fresh)
            })
            .collect();
        scheme.ty.map_vars(&mut |var| fresh.get(var).cloned().unwrap_or(Type::Var(*var)))
    }

    // --- Values ---

    /// Look up a value by name, searching from the innermost scope outwards.
    /// Returns the scheme it is bound to, if any.
    pub fn lookup_value(&self, name: Spur) -> Option<&Scheme> {
        self.stack.iter().rev().find_map(|scope| scope.get(&name))
    }

    /// Bind a value in the innermost scope. Returns the scheme it shadows, if
    /// any.
    pub(crate) fn bind_value(&mut self, name: Spur, scheme: Scheme) -> Option<Scheme> {
        self.stack.last_mut().and_then(|scope| scope.insert(name, scheme))
    }

    // --- Global Types ---

    /// Look up a type by name.
    pub(crate) fn lookup_type(&self, name: Spur) -> Option<&Type> {
        self.types.get(&name)
    }

    /// Bind a type name. Returns the type it replaces, if any.
    pub(crate) fn bind_type(&mut self, name: Spur, ty: Type) -> Option<Type> {
        self.types.insert(name, ty)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use crate::{
        error::Origin,
        infer::Infer,
        types::{Constraint, Type},
    };

    #[test]
    fn binding_lowers_levels() {
        let mut scope = super::Scope::default();
        let outer = scope.fresh();
        scope.enter();
        let inner = scope.fresh();
        // Binding the outer var to a list of the inner var pulls the inner var
        // out to the outer level, so it will not be generalised when the
        // inner scope ends.
        scope.bind(&outer.as_var().unwrap(), Type::List(inner.clone().into()));
        assert_eq!(Some(0), scope.level_of(&inner.as_var().unwrap()));
    }

    #[test]
    fn generalise_quantifies_inner_vars_only() {
        let mut scope = super::Scope::default();

        // Create an outer and inner variable
        let outer = scope.fresh();
        scope.enter();
        let inner = scope.fresh();
        scope.exit();

        // Generalising a function type that uses both variables should only
        // quantify the inner variable.
        let scheme = scope.generalise(&Type::Fn(inner.clone().into(), outer.into()));
        assert_eq!(vec![inner.as_var().unwrap()], scheme.vars);
    }

    #[test]
    fn generalise_skips_vars_unified_with_the_outer_scope() {
        let mut infer = Infer::default();

        // Create an outer and inner variable, then unify them.
        let outer = infer.scope.fresh();
        infer.scope.enter();
        let inner = infer.scope.fresh();
        infer.unify(&inner, &outer, Origin::Empty);
        infer.scope.exit();

        // Generalising the inner variable should not quantify it, since it is
        // unified with the outer variable.
        assert!(infer.scope.generalise(&inner).vars.is_empty());
    }

    #[test]
    fn instantiate_is_fresh() {
        let mut scope = super::Scope::default();

        // Construct a scheme with a numeric constraint on the parameter and a
        // fresh return type.
        scope.enter();
        let param = scope.fresh_constrained(Constraint::Num);
        let return_ty = scope.fresh();
        scope.exit();
        let scheme = scope.generalise(&Type::Fn(param.into(), return_ty.into()));
        assert_eq!(2, scheme.vars.len());

        // Instantiate the scheme twice and check that the two instances are
        // different ...
        let inst1 = scope.instantiate(&scheme);
        let inst2 = scope.instantiate(&scheme);
        assert_ne!(inst1, inst2);

        // ... but that they both satisfy the constraints of the original
        // scheme.
        let Type::Fn(param, _) = &inst1 else { panic!() };
        assert_eq!(Some(&Constraint::Num), scope.constraint_of(&param.as_var().unwrap()));
    }
}
