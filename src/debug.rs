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
            return offset + 1;
        }
    };

    match op_code {
        | OpCode::Jump
        | OpCode::JumpIfFalse => {
            jump_instruction(op_code, 1, chunk, offset)
        }

        | OpCode::Constant
        | OpCode::DefineGlobal
        | OpCode::GetGlobal
        | OpCode::SetGlobal => {
            constant_instruction(op_code, chunk, offset)
        }

        | OpCode::GetLocal
        | OpCode::SetLocal => {
            byte_instruction(op_code, chunk, offset)
        }

        | OpCode::Nil
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
        | OpCode::Pop => simple_instruction(op_code, offset),
    }
}

fn jump_instruction(op_code: OpCode, sign: usize, chunk: &Chunk, offset: usize) -> usize {
    let b1 = chunk.code[offset + 1] as u16;
    let b2 = chunk.code[offset + 2] as u16;
    let jump = (b1 << 8) | b2;
    let s = format!("{op_code:?}");
    let dest = (offset + 3 + sign) * (jump as usize);
    println!("{s:<16} {offset:0>4} -> {dest}");
    offset + 3
}

fn constant_instruction(op_code: OpCode, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code[offset + 1];
    let s = format!("{op_code:?}");
    print!("{s:<16} {constant:>4} ");
    print_value(&chunk.constants.values[constant as usize]);
    print!("\n");
    offset + 2
}

fn byte_instruction(op_code: OpCode, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.code[offset + 1];
    let s = format!("{op_code:?}");
    println!("{s:<16} {slot:0>4}");
    offset + 2
}

fn simple_instruction(op_code: OpCode, offset: usize) -> usize {
    let s = format!("{op_code:?}");
    println!("{s:<16}");
    offset + 1
}

pub fn print_value(value: &Value) {
    print!("{value}")
}
