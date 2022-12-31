use std::{
    collections::{hash_map::Entry, HashMap},
    rc::Rc, time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    chunk::{Chunk, OpCode},
    compiler::compile,
    debug::{disassemble_instruction, print_value},
    string_intern::{StrId, StringInterner},
    value::{Object, Value, Function, NativeFunction},
};

pub fn interpret(source: &str) -> InterpretResult {
    if let Some(mut vm) = compile(Rc::from(source)) {
        vm.run()
    } else {
        return InterpretResult::CompileError;
    }

}

pub struct VM {
    pub frames: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub strings: StringInterner,
    pub functions: Vec<Function>,
    pub globals: HashMap<StrId, Value>,
    pub natives: Vec<NativeFunction>,
}

impl VM {
    pub fn new(strings: StringInterner, functions: Vec<Function>) -> Self {
        let mut vm = Self {
            frames: Vec::new(),
            stack: Vec::new(),
            strings,
            functions,
            globals: HashMap::new(),
            natives: Vec::new()
        };
        vm.define_global("clock", move |_args| {
            let t = SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            Value::Number(t as f64)
        });
        vm
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.chunk().code[self.frame().instruction_pointer];
        self.frame_mut().instruction_pointer += 1;
        byte
    }

    pub fn read_short(&mut self) -> usize {
        let b1 = self.read_byte() as usize;
        let b2 = self.read_byte() as usize;
        (b1 << 8) | b2
    }

    pub fn read_op_code(&mut self) -> Option<OpCode> {
        self.read_byte().try_into().ok()
    }

