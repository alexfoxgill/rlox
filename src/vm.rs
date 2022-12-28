use crate::{chunk::{Chunk, OpCode}, value::{Value, Object}, debug::{print_value, disassemble_instruction}, compiler::compile, string_intern::StringInterner};

pub struct VM {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack: Vec<Value>,
    pub strings: StringInterner,
}

impl VM {
    pub fn new(chunk: Chunk, strings: StringInterner) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
            strings
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
        self.chunk.constants.values[byte as usize].clone()
    }

    fn binary_op<F: Fn(f64, f64) -> Value>(&mut self, f: F) -> bool {
        let b = self.pop();
        let a = self.pop();

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

                OpCode::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Bool(a == b));
                }

                OpCode::Greater => if !self.binary_op(|a,b| Value::Bool(a > b)) {
                    return InterpretResult::RuntimeError
                }

                OpCode::Less => if !self.binary_op(|a,b| Value::Bool(a < b)) {
                    return InterpretResult::RuntimeError
                }

                OpCode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a.as_string(), b.as_string()) {
                        (Some(a), Some(b)) => {
                            let (_, concat) = {
                                let mut concat = a.to_owned();
                                concat.push_str(b);
                                self.strings.intern(&concat)
                            };
                            self.push(Value::Object(Box::new(Object::String(concat))));
                            continue;
                        }
                        _ => ()
                    }

                    match (a.as_number(), b.as_number()) {
                        (Some(a), Some(b)) => {
                            self.push(Value::Number(a + b));
                            continue;
                        },
                        _ => ()
                    }

                    self.runtime_error("Operands must be strings or numbers");
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

                OpCode::Not => {
                    let value = self.pop();
                    self.push(Value::Bool(is_falsey(value)));
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
    let mut strings = StringInterner::with_capacity(16);

    if !compile(source, &mut chunk, &mut strings) {
        chunk.free();
        return InterpretResult::CompileError;
    }

    let mut vm = VM::new(chunk, strings);
    let res = vm.run();
    vm.free();
    res
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(b) => !b,
        Value::Number(_) => false,
        Value::Object(_) => false
    }
}