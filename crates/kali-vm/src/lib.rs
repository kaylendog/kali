//! Implements a (mediocre) stack-based virtual machine for executing Kali code.

use std::collections::BTreeMap;

use kali_ast::{BinaryOp, Literal, UnaryOp};
use kali_ir::{Module, Operator};

/// A runtime for executing Kali code.
pub struct Runtime {
    /// The stack of literals.
    stack: Vec<Literal>,
    /// A map of variable names to their addresses on the heap.
    variables: BTreeMap<String, usize>,
    /// Program counter.
    pc: usize,
    /// The module to execute.
    module: Module,
}

impl Runtime {
    /// Create a new runtime.
    pub fn new(module: Module) -> Self {
        Self {
            stack: Vec::new(),
            variables: BTreeMap::new(),
            pc: 0,
            module,
        }
    }

    /// Run the module.
    pub fn run(&mut self) {
        while self.pc < self.module.len() {
            self.step();
        }
    }

    /// Run the program in debug mode.
    pub fn run_debug(&mut self) {
        while self.pc < self.module.len() {
            print!("{:04} {:?}", self.pc, self.stack);
            self.step();
            println!(" -> {:?}", self.stack);
        }
    }

    /// Execute the next instruction.
    pub fn step(&mut self) {
        let operator = { &self.module[self.pc].clone() };
        operator.execute(self);
        self.pc += 1;
    }

    /// Get a reference to the stack.
    pub fn stack(&self) -> &Vec<Literal> {
        &self.stack
    }
}

/// A trait for types that can be executed.
trait Execute {
    /// Execute the operator.
    fn execute(&self, runtime: &mut Runtime);
}

impl Execute for Operator {
    fn execute(&self, runtime: &mut Runtime) {
        match self {
            Operator::PushLiteral(literal) => runtime.stack.push(literal.clone()),
            Operator::PushVariable(var) => {
                let address = runtime.variables.get(var).copied().unwrap();
                let value = runtime.stack[address].clone();
                runtime.stack.push(value);
            }
            Operator::Pop => {
                runtime.stack.pop().unwrap();
            }
            Operator::BinaryOp(op) => match op {
                BinaryOp::Add => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a + b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Float(a + b),
                        _ => panic!("invalid types for addition"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Subtract => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a - b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Float(a - b),
                        _ => panic!("invalid types for subtraction"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Multiply => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a * b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Float(a * b),
                        _ => panic!("invalid types for multiplication"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Divide => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a / b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Float(a / b),
                        _ => panic!("invalid types for division"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Exponentiate => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a.pow(b as u32)),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Float(a.powf(b)),
                        _ => panic!("invalid types for exponentiation"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Modulo => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a % b),
                        _ => panic!("invalid types for modulo"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Equal => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Bool(a == b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Bool(a == b),
                        (Literal::Bool(a), Literal::Bool(b)) => Literal::Bool(a == b),
                        (Literal::String(a), Literal::String(b)) => Literal::Bool(a == b),
                        (Literal::Unit, Literal::Unit) => Literal::Bool(true),
                        _ => panic!("invalid types for equality"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::NotEqual => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Bool(a != b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Bool(a != b),
                        (Literal::Bool(a), Literal::Bool(b)) => Literal::Bool(a != b),
                        (Literal::String(a), Literal::String(b)) => Literal::Bool(a != b),
                        (Literal::Unit, Literal::Unit) => Literal::Bool(false),
                        _ => panic!("invalid types for inequality"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::LessThan => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Bool(a < b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Bool(a < b),
                        _ => panic!("invalid types for less than"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::LessThanOrEqual => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Bool(a <= b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Bool(a <= b),
                        _ => panic!("invalid types for less than or equal"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::GreaterThan => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Bool(a > b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Bool(a > b),
                        _ => panic!("invalid types for greater than"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::GreaterThanOrEqual => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Bool(a >= b),
                        (Literal::Float(a), Literal::Float(b)) => Literal::Bool(a >= b),
                        _ => panic!("invalid types for greater than or equal"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::LogicalAnd => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Bool(a), Literal::Bool(b)) => Literal::Bool(a && b),
                        _ => panic!("invalid types for logical and"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::LogicalOr => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Bool(a), Literal::Bool(b)) => Literal::Bool(a || b),
                        _ => panic!("invalid types for logical or"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::BitwiseAnd => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a & b),
                        _ => panic!("invalid types for bitwise and"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::BitwiseOr => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a | b),
                        _ => panic!("invalid types for bitwise or"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::BitwiseXor => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a ^ b),
                        _ => panic!("invalid types for bitwise xor"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::BitwiseShiftLeft => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a << b),
                        _ => panic!("invalid types for bitwise shift left"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::BitwiseShiftRight => {
                    let b = runtime.stack.pop().unwrap();
                    let a = runtime.stack.pop().unwrap();
                    let result = match (a, b) {
                        (Literal::Int(a), Literal::Int(b)) => Literal::Int(a >> b),
                        _ => panic!("invalid types for bitwise shift right"),
                    };
                    runtime.stack.push(result);
                }
                BinaryOp::Concatenate => todo!("concatenate operator"),
            },
            Operator::UnaryOp(op) => match op {
                UnaryOp::Negate => {
                    let a = runtime.stack.pop().unwrap();
                    let result = match a {
                        Literal::Int(a) => Literal::Int(-a),
                        Literal::Float(a) => Literal::Float(-a),
                        _ => panic!("invalid type for negation"),
                    };
                    runtime.stack.push(result);
                }
                UnaryOp::LogicalNot => {
                    let a = runtime.stack.pop().unwrap();
                    let result = match a {
                        Literal::Bool(a) => Literal::Bool(!a),
                        _ => panic!("invalid type for logical not"),
                    };
                    runtime.stack.push(result);
                }
                UnaryOp::BitwiseNot => {
                    let a = runtime.stack.pop().unwrap();
                    let result = match a {
                        Literal::Int(a) => Literal::Int(!a),
                        _ => panic!("invalid type for bitwise not"),
                    };
                    runtime.stack.push(result);
                }
            },
            Operator::Jump(target) => {
                // need -1 because the pc will be incremented after this
                runtime.pc = *target - 1;
            }
            Operator::ConditionalJump(target) => {
                let condition = runtime.stack.pop().expect("missing condition");
                if let Literal::Bool(true) = condition {
                    // need -1 because the pc will be incremented after this
                    runtime.pc = *target - 1;
                }
            }
        }
    }
}
