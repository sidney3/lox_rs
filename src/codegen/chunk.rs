use std::fmt;

use itertools::Itertools;

use super::chunk_builder::ChunkBuilder;
use super::compile::compile_ast;
use super::constant::Constant;
use super::instruction::Instruction;
use crate::frontend::ast::Ast;

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

impl fmt::Display for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Constants: [{}]", self.constants.iter().join(", "))?;
    writeln!(f, "Instructions: [{}]", self.instructions.iter().join(", "))?;

    Ok(())
  }
}
