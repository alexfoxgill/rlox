use crate::{
    chunk::{Chunk, OpCode},
    memory::Memory,
    value::{Object, Value},
};

use std::fmt::Write;

pub fn disassemble_chunk(chunk: &Chunk, name: &str, memory: &Memory, output: &mut impl Write) {
    writeln!(output, "== {name} ==").unwrap();

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset, memory, output);
    }
}

pub fn disassemble_instruction(
    chunk: &Chunk,
    mut offset: usize,
    memory: &Memory,
    output: &mut impl Write,
) -> usize {
    write!(output, "{offset:0>4} ").unwrap();
    let line = chunk.lines[offset];
    if offset > 0 && line == chunk.lines[offset - 1] {
        write!(output, "   | ").unwrap();
    } else {
        write!(output, "{line:>4} ").unwrap();
    }

    let byte = chunk.code[offset];

    let op_code: OpCode = match byte.try_into() {
        Ok(x) => x,
        Err(_) => {
            write!(output, "Unknown opcode {byte}").unwrap();
            return offset + 1;
        }
    };

    match op_code {
        OpCode::Loop => jump_instruction(op_code, -1, chunk, offset, output),

        OpCode::Jump | OpCode::JumpIfFalse => jump_instruction(op_code, 1, chunk, offset, output),

        OpCode::Constant | OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::SetGlobal => {
            constant_instruction(op_code, chunk, offset, memory, output)
        }

        OpCode::Call | OpCode::GetLocal | OpCode::SetLocal => {
            byte_instruction(op_code, chunk, offset, output)
        }

        OpCode::Nil
        | OpCode::True
        | OpCode::False
        | OpCode::Equal
        | OpCode::Less
        | OpCode::Greater
        | OpCode::Add
        | OpCode::Subtract
        | OpCode::Multiply
        | OpCode::Divide
        | OpCode::Not
        | OpCode::Negate
        | OpCode::Return
        | OpCode::Print
        | OpCode::Pop => simple_instruction(op_code, offset, output),

        OpCode::Closure => {
            offset += 1;
            let constant = chunk.code[offset];
            offset += 1;
            let s = format!("{op_code:?}");
            write!(output, "{s:<16} {:>4} ", constant).unwrap();
            print_value(&chunk.constants[constant as usize], memory, output);
            write!(output, "\n").unwrap();
            offset
        }
    }
}

fn jump_instruction(
    op_code: OpCode,
    sign: i32,
    chunk: &Chunk,
    offset: usize,
    output: &mut impl Write,
) -> usize {
    let b1 = chunk.code[offset + 1] as u16;
    let b2 = chunk.code[offset + 2] as u16;
    let jump = (b1 << 8) | b2;
    let s = format!("{op_code:?}");
    let dest = (offset as i32 + 3) + (sign * jump as i32);
    writeln!(output, "{s:<16} {offset:0>4} -> {dest:0>4}").unwrap();
    offset + 3
}

fn constant_instruction(
    op_code: OpCode,
    chunk: &Chunk,
    offset: usize,
    memory: &Memory,
    output: &mut impl Write,
) -> usize {
    let constant = chunk.code[offset + 1];
    let s = format!("{op_code:?}");
    write!(output, "{s:<16} {constant:>4} ").unwrap();
    print_value(&chunk.constants[constant as usize], memory, output);
    write!(output, "\n").unwrap();
    offset + 2
}

fn byte_instruction(
    op_code: OpCode,
    chunk: &Chunk,
    offset: usize,
    output: &mut impl Write,
) -> usize {
    let slot = chunk.code[offset + 1];
    let s = format!("{op_code:?}");
    writeln!(output, "{s:<16} {slot:0>4}").unwrap();
    offset + 2
}

fn simple_instruction(op_code: OpCode, offset: usize, output: &mut impl Write) -> usize {
    let s = format!("{op_code:?}");
    writeln!(output, "{s:<16}").unwrap();
    offset + 1
}

pub fn print_value(value: &Value, memory: &Memory, output: &mut impl Write) {
    match value {
        Value::Nil => {
            write!(output, "nil").unwrap();
        }
        Value::Bool(b) => {
            write!(output, "{b}").unwrap();
        }
        Value::Number(n) => {
            write!(output, "{n}").unwrap();
        }
        Value::Object(o) => match o.as_ref() {
            Object::String(s) => {
                write!(output, "{s}").unwrap();
            }
            Object::StringId(id) => {
                let s = memory.get_string(*id);
                write!(output, "{s}").unwrap();
            }
            Object::Function(id) => {
                let f = &memory.functions[*id];
                let s = memory.get_string(f.name);
                write!(output, "<fn {s}>").unwrap();
            }
            Object::NativeFunction(id) => {
                let f = &memory.natives[*id];
                let s = memory.get_string(f.name);
                write!(output, "<native fn {s}>").unwrap();
            }
            Object::Closure(id) => {
                let c = &memory.closures[*id];
                let f = &memory.functions[c.function];
                let s = memory.get_string(f.name);
                write!(output, "<closure {s}>").unwrap();
            }
        },
    }
}
