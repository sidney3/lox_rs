use super::chunk::Chunk;
use super::constant::Constant;
use super::instruction::Instruction;
use lasso::Rodeo;

pub struct ChunkBuilder {
    constants: Vec<Constant>,
    instructions: Vec<Instruction>,
}

impl ChunkBuilder {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        let index = self.constants.len();
        self.constants.push(constant);
        index
    }

    pub fn chunk(self) -> Chunk {
        Chunk {
            constants: self.constants,
            instructions: self.instructions,
        }
    }
}
