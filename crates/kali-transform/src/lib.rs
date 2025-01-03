/// A trait implemented by AST transforms.
pub trait Transform {
    /// The transformation context.
    type Context;

    /// The error type.
    type Error;

    /// The input metadata type.
    type MetaInput;

    /// The output metadata type.
    type MetaOutput;

    /// Transform a statement.
    fn transform_stmt(
        &self,
        context: &mut Self::Context,
        stmt: kali_ast::Stmt<Self::MetaInput>,
    ) -> Result<kali_ast::Stmt<Self::MetaOutput>, Self::Error> {
        match stmt {
            kali_ast::Stmt::Import(import) => self
                .transform_import(context, import)
                .map(kali_ast::Stmt::Import),
            kali_ast::Stmt::Export(export) => self
                .transform_export(context, export)
                .map(kali_ast::Stmt::Export),
            kali_ast::Stmt::Const(name, value) => self
                .transform_const(context, name, value)
                .map(|(name, value)| kali_ast::Stmt::Const(name, value)),
            kali_ast::Stmt::Type(name, ty) => self
                .transform_type_decl(context, name, ty)
                .map(|(name, ty)| kali_ast::Stmt::Type(name, ty)),
            kali_ast::Stmt::Decl(decl) => {
                self.transform_decl(context, decl).map(kali_ast::Stmt::Decl)
            }
            kali_ast::Stmt::FuncDecl(func_decl) => self
                .transform_func_decl(context, func_decl)
                .map(kali_ast::Stmt::FuncDecl),
        }
    }

    /// Transform an import statement.
    fn transform_import(
        &self,
        context: &mut Self::Context,
        import: kali_ast::Import,
    ) -> Result<kali_ast::Import, Self::Error> {
        Ok(import)
    }

    /// Transform an export statement.
    fn transform_export(
        &self,
        context: &mut Self::Context,
        export: kali_ast::Export,
    ) -> Result<kali_ast::Export, Self::Error> {
        Ok(export)
    }

    /// Transform a constant declaration.
    fn transform_const(
        &self,
        context: &mut Self::Context,
        name: String,
        value: kali_ast::Literal<Self::MetaInput>,
    ) -> Result<(String, kali_ast::Literal<Self::MetaOutput>), Self::Error> {
        Ok((name, self.transform_literal(context, value)?))
    }

    /// Transform a type declaration.
    fn transform_type_decl(
        &self,
        context: &mut Self::Context,
        name: String,
        ty: kali_ast::TypeExpr,
    ) -> Result<(String, kali_ast::TypeExpr), Self::Error> {
        Ok((name, ty))
    }

    /// Transform a declaration.
    fn transform_decl(
        &self,
        context: &mut Self::Context,
        decl: kali_ast::Decl<Self::MetaInput>,
    ) -> Result<kali_ast::Decl<Self::MetaOutput>, Self::Error> {
        let value = self.transform_node_expr(context, decl.value)?;
        Ok(kali_ast::Decl {
            name: decl.name,
            value,
        })
    }

    /// Transform a function declaration.
    fn transform_func_decl(
        &self,
        context: &mut Self::Context,
        func_decl: kali_ast::FuncDecl<Self::MetaInput>,
    ) -> Result<kali_ast::FuncDecl<Self::MetaOutput>, Self::Error> {
        let params = func_decl
            .params
            .into_iter()
            .map(|param| self.transform_node_func_decl_param(context, param))
            .collect::<Result<_, _>>()?;
        let ret_ty = func_decl
            .ret_ty
            .map(|ty| self.transform_node_type_expr(context, ty))
            .transpose()?;
        let body = self.transform_node_expr(context, func_decl.body)?;
        Ok(kali_ast::FuncDecl {
            name: func_decl.name,
            params,
            ret_ty,
            body,
        })
    }

    fn transform_func_decl_param(
        &self,
        context: &mut Self::Context,
        param: kali_ast::FuncDeclParam,
    ) -> Result<kali_ast::FuncDeclParam, Self::Error> {
        let ty = param
            .ty
            .map(|ty| self.transform_type_expr(context, ty))
            .transpose()?;
        Ok(kali_ast::FuncDeclParam {
            name: param.name,
            ty,
        })
    }

