use std::error::Error;

use crate::value::{Value, ValueArray};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant,

    Nil,
    True,
    False,

    Add,
    Subtract,
    Multiply,
    Divide,

    Negate,
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = Box<dyn Error>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use OpCode::*;
        Ok(match value {
            x if x == Constant as u8 => Constant,
            x if x == Add as u8 => Add,
            x if x == Subtract as u8 => Subtract,
            x if x == Multiply as u8 => Multiply,
            x if x == Divide as u8 => Divide,
            x if x == Negate as u8 => Negate,
            x if x == Return as u8 => Return,
            _ => return Err("Unknown opcode".into())
        })
    }
}


pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: ValueArray,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::with_capacity(8),
            constants: ValueArray::new(),
            lines: Vec::with_capacity(8),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_opcode(&mut self, op_code: OpCode, line: usize) {
        self.write(op_code as u8, line);
    }

    pub fn free(&mut self) {
        self.code.clear();
        self.constants.free();
        self.lines.clear();
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.len() - 1
    }
}
