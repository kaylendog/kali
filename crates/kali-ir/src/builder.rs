//! A stack-based intermediate representation for the Kali language.

use std::collections::HashMap;

use kali_ast::{BinaryExpr, BinaryOp, Conditional, Expr, Literal, Node, Stmt, UnaryExpr, UnaryOp};

use crate::Operator;

/// A translation unit in the stack-based intermediate representation.
pub struct ModuleBuilder {
    pub functions: HashMap<String, Vec<Operator>>,
}

impl ModuleBuilder {
    /// Create a new translation unit.
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Create a new function in the translation unit.
    pub fn function(&mut self, name: &str) -> FunctionBuilder {
        FunctionBuilder {
            module: self,
            instructions: Vec::new(),
            name: name.to_string(),
        }
    }
}

/// A builder for constructing functions in the stack-based intermediate representation.
pub struct FunctionBuilder<'a> {
    /// The module to which the function belongs.
    pub module: &'a mut ModuleBuilder,
    /// The instructions for the function.
    pub instructions: Vec<Operator>,
    /// The name of the function.
    pub name: String,
}

impl FunctionBuilder<'_> {
    /// Push a literal onto the stack.
    pub fn push_literal(&mut self, literal: Literal) {
        self.instructions.push(Operator::PushLiteral(literal));
    }

    /// Push a variable onto the stack.
    pub fn push_variable(&mut self, name: &str) {
        self.instructions
            .push(Operator::PushVariable(name.to_string()));
    }

    /// Perform a binary operation.
    pub fn binary_op(&mut self, operator: BinaryOp) {
        self.instructions.push(Operator::BinaryOp(operator));
    }

    /// Perform a unary operation.
    pub fn unary_op(&mut self, operator: UnaryOp) {
        self.instructions.push(Operator::UnaryOp(operator));
    }

    /// Insert a jump instruction.
    pub fn jump(&mut self, target: usize) {
        self.instructions.push(Operator::Jump(target));
    }

    /// Insert a conditional jump instruction.
    pub fn conditional_jump(&mut self, target: usize) {
        self.instructions.push(Operator::ConditionalJump(target));
    }

    /// Finish building the function and add the instructions to the module.
    pub fn finish(self) {
        self.module.functions.insert(self.name, self.instructions);
    }
}

/// A trait for types that can be compiled to a stack-based intermediate representation.
pub trait Compile {
    fn compile(&self, unit: &mut FunctionBuilder);
}

impl<T> Compile for Node<T>
where
    T: Compile,
{
    fn compile(&self, unit: &mut FunctionBuilder) {
        self.inner.compile(unit);
    }
}

impl Compile for Expr {
    fn compile(&self, unit: &mut FunctionBuilder) {
        match self {
            Expr::Literal(literal) => unit.push_literal(literal.clone()),
            Expr::Identifier(name) => unit.push_variable(name),
            Expr::BinaryExpr(binary) => binary.compile(unit),
            Expr::UnaryExpr(unary) => unary.compile(unit),
            Expr::Conditional(conditional) => conditional.compile(unit),
            Expr::Lambda(_) => todo!("lambda compilation"),
        }
    }
}

impl Compile for BinaryExpr {
    fn compile(&self, unit: &mut FunctionBuilder) {
        self.lhs.compile(unit);
        self.rhs.compile(unit);
        unit.binary_op(self.operator);
    }
}

impl Compile for UnaryExpr {
    fn compile(&self, unit: &mut FunctionBuilder) {
        self.inner.compile(unit);
        unit.unary_op(self.operator);
    }
}

impl Compile for Conditional {
    fn compile(&self, unit: &mut FunctionBuilder) {
        // compile the condition,
        self.condition.compile(unit);

        // add jumps - we will fill in the targets later
        let jump_idx = unit.instructions.len();
        unit.conditional_jump(0);
        unit.jump(0);

        // compile the body, keeping track of where it begins
        let body_idx = unit.instructions.len();
        self.body.compile(unit);
        let body_end_idx = unit.instructions.len();
        unit.jump(0);

        // compile the otherwise branch, keeping track of where it begins
        let otherwise_idx = unit.instructions.len();
        self.otherwise.compile(unit);
        let otherwise_end_idx = unit.instructions.len();

        // fill in the jump targets
        unit.instructions[jump_idx] = Operator::ConditionalJump(body_idx);
        unit.instructions[jump_idx + 1] = Operator::Jump(otherwise_idx);
        unit.instructions[body_end_idx] = Operator::Jump(otherwise_end_idx);
    }
}

impl Compile for Stmt {
    fn compile(&self, unit: &mut FunctionBuilder) {
        match self {
            Stmt::Import(_) => todo!(),
            Stmt::Export(_) => todo!(),
            Stmt::Const(_, _) => todo!(),
            Stmt::Type(_, _) => todo!(),
            Stmt::Decl(_) => todo!(),
        }
    }
}
