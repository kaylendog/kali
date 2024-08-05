use kali_type::{Constant, Type};

/// A type expression in the Kali language.
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Constant(ConstantType),
    /// A type variable.
    Variable(String),
    /// A function type.
    Function(Vec<TypeExpr>, Box<TypeExpr>),
    /// A tuple type.
    Tuple(Vec<TypeExpr>),
    /// An array type.
    Array(Box<TypeExpr>),
    /// A record type.
    Record(Vec<(String, TypeExpr)>),
}

/// An enumeration of literal constant types.
#[derive(Debug, Clone)]
pub enum ConstantType {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Never,
}

impl TypeExpr {
    pub fn as_ty(&self) -> Type {
        match self {
            TypeExpr::Constant(primitive) => match primitive {
                ConstantType::Int => Type::Constant(Constant::Int),
                ConstantType::Float => Type::Constant(Constant::Float),
                ConstantType::Bool => Type::Constant(Constant::Bool),
                ConstantType::String => Type::Constant(Constant::String),
                ConstantType::Unit => Type::Constant(Constant::Unit),
                ConstantType::Never => Type::Constant(Constant::Never),
            },
            TypeExpr::Variable(_name) => todo!("TypeExpr::Variable"),
            TypeExpr::Function(params, ret) => {
                let params = params.iter().map(|param| param.as_ty()).collect();
                let ret = ret.as_ty();
                Type::Lambda(params, Box::new(ret))
            }
            TypeExpr::Tuple(types) => {
                let types = types.iter().map(|ty| ty.as_ty()).collect();
                Type::Tuple(types)
            }
            TypeExpr::Array(ty) => Type::Array(Box::new(ty.as_ty())),
            TypeExpr::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.as_ty()))
                    .collect();
                Type::Record(fields)
            }
        }
    }
}
