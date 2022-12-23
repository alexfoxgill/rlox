use crate::{chunk::{Chunk, OpCode}, value::Value, debug::{print_value, disassemble_instruction}};

pub struct VM {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack: Vec<Value>
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new()
        }
    }

    pub fn free(&mut self) {
        self.chunk.free();
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    pub fn read_op_code(&mut self) -> Option<OpCode> {
        self.read_byte().try_into().ok()
    }

    pub fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.constants.values[byte as usize]
    }

    fn binary_op<F: Fn(Value, Value) -> Value>(&mut self, f: F) {
        let a = self.pop();
        let b = self.pop();
        self.push(f(a, b))
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            print!("          ");
            for value in self.stack.iter() {
                print!("[ ");
                print_value(value);
                print!(" ]");
            }
            print!("\n");

            disassemble_instruction(&self.chunk, self.ip);

            let op_code = match self.read_op_code() {
                Some(x) => x,
                None =>  return InterpretResult::CompileError
            };

            match op_code {
                OpCode::Return => {
                    let res = self.pop();
                    print_value(&res);
                    print!("\n");
                    return InterpretResult::OK
                },

                OpCode::Add => self.binary_op(|a,b| a + b),
                OpCode::Subtract => self.binary_op(|a,b| a - b),
                OpCode::Multiply => self.binary_op(|a,b| a * b),
                OpCode::Divide => self.binary_op(|a,b| a / b),

                OpCode::Negate => {
                    let value = self.pop();
                    self.push(-value);
                },

                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
            }
        }
    }

    pub fn reset_stack(&mut self) {
        self.stack.clear();
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
}

pub enum InterpretResult {
    OK,
    CompileError,
    RuntimeError
}

pub fn interpret(chunk: Chunk) -> InterpretResult {
    VM::new(chunk).run()
}
