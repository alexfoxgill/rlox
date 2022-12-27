use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{offset:0>4} ");
    let line = chunk.lines[offset];
    if offset > 0 && line == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{line:>4} ");
    }

    let byte = chunk.code[offset];

    let op_code: OpCode = match byte.try_into() {
        Ok(x) => x,
        Err(_) => {
            print!("Unknown opcode {byte}");
            return offset + 1
        },
    };

    match op_code {
        OpCode::Constant => {
            constant_instruction("OP_CONSTANT", chunk, offset)
        }
        OpCode::Nil => {
            simple_instruction("OP_NIL", offset)
        }
        OpCode::True => {
            simple_instruction("OP_TRUE", offset)
        }
        OpCode::False => {
            simple_instruction("OP_FALSE", offset)
        }
        OpCode::Add => {
            simple_instruction("OP_ADD", offset)
        }
        OpCode::Subtract => {
            simple_instruction("OP_SUBTRACT", offset)
        }
        OpCode::Multiply => {
            simple_instruction("OP_MULTIPLY", offset)
        }
        OpCode::Divide => {
            simple_instruction("OP_DIVIDE", offset)
        }
        OpCode::Negate => {
            simple_instruction("OP_NEGATE", offset)  
        }
        OpCode::Return => {
            simple_instruction("OP_RETURN", offset)
        }
    }
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1];
    print!("{name:16} {constant:>4} ");
    print_value(&chunk.constants.values[constant as usize]);
    print!("\n");
    offset + 2
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

pub fn print_value(value: &Value) {
    print!("{value}")
}
