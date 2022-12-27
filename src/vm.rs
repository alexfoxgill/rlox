use crate::{chunk::{Chunk, OpCode}, value::Value, debug::{print_value, disassemble_instruction}, compiler::compile};

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

    fn binary_op<F: Fn(f64, f64) -> Value>(&mut self, f: F) -> bool {
        let a = self.pop();
        let b = self.pop();

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                self.push(f(a, b));
                true
            },
            _ => {
                self.runtime_error("Operands must be numbers");
                false
            }
        }
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
                }

                OpCode::Add => if !self.binary_op(|a,b| Value::Number(a + b)) {
                    return InterpretResult::RuntimeError
                }
                OpCode::Subtract => if !self.binary_op(|a,b| Value::Number(a - b)) {
                    return InterpretResult::RuntimeError
                }
                OpCode::Multiply => if !self.binary_op(|a,b| Value::Number(a * b)) {
                    return InterpretResult::RuntimeError
                }
                OpCode::Divide => if !self.binary_op(|a,b| Value::Number(a / b)) {
                    return InterpretResult::RuntimeError
                }

                OpCode::Negate => {
                    let value = self.pop();

                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => {
                            self.runtime_error("Operand must be a number");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }

                OpCode::Nil => {
                    self.push(Value::Nil)
                }

                OpCode::True => {
                    self.push(Value::Bool(true))
                }

                OpCode::False => {
                    self.push(Value::Bool(false))
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

    fn runtime_error(&mut self, error: &str) {
        eprintln!("{error}");

        let ins = self.chunk.code[self.ip - 1];
        let line = self.chunk.lines[ins as usize];
        eprintln!("[line {line}] in script");
        self.reset_stack();
    }
}

pub enum InterpretResult {
    OK,
    CompileError,
    RuntimeError
}

pub fn interpret(source: &str) -> InterpretResult {
    let mut chunk = Chunk::new();

    if !compile(source, &mut chunk) {
        chunk.free();
        return InterpretResult::CompileError;
    }

    let mut vm = VM::new(chunk);
    let res = vm.run();
    vm.free();
    res
}
