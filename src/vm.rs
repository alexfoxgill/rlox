use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    chunk::{Chunk, OpCode},
    compiler::compile,
    config::Config,
    debug::{disassemble_instruction, print_value},
    memory::Memory,
    string_intern::StrId,
    value::{NativeFunction, Object, Value, Closure},
};

pub fn interpret(source: &str, config: Config) -> InterpretResult {
    if let Some(mut vm) = compile(Rc::from(source), config) {
        vm.run()
    } else {
        return InterpretResult::CompileError;
    }
}

pub struct VM {
    pub config: Config,
    pub frames: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub globals: HashMap<StrId, Value>,
    pub memory: Memory,
}

impl VM {
    pub fn new(memory: Memory, config: Config) -> Self {
        let mut vm = Self {
            config,
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            memory,
        };
        vm.define_native("clock", move |_args| {
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
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
            {
                let c = self.frame().closure;
                let f = self.memory.closures[c].function;
                let ip = self.frame().instruction_pointer;
                let chunk = &self.memory.functions[f].chunk;

                let output = &mut self.config.vm_debug;

                write!(output, "          ").unwrap();
                for value in self.stack.iter() {
                    write!(output, "[ ").unwrap();
                    print_value(value, &self.memory, output);
                    write!(output, " ]").unwrap();
                }
                write!(output, "\n").unwrap();

                disassemble_instruction(&chunk, ip, &self.memory, output);
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
                        return InterpretResult::OK;
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
                            let concat = {
                                let mut concat = a.to_owned();
                                concat.push_str(b);
                                self.memory.string_intern(&concat)
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
                    print_value(&val, &self.memory, &mut self.config.print_output);
                    write!(&mut self.config.print_output, "\n").unwrap();
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
                            let name = self.memory.get_string(global_name);
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
                            let name = self.memory.get_string(global_name);
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
                        return InterpretResult::RuntimeError;
                    }
                }

                OpCode::Closure => {
                    if let Some(function) = self.read_constant().as_function() {
                        let closure = self.new_closure(function);
                        self.push(Value::Object(Rc::new(Object::Closure(closure))));
                    } else {
                        self.runtime_error("Expected closure");
                        return InterpretResult::RuntimeError;
                    }
                }
            }
        }
    }

    pub fn new_closure(&mut self, function_id: usize) -> usize {
        let closure = self.memory.closures.len();
        self.memory.closures.push(Closure {
            function: function_id,
        });
        closure
    }

    fn call_value(&mut self, value: Value, arg_count: u8) -> bool {
        if let Some(c_id) = value.as_closure() {
            self.call(c_id, arg_count as usize)
        } else if let Some(f_id) = value.as_native_function() {
            let native = &self.memory.natives[f_id];
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

    pub fn call(&mut self, c_id: usize, arg_count: usize) -> bool {
        let closure = &self.memory.closures[c_id];
        let f_id = closure.function;
        let arity = self.memory.functions[f_id].arity;
        if arg_count != arity {
            self.runtime_error(&format!("Expected {arity} arguments but got {arg_count}"));
            return false;
        }

        if self.frames.len() == 64 {
            self.runtime_error("Stack overflow");
            return false;
        }

        self.frames.push(CallFrame {
            closure: c_id,
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
        let c_id = self.frame().closure;
        let f = self.memory.closures[c_id].function;
        &self.memory.functions[f].chunk
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
        write!(self.config.vm_error, "{error}").unwrap();

        let ins = self.chunk().code[self.frame().instruction_pointer - 1];
        let line = self.chunk().lines[ins as usize];
        write!(self.config.vm_error, "[line {line}] in script").unwrap();

        for frame in self.frames.iter().rev() {
            let f_id = self.memory.closures[frame.closure].function;
            let function = &self.memory.functions[f_id];
            let name = self.memory.get_string(function.name);
            writeln!(
                self.config.vm_error,
                "[line {} in {}]",
                function.chunk.lines[frame.instruction_pointer], name
            )
            .unwrap();
        }

        self.reset_stack();
    }

    fn define_native<F: Fn(&[Value]) -> Value + 'static>(&mut self, name: &str, function: F) {
        let idx = self.memory.natives.len();
        let name = self.memory.string_id(name);
        self.memory
            .natives
            .push(NativeFunction::new(name, Box::new(function)));
        self.globals
            .insert(name, Value::Object(Rc::new(Object::NativeFunction(idx))));
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
    pub closure: usize,
    pub instruction_pointer: usize,
    pub slot_start: usize,
}
