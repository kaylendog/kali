//! Recursive resolution of types.

use std::collections::BTreeMap;

use crate::{Context, Type, TypeInferenceError, TypeIterator};

impl Type {
    /// Consume and attempt to resolve the type.
    pub fn resolve(self, context: &mut Context) -> Result<Type, TypeInferenceError> {
        Ok(match self {
            Type::Array(x) => Type::Array(Box::new(x.resolve(context)?)),
            Type::Tuple(types) => Type::Tuple(
                types
                    .into_iter()
                    .map_resolve(context)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Type::Record(types) => {
                let mut fields = BTreeMap::new();
                for (name, ty) in types {
                    fields.insert(name, ty.resolve(context)?);
                }
                Type::Record(fields)
            }
            Type::Parameterized(name, types) => Type::Parameterized(
                name,
                types
                    .into_iter()
                    .map_resolve(context)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Type::Lambda(params, body) => Type::Lambda(
                params
                    .into_iter()
                    .map_resolve(context)
                    .collect::<Result<Vec<_>, _>>()?,
                Box::new(body.resolve(context)?),
            ),
            Type::Infer(x) => match context.get_inferred(x) {
                // TODO: this clones - use mutability?
                Some(ty) => ty.clone().resolve(context)?,
                None => return Err(TypeInferenceError::ResolutionFailed(Type::Infer(x))),
            },
            x => x,
        })
    }
}
