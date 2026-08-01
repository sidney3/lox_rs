use std::fmt;

use itertools::Itertools;

use super::instruction::Instruction;
use crate::runtime::Value;

#[derive(Debug, Clone)]
pub struct Chunk {
  pub instructions: Vec<Instruction>,
  pub constants: Vec<Value>,
}

impl fmt::Display for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Instructions: [{}]", self.instructions.iter().join(", "))?;

    Ok(())
  }
}
