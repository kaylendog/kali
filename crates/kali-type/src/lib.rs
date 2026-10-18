//! Kali's type checker, implemented as a Hindley-Milner type inference engine
//! with let-polymorphism.

mod builtins;
mod cache;
mod error;
mod infer;
mod scope;
mod types;
mod unify;

pub use error::{Error, ErrorKind};
pub use infer::Infer;
pub use scope::Scope;
pub use types::{Scheme, Type};

use crate::error::Origin;

/// Type check a module, returning the inference state and any errors
/// encountered.
pub fn check(module: &kali_ast::Module) -> Infer {
    let mut infer = Infer::default();
    infer.scope.enter();

    for def in module.items.iter().filter_map(|item| item.kind.as_definition()) {
        infer.scope.enter();
        let pattern_ty = infer.scope.fresh();
        let pattern_ty = infer.pattern(&def.name, &pattern_ty);
        let body = infer.expr(&def.expr);
        infer.unify(
            &pattern_ty,
            &body,
            Origin::Definition { name: def.name.span, value: def.expr.span },
        );
        for (name, scheme) in infer.scope.exit() {
            let scheme = infer.scope.generalise(&scheme.ty);
            infer.scope.bind_value(name, scheme);
        }
    }

    infer
}
