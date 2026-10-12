use crate::{
    error::{Error, ErrorKind, Origin},
    infer::Infer,
    types::Type,
};

impl Infer {
    /// Converts a `kali_ast::Type` into a `Type` in the type system.
    fn infer_type_expr(&mut self, ty: &kali_ast::Type) -> Type {
        use kali_ast::{PrimitiveTypeKind, TypeKind};
        match &ty.kind {
            TypeKind::Primitive(primitive) => match primitive {
                PrimitiveTypeKind::Integer => Type::Int,
                PrimitiveTypeKind::Float => Type::Float,
                PrimitiveTypeKind::Bool => Type::Bool,
                PrimitiveTypeKind::String => Type::String,
                PrimitiveTypeKind::Unit => Type::Unit,
            },
            TypeKind::Named(ident) => {
                let Some(ty) = self.lookup(ident.key).cloned() else {
                    self.errors.push(Error {
                        kind: ErrorKind::UnknownName(ident.key),
                        origin: Origin::Name(ident.span),
                    });
                    return Type::Error;
                };
                ty
            }
            TypeKind::Tuple(items) => {
                Type::Tuple(items.into_iter().map(|item| self.infer_type_expr(&item)).collect())
            }
            TypeKind::List(item) => Type::List(self.infer_type_expr(&item).into()),
            TypeKind::Record(entries) => Type::Record(
                entries
                    .into_iter()
                    .map(|(name, ty)| (name.key, self.infer_type_expr(&ty)))
                    .collect(),
            ),
            TypeKind::Fn(params, ret) => {
                params.iter().rev().fold(self.infer_type_expr(ret), |acc, param| {
                    Type::Fn(self.infer_type_expr(param).into(), acc.into())
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use kali_parse::kali_unwrap;
    use lasso::{Key, Spur};
    use pretty_assertions::assert_eq;

    use crate::{error::Error, infer::Infer, types::Type};

    /// Returns the type expression held by the last item in the given module, which is expected to be a type alias.
    fn type_expr_of(module: &kali_ast::Module, idx: usize) -> kali_ast::Type {
        let kali_ast::ItemKind::TypeAlias(ref item) = module
            .items
            .iter()
            .filter(|item| matches!(item.kind, kali_ast::ItemKind::TypeAlias(_)))
            .nth(idx)
            .expect("module does not contain enough type aliases")
            .kind
        else {
            panic!("last item is not a type alias");
        };
        item.ty.clone()
    }

    /// Returns a `Spur` from a `usize`, panicking if the conversion fails.
    pub fn spur(usize: usize) -> Spur {
        Spur::try_from_usize(usize).unwrap()
    }

    #[test]
    fn infer_type_expr() {
        let module = kali_unwrap! {
            type P = { x: int };
            type Q = [P];
        };

        let mut infer = Infer::default();

        let p_ty = infer.infer_type_expr(&type_expr_of(&module, 0));
        // We need to manually insert the alias, since we're not actually running the full inference.
        infer.aliases.insert(spur(0), p_ty.clone());
        let q_ty = infer.infer_type_expr(&type_expr_of(&module, 1));

        assert_eq!(Vec::<Error>::default(), infer.errors);
        assert_eq!(p_ty, Type::Record([(spur(1), Type::Int)].into()));
        assert_eq!(q_ty, Type::List(Type::Record([(spur(1), Type::Int)].into()).into()));
    }
}