    pub fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk().constants[byte as usize].clone()
    }

    fn binary_op<F: Fn(f64, f64) -> Value>(&mut self, f: F) -> bool {
        let b = self.pop();
        let a = self.pop();

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                self.push(f(a, b));
                true
            }
            _ => {
                self.runtime_error("Operands must be numbers");
                false
            }
        }
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            if false {
                print!("          ");
                for value in self.stack.iter() {
                    print!("[ ");
                    print_value(value, &self.strings, &self.functions, &self.natives);
                    print!(" ]");
                }
                print!("\n");
    
                disassemble_instruction(&self.chunk(), self.frame().instruction_pointer, &self.strings, &self.functions, &self.natives);
            }

            let op_code = match self.read_op_code() {
                Some(x) => x,
                None => return InterpretResult::CompileError,
            };

            match op_code {
                OpCode::Return => {
                    let result = self.pop();
                    let frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        self.pop();
                        return InterpretResult::OK
                    }

                    self.stack.truncate(frame.slot_start);
                    self.push(result);
                }

                OpCode::Pop => {
                    self.pop();
                }

                OpCode::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Bool(a == b));
                }

                OpCode::Greater => {
                    if !self.binary_op(|a, b| Value::Bool(a > b)) {
                        return InterpretResult::RuntimeError;
                    }
                }

                OpCode::Less => {
                    if !self.binary_op(|a, b| Value::Bool(a < b)) {
                        return InterpretResult::RuntimeError;
                    }
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
                            self.push(Value::Object(Rc::new(Object::String(concat))));
                            continue;
                        }
                        _ => (),
                    }

                    match (a.as_number(), b.as_number()) {
                        (Some(a), Some(b)) => {
                            self.push(Value::Number(a + b));
                            continue;
                        }
                        _ => (),
                    }

                    self.runtime_error("Operands must be strings or numbers");
                    return InterpretResult::RuntimeError;
                }
                OpCode::Subtract => {
                    if !self.binary_op(|a, b| Value::Number(a - b)) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Multiply => {
                    if !self.binary_op(|a, b| Value::Number(a * b)) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Divide => {
                    if !self.binary_op(|a, b| Value::Number(a / b)) {
                        return InterpretResult::RuntimeError;
                    }
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

                OpCode::Nil => self.push(Value::Nil),

                OpCode::True => self.push(Value::Bool(true)),

                OpCode::False => self.push(Value::Bool(false)),

                OpCode::Print => {
                    let val = self.pop();
                    print_value(&val, &self.strings, &self.functions, &self.natives);
                    println!("");
                }

                OpCode::DefineGlobal => {
                    let global_name = self.read_constant().as_string_id().unwrap();
                    let val = self.pop();
                    self.globals.insert(global_name, val);
                }

                OpCode::GetGlobal => {
                    let global_name = self.read_constant().as_string_id().unwrap();
                    match self.globals.get(&global_name) {
                        Some(value) => self.push(value.clone()),
                        None => {
                            let name = self.strings.lookup(global_name);
                            self.runtime_error(&format!("Undefined variable '{name}'"));
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::SetGlobal => {
                    let global_name = self.read_constant().as_string_id().unwrap();
                    let val = self.peek(0);
                    match self.globals.entry(global_name) {
                        Entry::Occupied(mut e) => {
                            e.insert(val);
                        }
                        Entry::Vacant(_) => {
                            let name = self.strings.lookup(global_name);
                            self.runtime_error(&format!("Undefined variable' {name}'"));
                            return InterpretResult::RuntimeError;
                        }
                    }
                }

                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    let slot = self.frame().slot_start + slot;
                    let value = self.stack[slot].clone();
                    self.push(value);
                }

                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let slot = self.frame().slot_start + slot;
                    let value = self.peek(0);
                    self.stack[slot] = value;
                }

                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if is_falsey(self.peek(0)) {
                        self.frame_mut().instruction_pointer += offset;
                    }
                }

                OpCode::Jump => {
                    let offset = self.read_short();
                    self.frame_mut().instruction_pointer += offset;
                }

                OpCode::Loop => {
                    let offset = self.read_short();
                    self.frame_mut().instruction_pointer -= offset;
                }

                OpCode::Call => {
                    let arg_count = self.read_byte();
                    if !self.call_value(self.peek(arg_count as usize), arg_count) {
                        return InterpretResult::RuntimeError
                    }
                }
            }
        }
    }

    fn call_value(&mut self, value: Value, arg_count: u8) -> bool {
        if let Some(f_id) = value.as_function() {
            self.call(f_id, arg_count as usize)
        } else if let Some(f_id) = value.as_native_function() {
            let native = &self.natives[f_id];
            let init_stack = self.stack.len() - arg_count as usize;
            let args = &self.stack[init_stack..];
            let res = (native.callable)(args);
            self.stack.truncate(init_stack);
            self.push(res);
            true
        } else {
            self.runtime_error("Can only call functions and classes");
            false
        }
    }

    pub fn call(&mut self, f_id: usize, arg_count: usize) -> bool {
        let arity = self.functions[f_id].arity;
        if arg_count != arity {
            self.runtime_error(&format!("Expected {arity} arguments but got {arg_count}"));
            return false;
        }

        if self.frames.len() == 64 {
            self.runtime_error("Stack overflow");
            return false;
        }

        self.frames.push(CallFrame {
            function: f_id,
            instruction_pointer: 0,
            slot_start: self.stack.len() - arg_count - 1,
        });
        true
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn chunk(&self) -> &Chunk {
        let f = self.frame().function;
        &self.functions[f].chunk
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

    pub fn peek(&self, i: usize) -> Value {
        self.stack.iter().rev().nth(i).unwrap().clone()
    }

    fn runtime_error(&mut self, error: &str) {
        eprintln!("{error}");

        let ins = self.chunk().code[self.frame().instruction_pointer - 1];
        let line = self.chunk().lines[ins as usize];
        eprintln!("[line {line}] in script");

        for frame in self.frames.iter().rev() {
            let function = &self.functions[frame.function];
            let name = self.strings.lookup(function.name);
            eprintln!("[line {} in {}]",
                function.chunk.lines[frame.instruction_pointer],
                name);
        }

        self.reset_stack();
    }

    fn define_global<F: Fn(&[Value]) -> Value + 'static>(&mut self, name: &str, function: F) {
        let idx = self.natives.len();
        let (name, _) = self.strings.intern(name);
        self.natives.push(NativeFunction::new(name, Box::new(function)));
        self.globals.insert(name, Value::Object(Rc::new(Object::NativeFunction(idx))));
    }
}

pub enum InterpretResult {
    OK,
    CompileError,
    RuntimeError,
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(b) => !b,
        Value::Number(_) => false,
        Value::Object(_) => false,
    }
}

pub struct CallFrame {
    pub function: usize,
    pub instruction_pointer: usize,
    pub slot_start: usize
}