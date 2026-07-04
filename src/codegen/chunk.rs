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
    pub fn new(ast: &Ast, arena: Rodeo) -> Self {
        let mut builder = ChunkBuilder::new(arena);
        compile_ast(&mut builder, ast);
        builder.chunk()
    }
}
