//! Implements a (mediocre) stack-based virtual machine for executing Kali code.

use kali_ast::Literal;
use kali_ir::Module;

mod heap;
mod stack;

use heap::Heap;
use stack::Stack;

const STACK_SIZE: usize = 32768;

/// A runtime for executing Kali code.
pub struct Runtime {
    /// The runtime stack.
    stack: Stack,
    /// The runtime heap.
    heap: Heap,
    /// Program counter.
    pc: usize,
    /// The module to execute.
    module: Module,
}

impl Runtime {
    /// Create a new runtime.
    pub fn new(module: Module) -> Self {
        Self {
            stack: Stack::new(STACK_SIZE),
            heap: Heap::new(),
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
}
