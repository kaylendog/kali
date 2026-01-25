use crate::{
    Definition, Destructor, Expr, ExprKind, Ident, ImportTree, ImportTreeKind, Item, ItemKind,
    LambdaParam, MatchArm, Module, Pattern, Type, TypeAlias, TypeKind,
};

/// A trait for visiting elements of the Kali Abstract Syntax Tree (AST).
///
/// This trait provides a mechanism for performing operations on individual elements
/// of the Kali AST. It is designed to be used in scenarios where you need to
/// process or analyse specific parts of the hierarchical or structured representation
/// of the AST.
///
/// # Associated Types
///
/// * `Error` - The type of error that may be returned during the visit operation.
///
/// # Required Methods
///
/// * `visit` - The method responsible for visiting a specific target of type `T`
///   within the Kali AST. It takes a reference to the target and returns a `Result`
///   indicating success or failure.
pub trait Visitor {
    /// The associated error type that may be returned during the visit operation.
    type Error;

    /// Visits a module within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `module` - A reference to the `Module` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_module(&mut self, module: &Module) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits an item within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `item` - A reference to the `Item` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_item(&mut self, item: &Item) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits an import tree within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `import_tree` - A reference to the `ImportTree` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_import_tree(&mut self, import_tree: &ImportTree) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a type alias within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `type_alias` - A reference to the `TypeAlias` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_type_alias(&mut self, type_alias: &TypeAlias) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a type within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `ty` - A reference to the `Type` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_type(&mut self, ty: &Type) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a definition within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `definition` - A reference to the `Definition` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_definition(&mut self, definition: &Definition) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits an identifier within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `ident` - A reference to the `Ident` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_ident(&mut self, ident: &Ident) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits an expression within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `expr` - A reference to the `Expr` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_expr(&mut self, expr: &Expr) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a match arm within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `match_arm` - A reference to the `MatchArm` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_match_arm(&mut self, match_arm: &MatchArm) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a pattern within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A reference to the `Pattern` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_pattern(&mut self, pattern: &Pattern) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a destructor within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `destructor` - A reference to the `Destructor` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_destructor(&mut self, destructor: &Destructor) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visits a lambda parameter within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `lambda_param` - A reference to the `LambdaParam` to be visited.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn visit_lambda_param(&mut self, lambda_param: &LambdaParam) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A tree walker for the Kali AST.
pub struct Walker {
    /// Specifies the order in which the AST should be walked.
    order: WalkOrder,
}

/// Represents the order in which the AST should be walked.
#[derive(Clone, Copy)]
pub enum WalkOrder {
    /// Visit parent nodes before their children.
    PreOrder,
    /// Visit children nodes before their parent.
    PostOrder,
}

impl Walker {
    /// Walks a module within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `module` - A reference to the `Module` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_module<V: Visitor>(
        &self,
        visitor: &mut V,
        module: &Module,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_module(module)?;
        }
        for item in &module.items {
            self.walk_item(visitor, item)?;
        }
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_module(module)?;
        }
        Ok(())
    }

    /// Walks an item within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `item` - A reference to the `Item` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_item<V: Visitor>(&self, visitor: &mut V, item: &Item) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_item(item)?;
        }
        match &item.kind {
            ItemKind::Import(import_tree) => self.walk_import_tree(visitor, import_tree)?,
            ItemKind::TypeAlias(type_alias) => self.walk_type_alias(visitor, type_alias)?,
            ItemKind::Definition(definition) => self.walk_definition(visitor, definition)?,
        }
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_item(item)?;
        }
        Ok(())
    }

    /// Walks an import tree within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `import_tree` - A reference to the `ImportTree` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_import_tree<V: Visitor>(
        &self,
        visitor: &mut V,
        import_tree: &ImportTree,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_import_tree(import_tree)?;
        }

        match &import_tree.kind {
            ImportTreeKind::Item { name, alias } => {
                visitor.visit_ident(name)?;
                if let Some(alias) = alias {
                    visitor.visit_ident(alias)?;
                }
            }
            ImportTreeKind::Segment { name, child } => {
                visitor.visit_ident(name)?;
                self.walk_import_tree(visitor, child)?;
            }
            ImportTreeKind::Glob => {
                visitor.visit_import_tree(import_tree)?;
            }
            ImportTreeKind::List(import_trees) => {
                for tree in import_trees {
                    self.walk_import_tree(visitor, tree)?;
                }
            }
        }

        if let WalkOrder::PostOrder = self.order {
            visitor.visit_import_tree(import_tree)?;
        }
        Ok(())
    }

    /// Walks a type alias within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `type_alias` - A reference to the `TypeAlias` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_type_alias<V: Visitor>(
        &self,
        visitor: &mut V,
        type_alias: &TypeAlias,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_type_alias(type_alias)?;
        }
        self.walk_type(visitor, &type_alias.ty)?;
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_type_alias(type_alias)?;
        }
        Ok(())
    }

    /// Walks a type within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `ty` - A reference to the `Type` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_type<V: Visitor>(&self, visitor: &mut V, ty: &Type) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_type(ty)?;
        }

        match &ty.kind {
            TypeKind::Primitive(_) => {}
            TypeKind::Named(ident) => {
                self.walk_ident(visitor, ident)?;
            }
            TypeKind::Tuple(items) => {
                for item in items {
                    self.walk_type(visitor, item)?;
                }
            }
            TypeKind::List(inner_type) => {
                self.walk_type(visitor, inner_type)?;
            }
            TypeKind::Record(index_map) => {
                for (key, value) in index_map {
                    self.walk_ident(visitor, key)?;
                    self.walk_type(visitor, value)?;
                }
            }
            TypeKind::Fn(params, return_type) => {
                for param in params {
                    self.walk_type(visitor, param)?;
                }
                self.walk_type(visitor, return_type)?;
            }
            crate::TypeKind::Intersection { lhs, rhs } => {
                self.walk_type(visitor, lhs)?;
                self.walk_type(visitor, rhs)?;
            }
            crate::TypeKind::Union { lhs, rhs } => {
                self.walk_type(visitor, lhs)?;
                self.walk_type(visitor, rhs)?;
            }
        }

        if let WalkOrder::PostOrder = self.order {
            visitor.visit_type(ty)?;
        }
        Ok(())
    }

    /// Walks a definition within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `definition` - A reference to the `Definition` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_definition<V: Visitor>(
        &self,
        visitor: &mut V,
        definition: &Definition,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_definition(definition)?;
        }
        self.walk_destructor(visitor, &definition.name)?;
        self.walk_expr(visitor, &definition.expr)?;
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_definition(definition)?;
        }
        Ok(())
    }

    /// Walks an identifier within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `ident` - A reference to the `Ident` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_ident<V: Visitor>(&self, visitor: &mut V, ident: &Ident) -> Result<(), V::Error> {
        visitor.visit_ident(ident)?;
        Ok(())
    }

    /// Walks an expression within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `expr` - A reference to the `Expr` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_expr<V: Visitor>(&self, visitor: &mut V, expr: &Expr) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_expr(expr)?;
        }
        match &expr.kind {
            ExprKind::Var(ident) => {
                self.walk_ident(visitor, ident)?;
            }
            ExprKind::Literal(_) => {}
            ExprKind::BinaryExpr { op: _, lhs, rhs } => {
                self.walk_expr(visitor, lhs)?;
                self.walk_expr(visitor, rhs)?;
            }
            ExprKind::UnaryExpr { op: _, expr } => {
                self.walk_expr(visitor, expr)?;
            }
            ExprKind::Tuple(exprs) => {
                for e in exprs {
                    self.walk_expr(visitor, e)?;
                }
            }
            ExprKind::List(exprs) => {
                for e in exprs {
                    self.walk_expr(visitor, e)?;
                }
            }
            ExprKind::Record { fields } => {
                for (_, value) in fields {
                    self.walk_expr(visitor, value)?;
                }
            }
            ExprKind::Conditional {
                condition,
                body,
                otherwise,
            } => {
                self.walk_expr(visitor, condition)?;
                self.walk_expr(visitor, body)?;
                if let Some(otherwise_expr) = otherwise {
                    self.walk_expr(visitor, otherwise_expr)?;
                }
            }
            ExprKind::Match { value, arms } => {
                self.walk_expr(visitor, value)?;
                for arm in arms {
                    self.walk_match_arm(visitor, arm)?;
                }
            }
            ExprKind::Lambda {
                params,
                ret_ty,
                body,
            } => {
                for param in params {
                    self.walk_lambda_param(visitor, param)?;
                }
                if let Some(return_type) = ret_ty {
                    self.walk_type(visitor, return_type)?;
                }
                self.walk_expr(visitor, body)?;
            }
            ExprKind::Call {
                function,
                arguments,
            } => {
                self.walk_expr(visitor, function)?;
                for arg in arguments {
                    self.walk_expr(visitor, arg)?;
                }
            }
        }

        if let WalkOrder::PostOrder = self.order {
            visitor.visit_expr(expr)?;
        }
        Ok(())
    }

    /// Walks a match arm within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `match_arm` - A reference to the `MatchArm` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_match_arm<V: Visitor>(
        &self,
        visitor: &mut V,
        match_arm: &MatchArm,
    ) -> Result<(), V::Error> {
        self.walk_pattern(visitor, &match_arm.pattern)?;
        self.walk_expr(visitor, &match_arm.expr)?;
        Ok(())
    }

    /// Walks a pattern within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `pattern` - A reference to the `Pattern` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_pattern<V: Visitor>(
        &self,
        visitor: &mut V,
        pattern: &Pattern,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_pattern(pattern)?;
        }
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_pattern(pattern)?;
        }
        Ok(())
    }

    /// Walks a destructor within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `destructor` - A reference to the `Destructor` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_destructor<V: Visitor>(
        &self,
        visitor: &mut V,
        destructor: &Destructor,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_destructor(destructor)?;
        }
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_destructor(destructor)?;
        }
        Ok(())
    }

    /// Walks a lambda parameter within the Kali AST.
    ///
    /// # Arguments
    ///
    /// * `visitor` - A mutable reference to the `Visitor` implementation.
    /// * `lambda_param` - A reference to the `LambdaParam` to be walked.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn walk_lambda_param<V: Visitor>(
        &self,
        visitor: &mut V,
        lambda_param: &LambdaParam,
    ) -> Result<(), V::Error> {
        if let WalkOrder::PreOrder = self.order {
            visitor.visit_lambda_param(lambda_param)?;
        }
        self.walk_destructor(visitor, &lambda_param.parameter)?;
        if let Some(param_type) = &lambda_param.ty {
            self.walk_type(visitor, param_type)?;
        }
        if let WalkOrder::PostOrder = self.order {
            visitor.visit_lambda_param(lambda_param)?;
        }
        Ok(())
    }
}
