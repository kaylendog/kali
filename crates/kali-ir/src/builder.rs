//! A stack-based intermediate representation for the Kali language.

use std::collections::HashMap;

use kali_ast::{BinaryExpr, BinaryOp, Conditional, Expr, Literal, Node, Stmt, UnaryExpr, UnaryOp};
use kali_type::{Constant, Type};

use crate::Operator;

/// A translation unit in the stack-based intermediate representation.
pub struct ModuleBuilder {
    /// The functions in the translation unit.
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
    /// Push an integer literal onto the stack.
    pub fn pushi(&mut self, value: i64) {
        self.instructions.push(Operator::PushInt(value));
    }

    /// Push a floating-point literal onto the stack.
    pub fn pushf(&mut self, value: f64) {
        self.instructions.push(Operator::PushFloat(value));
    }

    /// Push a boolean literal onto the stack.
    pub fn pushb(&mut self, value: bool) {
        self.instructions.push(Operator::PushBool(value));
    }

    /// Load an integer from a variable onto the stack.
    pub fn loadi(&mut self, name: &str) {
        self.instructions
            .push(Operator::LoadInteger(name.to_string()));
    }

    /// Load a floating-point number from a variable onto the stack.
    pub fn loadf(&mut self, name: &str) {
        self.instructions
            .push(Operator::LoadFloat(name.to_string()));
    }

    /// Load a boolean from a variable onto the stack.
    pub fn loadb(&mut self, name: &str) {
        self.instructions.push(Operator::LoadBool(name.to_string()));
    }
    /// Add two integers.
    pub fn addi(&mut self) {
        self.instructions.push(Operator::AddInt);
    }

    /// Subtract two integers.
    pub fn subi(&mut self) {
        self.instructions.push(Operator::SubtractInt);
    }

    /// Multiply two integers.
    pub fn muli(&mut self) {
        self.instructions.push(Operator::MultiplyInt);
    }

    /// Divide two integers.
    pub fn divi(&mut self) {
        self.instructions.push(Operator::DivideInt);
    }

    /// Left-shift an integer.
    pub fn lshift(&mut self) {
        self.instructions.push(Operator::LeftShift);
    }

    /// Right-shift an integer.
    pub fn rshift(&mut self) {
        self.instructions.push(Operator::RightShift);
    }

    /// Perform a bitwise and operation on two integers.
    pub fn andi(&mut self) {
        self.instructions.push(Operator::AndInt);
    }

    /// Perform a bitwise or operation on two integers.
    pub fn ori(&mut self) {
        self.instructions.push(Operator::OrInt);
    }

    /// Perform a bitwise xor operation on two integers.
    pub fn xori(&mut self) {
        self.instructions.push(Operator::XorInt);
    }

    /// Add two floating-point numbers.
    pub fn addf(&mut self) {
        self.instructions.push(Operator::AddFloat);
    }

    /// Subtract two floating-point numbers.
    pub fn subf(&mut self) {
        self.instructions.push(Operator::SubtractFloat);
    }

    /// Multiply two floating-point numbers.
    pub fn mulf(&mut self) {
        self.instructions.push(Operator::MultiplyFloat);
    }

    /// Divide two floating-point numbers.
    pub fn divf(&mut self) {
        self.instructions.push(Operator::DivideFloat);
    }

    /// Negate an integer.
    pub fn negi(&mut self) {
        self.instructions.push(Operator::NegateInt);
    }

    /// Bitwise not an integer.
    pub fn noti(&mut self) {
        self.instructions.push(Operator::NotInt);
    }

    /// Negate a floating-point number.
    pub fn negf(&mut self) {
        self.instructions.push(Operator::NegateFloat);
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
            Expr::Literal(literal) => match literal {
                Literal::Int(value) => unit.pushi(*value),
                Literal::Float(value) => unit.pushf(*value),
                Literal::Bool(value) => unit.pushb(*value),
                _ => todo!("literal compilation"),
            },
            Expr::Identifier(_) => todo!("identifier compilation"),
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
        // get type of lhs - should be the same as rhs under the type checker
        let ty = self.lhs.meta.ty.get().unwrap();
        match ty {
            Type::Constant(c) => match c {
                Constant::Int => match self.operator {
                    BinaryOp::Add => unit.addi(),
                    BinaryOp::Subtract => unit.subi(),
                    BinaryOp::Multiply => unit.muli(),
                    BinaryOp::Divide => unit.divi(),
                    BinaryOp::BitwiseShiftLeft => unit.lshift(),
                    BinaryOp::BitwiseShiftRight => unit.rshift(),
                    BinaryOp::BitwiseAnd => unit.andi(),
                    BinaryOp::BitwiseOr => unit.ori(),
                    BinaryOp::BitwiseXor => unit.xori(),
                    x => panic!("unsupported operator {x:?}"),
                },
                Constant::Float => match self.operator {
                    BinaryOp::Add => unit.addf(),
                    BinaryOp::Subtract => unit.subf(),
                    BinaryOp::Multiply => unit.mulf(),
                    BinaryOp::Divide => unit.divf(),
                    x => panic!("unsupported operator {x:?}"),
                },
                x => panic!("unsupported type {x}"),
            },
            x => panic!("unsupported type {x}"),
        }
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
            Stmt::Import(_) => todo!("import compilation"),
            Stmt::Export(_) => todo!("export compilation"),
            Stmt::Const(_, _) => todo!(),
            Stmt::Type(_, _) => {}
            Stmt::Decl(_) => todo!(),
        }
    }
}
