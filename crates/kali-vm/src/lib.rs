//! Implements a (mediocre) stack-based virtual machine for executing Kali code.

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
}
