use crate::{
    chunk::{Chunk, OpCode},
    memory::Memory,
    value::Value,
    vm::InstructionPointer,
};

use std::fmt::Write;

pub fn disassemble_chunk(chunk: &Chunk, name: &str, memory: &Memory, output: &mut impl Write) {
    writeln!(output, "== {name} ==").unwrap();

    let mut offset = InstructionPointer(0);
    while offset.0 < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset, memory, output);
    }
}

pub fn disassemble_instruction(
    chunk: &Chunk,
    mut offset: InstructionPointer,
    memory: &Memory,
    output: &mut impl Write,
) -> InstructionPointer {
    write!(output, "{offset} ").unwrap();
    let line = chunk.line(offset);
    if offset.0 > 0 && line == chunk.line(offset.minus(1)) {
        write!(output, "   | ").unwrap();
    } else {
        write!(output, "{line:>4} ").unwrap();
    }

    let byte = chunk.byte(offset);

    let op_code: OpCode = match byte.try_into() {
        Ok(x) => x,
        Err(_) => {
            write!(output, "Unknown opcode {byte}").unwrap();
            return offset.plus(1);
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
            offset.increment(1);
            let constant = chunk.constant(offset);
            offset.increment(1);
            let s = format!("{op_code:?}");
            write!(output, "{s:<16} {constant:?} ").unwrap();
            print_value(&chunk.constant_value(constant), memory, output);
            write!(output, "\n").unwrap();
            offset
        }
    }
}

fn jump_instruction(
    op_code: OpCode,
    sign: i32,
    chunk: &Chunk,
    offset: InstructionPointer,
    output: &mut impl Write,
) -> InstructionPointer {
    let b1 = chunk.byte(offset.plus(1)) as u16;
    let b2 = chunk.byte(offset.plus(2)) as u16;
    let jump = (b1 << 8) | b2;
    let s = format!("{op_code:?}");
    let dest = (offset.0 as i32 + 3) + (sign * jump as i32);
    writeln!(output, "{s:<16} {offset} -> {dest:0>4}").unwrap();
    offset.plus(3)
}

fn constant_instruction(
    op_code: OpCode,
    chunk: &Chunk,
    offset: InstructionPointer,
    memory: &Memory,
    output: &mut impl Write,
) -> InstructionPointer {
    let constant = chunk.constant(offset.plus(1));
    let s = format!("{op_code:?}");
    write!(output, "{s:<16} {constant:?} ").unwrap();
    print_value(&chunk.constant_value(constant), memory, output);
    write!(output, "\n").unwrap();
    offset.plus(2)
}

fn byte_instruction(
    op_code: OpCode,
    chunk: &Chunk,
    offset: InstructionPointer,
    output: &mut impl Write,
) -> InstructionPointer {
    let slot = chunk.byte(offset.plus(1));
    let s = format!("{op_code:?}");
    writeln!(output, "{s:<16} {slot:0>4}").unwrap();
    offset.plus(2)
}

fn simple_instruction(
    op_code: OpCode,
    offset: InstructionPointer,
    output: &mut impl Write,
) -> InstructionPointer {
    let s = format!("{op_code:?}");
    writeln!(output, "{s:<16}").unwrap();
    offset.plus(1)
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
        Value::String(s) => {
            write!(output, "\"{s}\"").unwrap();
        }
        Value::StringId(id) => {
            let s = memory.get_string(*id);
            write!(output, "\"{s}\"").unwrap();
        }
        Value::Function(id) => {
            let f = &memory.function(*id);
            let s = memory.get_string(f.name);
            write!(output, "<fn {s}>").unwrap();
        }
        Value::NativeFunction(id) => {
            let f = &memory.native(*id);
            let s = memory.get_string(f.name);
            write!(output, "<native fn {s}>").unwrap();
        }
        Value::Closure(id) => {
            let c = &memory.closure(*id);
            let f = &memory.function(c.function);
            let s = memory.get_string(f.name);
            write!(output, "<closure {s}>").unwrap();
        }
    }
}
