use super::chunk_builder::ChunkBuilder;
use super::compile::compile_ast;
use super::constant::Constant;
use super::instruction::Instruction;
use crate::frontend::ast::Ast;

use lasso::Rodeo;

pub struct Chunk {
    pub constants: Vec<Constant>,
    pub instructions: Vec<Instruction>,
}

impl Chunk {
    pub fn new(ast: &Ast) -> Self {
        let mut builder = ChunkBuilder::new();
        compile_ast(&mut builder, ast);
        builder.chunk()
    }
}
