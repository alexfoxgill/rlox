use std::error::Error;

use crate::value::Value;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpCode {
    Constant,

    Nil,
    True,
    False,

    Equal,
    Greater,
    Less,

    Add,
    Subtract,
    Multiply,
    Divide,

    Not,
    Negate,
    Return,

    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
    Call,
}

impl TryFrom<u8> for OpCode {
    type Error = Box<dyn Error>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use OpCode::*;
        Ok(match value {
            x if x == Constant as u8 => Constant,

            x if x == Nil as u8 => Nil,
            x if x == True as u8 => True,
            x if x == False as u8 => False,

            x if x == Equal as u8 => Equal,
            x if x == Greater as u8 => Greater,
            x if x == Less as u8 => Less,

            x if x == Add as u8 => Add,
            x if x == Subtract as u8 => Subtract,
            x if x == Multiply as u8 => Multiply,
            x if x == Divide as u8 => Divide,

            x if x == Not as u8 => Not,
            x if x == Negate as u8 => Negate,
            x if x == Return as u8 => Return,

            x if x == Print as u8 => Print,
            x if x == Pop as u8 => Pop,
            x if x == DefineGlobal as u8 => DefineGlobal,
            x if x == GetGlobal as u8 => GetGlobal,
            x if x == SetGlobal as u8 => SetGlobal,

            x if x == GetLocal as u8 => GetLocal,
            x if x == SetLocal as u8 => SetLocal,

            x if x == JumpIfFalse as u8 => JumpIfFalse,
            x if x == Jump as u8 => Jump,

            x if x == Loop as u8 => Loop,
            x if x == Call as u8 => Call,

            _ => return Err("Unknown opcode".into()),
        })
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::with_capacity(8),
            constants: Vec::with_capacity(8),
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

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
