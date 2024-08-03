use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Module};
use kali_ast::{BinaryOp, Literal, UnaryOp};
use kali_ir::Operator;

/// A stack translation unit.
pub struct StackTranslationUnit {
    /// The program to be translated.
    program: Vec<Operator>,
    /// The function builder context.
    builder_ctx: FunctionBuilderContext,
    /// The code gneration context.
    ctx: codegen::Context,
    /// The data context.
    data_description: DataDescription,
    /// The JIT module.
    module: JITModule,
}

impl StackTranslationUnit {
    /// Create a new stack translation unit.
    pub fn new(program: Vec<Operator>) -> Self {
        // flags
        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "false").unwrap();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();

        // isa
        let isa_builder = cranelift_native::builder()
            .unwrap_or_else(|msg| panic!("error creating Cranelift native builder: {}", msg));
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        // module
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Self {
            program,
            builder_ctx: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            data_description: DataDescription::new(),
            module,
        }
    }

    /// Translate the program to Cranelift IR.
    pub fn translate(&mut self) {
        // preamble
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);

        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // translate program
        for op in &self.program {
            op.translate(&self.module, &mut builder);
        }

        builder.finalize();
    }

    /// Get the program.
    pub fn into_inner(self) -> Vec<Operator> {
        self.program
    }
}

/// Translate the program to Cranelift IR.
trait Translate {
    /// Translate the expression to Cranelift IR.
    fn translate(&self, module: &JITModule, builder: &mut FunctionBuilder);

    /// Push a value to the stack.
    fn push_value(
        &self,
        module: &JITModule,
        builder: &mut FunctionBuilder,
        value: Value,
        size: usize,
    ) {
        let pointer = module.target_config().pointer_type();
        let sp = builder.ins().get_stack_pointer(pointer);
        // stack grows downwards, so pushing decreases the stack pointer
        let sp_offset = builder.ins().iconst(pointer, size as i64);
        let sp = builder.ins().isub(sp, sp_offset);
        builder.ins().store(MemFlags::new(), value, sp, 0);
    }

    /// Pop a value from the stack.
    fn pop_value(&self, module: &JITModule, builder: &mut FunctionBuilder, size: usize) -> Value {
        let pointer = module.target_config().pointer_type();
        let sp = builder.ins().get_stack_pointer(pointer);
        // stack grows downwards, so removing increases the stack pointer
        let sp_offset = builder.ins().iconst(pointer, size as i64);
        let sp = builder.ins().isub(sp, sp_offset);
        builder.ins().load(types::I64, MemFlags::new(), sp, 0)
    }

    /// Pop two values from the stack. The values must have the same size.
    fn pop_value_2(
        &self,
        module: &JITModule,
        builder: &mut FunctionBuilder,
        size: usize,
    ) -> (Value, Value) {
        let pointer = module.target_config().pointer_type();
        let sp = builder.ins().get_stack_pointer(pointer);
        let sp_offset = builder.ins().iconst(pointer, size as i64);
        // stack grows downwards, so removing increases the stack pointer
        let sp = builder.ins().iadd(sp, sp_offset);
        let a = builder.ins().load(types::I64, MemFlags::new(), sp, 0);
        let b = builder
            .ins()
            .load(types::I64, MemFlags::new(), sp, size as i32);
        (a, b)
    }
}

