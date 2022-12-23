pub mod chunk;
pub mod debug;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests {
    use crate::{
        chunk::{Chunk, OpCode}, vm::VM,
    };

    #[test]
    fn test_chunk() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(1.2);
        chunk.write_opcode(OpCode::Constant, 123);
        chunk.write(constant as u8, 123);

        let constant = chunk.add_constant(3.4);
        chunk.write_opcode(OpCode::Constant, 123);
        chunk.write(constant as u8, 123);

        chunk.write_opcode(OpCode::Add, 123);

        let constant = chunk.add_constant(5.6);
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
