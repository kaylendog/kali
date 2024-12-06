use std::collections::{BTreeMap, HashMap};

use bitcode::{Decode, Encode};
use kali_ast::UnaryOp;
use strum::Display;

mod builder;

pub use builder::*;

pub struct ImportPath {}

pub enum Export {
    Function(String),
    Constant(),
}

/// A module in the stack-based intermediate representation.
#[derive(Debug, Default, Encode, Decode)]
pub struct Module {
    /// The module bytecode version.
    pub version: u16,
    /// A map of exports.
    pub imports: HashMap<String, String>,
    /// Functions in the module.
    pub functions: BTreeMap<String, Function>,
}

/// A built function in the stack-based intermediate representation.
#[derive(Debug, Encode, Decode)]
pub struct Function {
    /// The name of the function.
    pub name: String,
    /// The instructions for the function.
    pub instructions: Vec<Operator>,
}

/// An instruction operating on the stack.
#[derive(Debug, Clone, Display, Encode, Decode)]
pub enum Operator {
    /// No operation.
    Noop,
    /// Push an integer literal onto the stack.
    #[strum(serialize = "pushi {0}")]
    PushInt(i64),
    /// Push a floating-point literal onto the stack.
    #[strum(serialize = "pushf {0}")]
    PushFloat(f64),
    /// Push a boolean literal onto the stack.
    #[strum(serialize = "pushb {0}")]
    PushBool(bool),
    /// Load an integer from a variable onto the stack.
    #[strum(serialize = "loadi {0}")]
    LoadInteger(String),
    /// Load a floating-point number from a variable onto the stack.
    #[strum(serialize = "loadf {0}")]
    LoadFloat(String),
    /// Load a boolean from a variable onto the stack.
    #[strum(serialize = "loadb {0}")]
    LoadBool(String),
    /// Add two integers.
    #[strum(serialize = "addi")]
    AddInt,
    /// Subtract two integers.
    #[strum(serialize = "subi")]
    SubtractInt,
    /// Multiply two integers.
    #[strum(serialize = "muli")]
    MultiplyInt,
    /// Divide two integers.
    #[strum(serialize = "divi")]
    DivideInt,
    /// Left-shift an integer.
    #[strum(serialize = "lshift")]
    LeftShift,
    /// Right-shift an integer.
    #[strum(serialize = "rshift")]
    RightShift,
    /// Perform a bitwise and operation on two integers.
    #[strum(serialize = "andi")]
    AndInt,
    /// Perform a bitwise or operation on two integers.
    #[strum(serialize = "ori")]
    OrInt,
    /// Perform a bitwise xor operation on two integers.
    #[strum(serialize = "xori")]
    XorInt,
    /// Add two floating-point numbers.
    #[strum(serialize = "addf")]
    AddFloat,
    /// Subtract two floating-point numbers.
    #[strum(serialize = "subf")]
    SubtractFloat,
    /// Multiply two floating-point numbers.
    #[strum(serialize = "mulf")]
    MultiplyFloat,
    /// Divide two floating-point numbers.
    #[strum(serialize = "divf")]
    DivideFloat,
    /// Negate an integer.
    #[strum(serialize = "negi")]
    NegateInt,
    /// Bitwise not an integer.
    #[strum(serialize = "noti")]
    NotInt,
    /// Negate a floating-point number.
    #[strum(serialize = "negf")]
    NegateFloat,
    /// Perform a unary operation.
    #[strum(serialize = "unary")]
    UnaryOp(UnaryOp),
    /// Jump to another instruction.
    #[strum(serialize = "jump {0}")]
    Jump(usize),
    /// Jump conditionally to another instruction.
    #[strum(serialize = "cjump {0}")]
    ConditionalJump(usize),
    /// Pop an integer from the stack, discarding it.
    #[strum(serialize = "popi")]
    PopInt,
    /// Pop a float from the stack, discarding it.
    #[strum(serialize = "popf")]
    PopFloat,
    /// Pop a boolean from the stack, discarding it.
    #[strum(serialize = "popb")]
    PopBool,
}
