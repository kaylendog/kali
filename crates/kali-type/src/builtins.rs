//! Schemes for the built-in operators.

use kali_ast::{BinaryOpKind, UnaryOpKind};

use crate::types::{Constraint, Scheme, Type, TypeVar};

/// The one quantified variable an operator scheme can have.
const A: TypeVar = TypeVar(0);

fn a() -> Type {
    Type::Var(A)
}

/// `param -> ret`.
fn fn1(param: Type, ret: Type) -> Type {
    Type::Fn(param.into(), ret.into())
}

/// `first -> second -> ret`, curried.
fn fn2(first: Type, second: Type, ret: Type) -> Type {
    fn1(first, fn1(second, ret))
}

/// A scheme with no quantified variables.
fn mono(ty: Type) -> Scheme {
    Scheme { vars: Vec::new(), constraints: Vec::new(), ty }
}

/// A scheme quantified over `'a`, with the given constraint on it.
fn poly(constraint: Option<Constraint>, ty: Type) -> Scheme {
    let constraints = constraint.map(|constraint| (A, constraint)).into_iter().collect();
    Scheme { vars: vec![A], constraints, ty }
}

/// The scheme of a binary operator.
pub fn binary(op: BinaryOpKind) -> Scheme {
    use BinaryOpKind::*;
    match op {
        // num 'a => 'a -> 'a -> 'a
        Add | Subtract | Multiply | Divide | Modulo | Exponentiate => {
            poly(Some(Constraint::Num), fn2(a(), a(), a()))
        }
        // 'a -> 'a -> bool: structural equality and ordering on any type, as in OCaml.
        Equal | NotEqual | LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => {
            poly(None, fn2(a(), a(), Type::Bool))
        }
        LogicalAnd | LogicalOr => mono(fn2(Type::Bool, Type::Bool, Type::Bool)),
        // 'a -> ['a] -> ['a]
        Cons => poly(None, fn2(a(), Type::List(a().into()), Type::List(a().into()))),
        // ['a] -> ['a] -> ['a]
        Concat => {
            let list = || Type::List(a().into());
            poly(None, fn2(list(), list(), list()))
        }
        // Not parsed yet. Typed on `int` so they are honest if they ever appear.
        BitwiseAnd | BitwiseOr | BitwiseXor | BitwiseShiftLeft | BitwiseShiftRight => {
            mono(fn2(Type::Int, Type::Int, Type::Int))
        }
    }
}

/// The scheme of a unary operator.
pub fn unary(op: UnaryOpKind) -> Scheme {
    use UnaryOpKind::*;
    match op {
        Negate | UnaryPlus => poly(Some(Constraint::Num), fn1(a(), a())),
        LogicalNot => mono(fn1(Type::Bool, Type::Bool)),
        BitwiseNot => mono(fn1(Type::Int, Type::Int)),
    }
}

#[cfg(test)]
mod tests {
    use kali_ast::BinaryOpKind;
    use pretty_assertions::{assert_eq, assert_ne};

    use crate::{builtins, infer::Infer, types::Constraint};

    #[test]
    fn instantiating_an_operator_is_fresh_and_keeps_its_constraint() {
        let mut infer = Infer::default();
        let first = infer.scope.instantiate(&builtins::binary(BinaryOpKind::Add));
        let second = infer.scope.instantiate(&builtins::binary(BinaryOpKind::Add));
        assert_ne!(first, second);

        let crate::types::Type::Fn(param, _) = &first else { panic!("expected a function") };
        assert_eq!(Some(&Constraint::Num), infer.scope.constraint_of(&param.as_var().unwrap()));
    }

    #[test]
    fn comparison_is_unconstrained() {
        let mut infer = Infer::default();
        let ty = infer.scope.instantiate(&builtins::binary(BinaryOpKind::Equal));
        let crate::types::Type::Fn(param, _) = &ty else { panic!("expected a function") };
        assert_eq!(None, infer.scope.constraint_of(&param.as_var().unwrap()));
    }
}
