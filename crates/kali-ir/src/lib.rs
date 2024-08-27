use std::collections::BTreeMap;

use kali_ast::{BinaryOp, Literal, UnaryOp};

mod builder;

pub use builder::*;
use strum::Display;

/// A module in the stack-based intermediate representation.
pub struct Module {
    /// Constants in the module.
    pub constants: BTreeMap<String, Literal>,
    /// Functions in the module.
    pub functions: BTreeMap<String, Function>,
}

/// A built function in the stack-based intermediate representation.
pub struct Function {
    /// The name of the function.
    pub name: String,
    /// The instructions for the function.
    pub instructions: Vec<Operator>,
}

/// An instruction operating on the stack.
#[derive(Debug, Clone, Display)]
pub enum Operator {
    /// Push a literal onto the stack.
    #[strum(serialize = "push")]
    PushLiteral(Literal),
    /// Push a variable onto the stack.
    #[strum(serialize = "load {0}")]
    PushVariable(String),
    /// Perform a binary operation.
    #[strum(serialize = "binary")]
    BinaryOp(BinaryOp),
    /// Perform a unary operation.
    #[strum(serialize = "unary")]
    UnaryOp(UnaryOp),
    #[strum(serialize = "jump {0}")]
    /// Jump to another instruction.
    Jump(usize),
    /// Jump conditionally to another instruction.
    ConditionalJump(usize),
    /// Pop a value from the stack, discarding it.
    Pop,
}