    /// Transform a type expression.
    fn transform_type_expr(
        &self,
        context: &mut Self::Context,
        ty: kali_ast::TypeExpr,
    ) -> Result<kali_ast::TypeExpr, Self::Error> {
        Ok(ty)
    }

    /// Transform a node.
    fn transform_expr(
        &self,
        context: &mut Self::Context,
        expr: kali_ast::Expr<Self::MetaInput>,
    ) -> Result<kali_ast::Expr<Self::MetaOutput>, Self::Error> {
        match expr {
            kali_ast::Expr::Literal(literal) => self
                .transform_literal(context, literal)
                .map(kali_ast::Expr::Literal),
            kali_ast::Expr::Ident(ident) => self
                .transform_ident(context, ident)
                .map(kali_ast::Expr::Ident),
            kali_ast::Expr::BinaryExpr(binary_expr) => self
                .transform_binary_expr(context, binary_expr)
                .map(kali_ast::Expr::BinaryExpr),
            kali_ast::Expr::UnaryExpr(unary_expr) => self
                .transform_unary_expr(context, unary_expr)
                .map(kali_ast::Expr::UnaryExpr),
            kali_ast::Expr::Conditional(conditional) => self
                .transform_conditional(context, conditional)
                .map(kali_ast::Expr::Conditional),
            kali_ast::Expr::Lambda(lambda) => self
                .transform_lambda(context, lambda)
                .map(kali_ast::Expr::Lambda),
            kali_ast::Expr::Match(r#match) => self
                .transform_match(context, r#match)
                .map(kali_ast::Expr::Match),
            kali_ast::Expr::Call(call) => {
                self.transform_call(context, call).map(kali_ast::Expr::Call)
            }
        }
    }

    fn transform_literal(
        &self,
        context: &mut Self::Context,
        literal: kali_ast::Literal<Self::MetaInput>,
    ) -> Result<kali_ast::Literal<Self::MetaOutput>, Self::Error> {
        Ok(match literal {
            kali_ast::Literal::Integer(value) => kali_ast::Literal::Integer(value),
            kali_ast::Literal::Natural(value) => kali_ast::Literal::Natural(value),
            kali_ast::Literal::Float(value) => kali_ast::Literal::Float(value),
            kali_ast::Literal::Bool(value) => kali_ast::Literal::Bool(value),
            kali_ast::Literal::String(value) => kali_ast::Literal::String(value),
            kali_ast::Literal::Unit => kali_ast::Literal::Unit,
            kali_ast::Literal::Array(items) => {
                let items = items
                    .into_iter()
                    .map(|item| self.transform_node_expr(context, item))
                    .collect::<Result<_, _>>()?;
                kali_ast::Literal::Array(items)
            }
            kali_ast::Literal::Tuple(items) => {
                let items = items
                    .into_iter()
                    .map(|item| self.transform_node_expr(context, item))
                    .collect::<Result<_, _>>()?;
                kali_ast::Literal::Tuple(items)
            }
            kali_ast::Literal::Struct(fields) => {
                let fields = fields
                    .into_iter()
                    .map(|(key, value)| {
                        self.transform_node_expr(context, value)
                            .map(|value| (key, value))
                    })
                    .collect::<Result<_, _>>()?;
                kali_ast::Literal::Struct(fields)
            }
        })
    }

    fn transform_ident(
        &self,
        context: &mut Self::Context,
        ident: String,
    ) -> Result<String, Self::Error> {
        Ok(ident)
    }

    fn transform_binary_expr(
        &self,
        context: &mut Self::Context,
        binary_expr: kali_ast::BinaryExpr<Self::MetaInput>,
    ) -> Result<kali_ast::BinaryExpr<Self::MetaOutput>, Self::Error> {
        Ok(kali_ast::BinaryExpr {
            operator: binary_expr.operator,
            lhs: self.transform_node_expr(context, *binary_expr.lhs)?.boxed(),
            rhs: self.transform_node_expr(context, *binary_expr.rhs)?.boxed(),
        })
    }

    fn transform_unary_expr(
        &self,
        context: &mut Self::Context,
        unary_expr: kali_ast::UnaryExpr<Self::MetaInput>,
    ) -> Result<kali_ast::UnaryExpr<Self::MetaOutput>, Self::Error> {
        Ok(kali_ast::UnaryExpr {
            operator: unary_expr.operator,
            inner: self
                .transform_node_expr(context, *unary_expr.inner)?
                .boxed(),
        })
    }

    fn transform_conditional(
        &self,
        context: &mut Self::Context,
        conditional: kali_ast::Conditional<Self::MetaInput>,
    ) -> Result<kali_ast::Conditional<Self::MetaOutput>, Self::Error> {
        Ok(kali_ast::Conditional {
            condition: self
                .transform_node_expr(context, *conditional.condition)?
                .boxed(),
            body: self
                .transform_node_expr(context, *conditional.body)?
                .boxed(),
            otherwise: self
                .transform_node_expr(context, *conditional.otherwise)?
                .boxed(),
        })
    }

    fn transform_lambda(
        &self,
        context: &mut Self::Context,
        lambda: kali_ast::Lambda<Self::MetaInput>,
    ) -> Result<kali_ast::Lambda<Self::MetaOutput>, Self::Error> {
        todo!()
    }

    fn transform_match(
        &self,
        context: &mut Self::Context,
        r#match: kali_ast::Match<Self::MetaInput>,
    ) -> Result<kali_ast::Match<Self::MetaOutput>, Self::Error> {
        Ok(kali_ast::Match {
            expr: self
                .transform_node_expr(context, *r#match.expr)
                .map(|subject| subject.boxed())?,
            branches: r#match
                .branches
                .into_iter()
                .map(|(pattern, expr)| Ok((pattern, self.transform_node_expr(context, expr)?)))
                .collect::<Result<_, _>>()?,
        })
    }

    fn transform_call(
        &self,
        context: &mut Self::Context,
        call: kali_ast::Call<Self::MetaInput>,
    ) -> Result<kali_ast::Call<Self::MetaOutput>, Self::Error> {
        Ok(kali_ast::Call {
            fun: self.transform_node_expr(context, *call.fun)?.boxed(),
            args: call
                .args
                .into_iter()
                .map(|arg| self.transform_node_expr(context, arg))
                .collect::<Result<_, _>>()?,
        })
    }

    fn transform_node_expr(
        &self,
        context: &mut Self::Context,
        node: kali_ast::Node<kali_ast::Expr<Self::MetaInput>, Self::MetaInput>,
    ) -> Result<kali_ast::Node<kali_ast::Expr<Self::MetaOutput>, Self::MetaOutput>, Self::Error>;

    fn transform_node_func_decl_param(
        &self,
        context: &mut Self::Context,
        node: kali_ast::Node<kali_ast::FuncDeclParam, Self::MetaInput>,
    ) -> Result<kali_ast::Node<kali_ast::FuncDeclParam, Self::MetaOutput>, Self::Error>;

    fn transform_node_type_expr(
        &self,
        context: &mut Self::Context,
        node: kali_ast::Node<kali_ast::TypeExpr, Self::MetaInput>,
    ) -> Result<kali_ast::Node<kali_ast::TypeExpr, Self::MetaOutput>, Self::Error>;
}

struct TypeCacheTransform;

impl Transform for TypeCacheTransform {
    type Context;

    type Error;

    type MetaInput;

    type MetaOutput;

    fn transform_node_expr(
        &self,
        context: &mut Self::Context,
        node: kali_ast::Node<kali_ast::Expr<Self::MetaInput>, Self::MetaInput>,
    ) -> Result<kali_ast::Node<kali_ast::Expr<Self::MetaOutput>, Self::MetaOutput>, Self::Error>
    {
        todo!()
    }

    fn transform_node_func_decl_param(
        &self,
        context: &mut Self::Context,
        node: kali_ast::Node<kali_ast::FuncDeclParam, Self::MetaInput>,
    ) -> Result<kali_ast::Node<kali_ast::FuncDeclParam, Self::MetaOutput>, Self::Error> {
        todo!()
    }

    fn transform_node_type_expr(
        &self,
        context: &mut Self::Context,
        node: kali_ast::Node<kali_ast::TypeExpr, Self::MetaInput>,
    ) -> Result<kali_ast::Node<kali_ast::TypeExpr, Self::MetaOutput>, Self::Error> {
        todo!()
    }
}