impl Translate for Operator {
    fn translate(&self, module: &JITModule, builder: &mut FunctionBuilder) {
        match self {
            Operator::PushLiteral(literal) => {
                let value = match literal {
                    Literal::Int(value) => builder.ins().iconst(types::I64, *value),
                    Literal::Float(value) => builder.ins().f64const(*value),
                    Literal::Bool(value) => builder.ins().iconst(types::I8, *value as i64),
                    Literal::String(value) => todo!(),
                    Literal::Unit => todo!(),
                    Literal::Array(value) => todo!(),
                    Literal::Tuple(value) => todo!(),
                    Literal::Struct(value) => todo!(),
                };
                // push value to the stack
                self.push_value(&module, builder, value, 8);
            }
            Operator::PushVariable(_) => todo!(),
            Operator::BinaryOp(op) => {
                // pop two values from the stack
                let (a, b) = self.pop_value_2(&module, builder, 8);

                // perform operation
                let result = match op {
                    BinaryOp::Add => builder.ins().iadd(a, b),
                    BinaryOp::Subtract => builder.ins().isub(a, b),
                    BinaryOp::Multiply => builder.ins().imul(a, b),
                    BinaryOp::Divide => builder.ins().sdiv(a, b),
                    BinaryOp::Exponentiate => todo!("exponentiate"),
                    BinaryOp::Modulo => {
                        let div = builder.ins().sdiv(a, b);
                        let mul = builder.ins().imul(div, b);
                        builder.ins().isub(a, mul)
                    }
                    BinaryOp::Equal => builder.ins().icmp(IntCC::Equal, a, b),
                    BinaryOp::NotEqual => builder.ins().icmp(IntCC::NotEqual, a, b),
                    BinaryOp::LessThan => builder.ins().icmp(IntCC::SignedLessThan, a, b),
                    BinaryOp::LessThanOrEqual => {
                        builder.ins().icmp(IntCC::SignedLessThanOrEqual, a, b)
                    }
                    BinaryOp::GreaterThan => builder.ins().icmp(IntCC::SignedGreaterThan, a, b),
                    BinaryOp::GreaterThanOrEqual => {
                        builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, a, b)
                    }
                    BinaryOp::LogicalAnd => {
                        let value1 = builder.ins().icmp_imm(IntCC::NotEqual, a, 0);
                        let value2 = builder.ins().icmp_imm(IntCC::NotEqual, b, 0);
                        builder.ins().band(value1, value2)
                    }
                    BinaryOp::LogicalOr => {
                        let value1 = builder.ins().icmp_imm(IntCC::NotEqual, a, 0);
                        let value2 = builder.ins().icmp_imm(IntCC::NotEqual, b, 0);
                        builder.ins().bor(value1, value2)
                    }
                    BinaryOp::BitwiseAnd => builder.ins().band(a, b),
                    BinaryOp::BitwiseOr => builder.ins().bor(a, b),
                    BinaryOp::BitwiseXor => builder.ins().bxor(a, b),
                    BinaryOp::BitwiseShiftLeft => builder.ins().ishl(a, b),
                    BinaryOp::BitwiseShiftRight => builder.ins().sshr(a, b),
                    BinaryOp::Concatenate => todo!(),
                };

                // push result to the stack
                self.push_value(&module, builder, result, 8);
            }
            Operator::UnaryOp(op) => {
                // pop value from the stack
                let value = self.pop_value(&module, builder, 4);

                // perform operation
                let result = match op {
                    UnaryOp::Negate => builder.ins().ineg(value),
                    UnaryOp::LogicalNot => builder.ins().icmp_imm(IntCC::Equal, value, 0),
                    UnaryOp::BitwiseNot => builder.ins().bnot(value),
                };

                // push result to the stack
                self.push_value(&module, builder, result, 4);
            }
            Operator::Jump(_) => {
                // create a new block
                let block = builder.create_block();
                builder.ins().jump(block, &[]);
                builder.seal_block(block);

                // switch to the new block
                builder.switch_to_block(block);
            }
            Operator::ConditionalJump(_) => {
                // pop value from the stackl
                let value = self.pop_value(&module, builder, 4);

                // create blocks for the conditional jump
                let body = builder.create_block();
                let otherwise = builder.create_block();

                // jump to the body if the condition is true
                builder.ins().brif(value, body, &[], otherwise, &[]);
                builder.seal_block(body);

                // the next block is the body - might need some help from the stack operations to know
                // where it ends?
                builder.switch_to_block(body);
            }
            Operator::Pop => {
                // pop value from the stack
                self.pop_value(&module, builder, 8);
            }
        }
    }
}
