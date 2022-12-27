use crate::vm::interpret;

pub mod compiler;
pub mod chunk;
pub mod debug;
pub mod value;
pub mod scanner;
pub mod vm;

fn repl() {
    loop {
        print!("> ");

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        interpret(&line);
        
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chunk::{Chunk, OpCode}, vm::VM, value::Value,
    };

    use super::*;

    #[test]
    fn test_expression() {
        let res = interpret("!(5 - 4 > 3 * 2 == !nil)");
    }

    #[test]
    fn test_chunk() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(Value::Number(1.2));
        chunk.write_opcode(OpCode::Constant, 123);
        chunk.write(constant as u8, 123);

        let constant = chunk.add_constant(Value::Number(3.4));
        chunk.write_opcode(OpCode::Constant, 123);
        chunk.write(constant as u8, 123);

        chunk.write_opcode(OpCode::Add, 123);

        let constant = chunk.add_constant(Value::Number(5.6));
        chunk.write_opcode(OpCode::Constant, 123);
        chunk.write(constant as u8, 123);

        chunk.write_opcode(OpCode::Divide, 123);
        chunk.write_opcode(OpCode::Negate, 123);
        chunk.write_opcode(OpCode::Return, 123);

        let mut vm = VM::new(chunk);
        vm.run();
        vm.free();
    }
}
